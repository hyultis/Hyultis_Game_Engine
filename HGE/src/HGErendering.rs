use crate::components::TimeStats::TimeStats;
use crate::BuilderDevice::BuilderDevice;
use crate::HGEFrame::HGEFrame;
use crate::HGEMain::{HGEMain, HGEMain_secondarybuffer_type};
use crate::HGESwapchain::HGESwapchain;
use crate::HGEsubpass::HGEsubpass;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use ahash::HashMap;
use std::sync::Arc;
use std::time::Duration;
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, BlitImageInfo, CommandBufferInheritanceInfo, CommandBufferUsage, ImageBlit, SecondaryAutoCommandBuffer, SubpassEndInfo};
use vulkano::image::sampler::Filter;
use vulkano::image::ImageLayout;
use vulkano::render_pass::RenderPass;
use vulkano::swapchain::{Surface, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use vulkano::{sync, Validated, VulkanError};
use Htrace::Type::Type;
use Htrace::{namedThread, HTrace, HTraceError};

pub struct HGErendering
{
	//content storage
	_swapChainC: HGESwapchain,
	_Frame: HGEFrame,
	_builderDevice: Arc<BuilderDevice>,
	_renderpassC: Arc<RenderPass>,
	_surface: Arc<Surface>,
	_stdAllocCommand: StandardCommandBufferAllocator,
	
	// running data
	_previousFrameEnd: Option<Box<dyn GpuFuture + Send + Sync + 'static>>,
	_recreatSwapChain: bool,
	_generating: bool,
	_stats: HashMap<String, TimeStats>
}

impl HGErendering
{
	pub fn new(builderDevice: Arc<BuilderDevice>, surface: Arc<Surface>) -> anyhow::Result<Self>
	{
		let HGEswapchain = HGESwapchain::new(builderDevice.clone(), surface.clone());
		let frame_format = HGEswapchain.getImageFormat();
		let render_pass = Self::define_renderpass(&builderDevice, &HGEswapchain)?;
		
		let stdAlloccommand = StandardCommandBufferAllocator::new(builderDevice.device.clone(), Self::getDefaultAllocInfos());
		
		Ok(Self {
			_swapChainC: HGEswapchain,
			_Frame: HGEFrame::new(frame_format, builderDevice.depthformat),
			_builderDevice: builderDevice,
			_renderpassC: render_pass,
			_surface: surface,
			_stdAllocCommand: stdAlloccommand,
			_previousFrameEnd: None,
			_recreatSwapChain: true,
			_generating: false,
			_stats: Default::default(),
		})
	}
	
	pub fn drawStats(&self)
	{
		let mut tmp = String::new();
		self._stats.iter().for_each(|(key, data)| {
			if (tmp.is_empty())
			{
				tmp = format!("{} : {}", key, data);
			} else {
				tmp = format!("{}, {} : {}", tmp, key, data);
			}
		});
		println!("Stats : {}", tmp);
	}
	
	pub fn recreate(&mut self, builderDevice: Arc<BuilderDevice>, surface: Arc<Surface>)
	{
		self._swapChainC = HGESwapchain::new(builderDevice.clone(), surface.clone());
		self._Frame = HGEFrame::new(self._swapChainC.getImageFormat(), builderDevice.depthformat);
		self._builderDevice = builderDevice;
		if let Ok(newrenderpass) = Self::define_renderpass(&self._builderDevice, &self._swapChainC)
		{
			self._renderpassC = newrenderpass;
		}
		self._stdAllocCommand = StandardCommandBufferAllocator::new(self._builderDevice.device.clone(), Self::getDefaultAllocInfos());
		self._surface = surface;
		self._previousFrameEnd = None;
		self._recreatSwapChain = true;
		self._generating = false;
	}
	
	pub fn window_size_dependent_setup(&mut self)
	{
		self._Frame.replace(self._swapChainC.getImages(), self._renderpassC.clone());
		ManagerPipeline::singleton().pipelineRefresh(self._renderpassC.clone());
	}
	
	pub fn forceSwapchainRecreate(&mut self)
	{
		self._recreatSwapChain = true;
	}
	
	pub fn rendering(&mut self, durationFromLast: Duration) -> bool
	{
		if (self._generating)
		{
			return false;
		}
		
		// Whenever the window resizes we need to recreate everything dependent on the window size.
		// In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
		if self._recreatSwapChain
		{
			self._swapChainC.recreate();
			self.window_size_dependent_setup();
			self._recreatSwapChain = false;
		};
		
		
		if (self._swapChainC.getFpsLimiter() > 0)
		{
			if (durationFromLast.as_millis() < 1000 / (self._swapChainC.getFpsLimiter() as u128))
			{
				return false;
			}
		}
		
		
		self._generating = true;
		//self.getStatsOrCreate("main").setNow();
		self.SwapchainGenerateImg();
		self.getStatsOrCreate("main").putElapsed();
		self._generating = false;
		return true;
	}
	
	pub fn getAllocCmd(&self) -> &StandardCommandBufferAllocator
	{
		return &self._stdAllocCommand;
	}
	
	//////////// PRIVATE ///////////////
	
	fn SwapchainGenerateImg(&mut self)
	{
		if let Some(x) = &mut self._previousFrameEnd
		{
			x.cleanup_finished();
		}
		
		let queueGraphic = self._builderDevice.getQueueGraphic();
		let device = self._builderDevice.device.clone();
		let swapchain = self._swapChainC.get();
		
		// Before we can draw on the output, we have to *acquire* an image from the swapchain. If
		// no image is available (which happens if you submit draw commands too quickly), then the
		// function will block.
		// This operation returns the index of the image that we are allowed to draw upon.
		//
		// This function can block if no image is available. The parameter is an optional timeout
		// after which the function call will return an error.
		
		let (image_index, acquire_future) =
			match vulkano::swapchain::acquire_next_image(swapchain.clone(), None).map_err(Validated::unwrap) {
				Ok((image_index, suboptimal, acquire_future)) =>
					{
						// acquire_next_image can be successful, but suboptimal. This means that the swapchain image
						if suboptimal
						{
							self._recreatSwapChain = true;
						}
						(image_index, acquire_future)
					},
				Err(VulkanError::OutOfDate) =>
					{
						self._recreatSwapChain = true;
						return;
					}
				Err(e) =>
					{
						self._recreatSwapChain = true;
						HTrace!((Type::WARNING) "acquire_next_image {}", e);
						return;
					},
			};
		
		//println!("HGEMain: SecondaryCmdBuffer");
		let mut cmdBufTexture = match AutoCommandBufferBuilder::primary(
			&self._stdAllocCommand,
			queueGraphic.queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
		) {
			Ok(r) => r,
			Err(err) => {
				HTrace!("Cannot crate primary command buffer for texture : {}",err);
				return;
			}
		};
		
		let mut callbackCmdBuffer = Vec::new();
		if let Some(mut entry) = HGEMain::SecondaryCmdBuffer_drain(HGEMain_secondarybuffer_type::TEXTURE)
		{
			for x in entry.0
			{
				cmdBufTexture.execute_commands(x).unwrap();
			}
			callbackCmdBuffer.append(&mut entry.1);
		}
		
		// execute callback of updated cmdBuffer
		let _ = namedThread!(move ||{
				for func in callbackCmdBuffer {
					func();
				}
			});
		
		let mut cmdBuf = match AutoCommandBufferBuilder::primary(
			&self._stdAllocCommand,
			queueGraphic.queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
		) {
			Ok(r) => r,
			Err(err) => {
				HTrace!("Cannot crate primary command buffer for mesh : {}",err);
				return;
			}
		};
		
		self._Frame.clearBuffer(&mut cmdBuf, image_index);
		HGEsubpass::singleton().ExecAllPass(self._renderpassC.clone(), &mut cmdBuf, &self._Frame, &self._stdAllocCommand);
		HTraceError!(cmdBuf.end_render_pass(SubpassEndInfo::default()));
		
		//println!("HGEMain: future");
		let future = match self._previousFrameEnd.take() {
			None => sync::now(device.clone()).boxed_send_sync(),
			Some(x) => x
		};
		
		let future = future
			.join(acquire_future)
			.then_execute(queueGraphic.clone(), cmdBufTexture.build().unwrap()).unwrap()
			.then_execute(queueGraphic.clone(), cmdBuf.build().unwrap()).unwrap();
		
		let semaphore = future.then_signal_semaphore();
		
		let mut cmdBufDynamicRes = match AutoCommandBufferBuilder::primary(
			&self._stdAllocCommand,
			queueGraphic.queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
		) {
			Ok(r) => r,
			Err(err) => {
				HTrace!("Cannot crate primary command buffer dynamic resolution : {}",err);
				return;
			}
		};
		
		if (cfg!(feature = "dynamicresolution"))
		{
			let _ = cmdBufDynamicRes.execute_commands(self.dynamic_resolution(image_index).build().unwrap());
		}
		
		let fence = semaphore
			.then_execute(queueGraphic.clone(), cmdBufDynamicRes.build().unwrap())
			.unwrap()
			.then_swapchain_present(
				queueGraphic.clone(),
				SwapchainPresentInfo::swapchain_image_index(swapchain, image_index),
			)
			//.then_signal_semaphore();
			.then_signal_fence_and_flush();
		
		match fence.map_err(Validated::unwrap) {
			Ok(fence) => {
				let _ = fence.wait(None); // if not present, make cleanup_finished() crash.
				self._previousFrameEnd = Some(fence.boxed_send_sync());
			}
			Err(VulkanError::OutOfDate) => {
				self._recreatSwapChain = true;
			}
			Err(e) => {
				HTrace!("Failed to flush future: {:?}", e);
			}
		}
	}
	
	/// applied dynamic resolution system (move last image to swapimage with blit operation, return true if something gone wrong
	fn dynamic_resolution(&self, image_index: u32) -> AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>
	{
		let mut cmdBuffer = AutoCommandBufferBuilder::secondary(
			&self._stdAllocCommand,
			self._builderDevice.getQueueGraphic().queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
			CommandBufferInheritanceInfo {
				..Default::default()
			},
		).unwrap();
		
		let winInfos = HGEMain::singleton().getWindowInfos();
		
		//for imageswapchain in swapchain.getImages()
		let binding = self._swapChainC.getImages();
		let Some(imageswapchain) = binding.get(image_index as usize) else {
			return cmdBuffer;
		};
		
		let tmpfull = self._Frame.getImgFull();
		let _ = cmdBuffer
			.blit_image(BlitImageInfo {
				src_image_layout: ImageLayout::TransferSrcOptimal,
				dst_image_layout: ImageLayout::TransferDstOptimal,
				regions: [ImageBlit {
					src_subresource: tmpfull.image().subresource_layers(),
					src_offsets: [
						[0, 0, 0],
						[winInfos.width, winInfos.height, 1],
					],
					dst_subresource: imageswapchain.image().subresource_layers(),
					dst_offsets: [
						[0, 0, 0],
						[winInfos.raw_width, winInfos.raw_height, 1],
					],
					..Default::default()
				}]
					.into(),
				filter: Filter::Nearest,
				..BlitImageInfo::images(tmpfull.image().clone(), imageswapchain.image().clone())
			});
		return cmdBuffer;
	}
	
	fn define_renderpass(builderdevice: &BuilderDevice, swapchain: &HGESwapchain) -> anyhow::Result<Arc<RenderPass>>
	{
		let depthformat = builderdevice.depthformat;
		let imageformat = swapchain.getImageFormat();
		
		let render_pass;
		if (cfg!(feature = "dynamicresolution"))
		{
			render_pass = vulkano::ordered_passes_renderpass!(
				builderdevice.device.clone(),
				attachments: {
					render_UI: {
						format: imageformat,
						samples: 1,
						load_op: Clear,
						store_op: DontCare,
						final_layout: ImageLayout::ShaderReadOnlyOptimal,
					},
					render_WorldSolid: {
						format: imageformat,
						samples: 1,
						load_op: Clear,
						store_op: DontCare,
						final_layout: ImageLayout::ShaderReadOnlyOptimal,
					},
					render_Full: {
						 format: imageformat,
						samples: 1,
						load_op: Clear,
						store_op: Store,
						final_layout: ImageLayout::TransferSrcOptimal,
					},
					render_Final: {
						format: imageformat,
						samples: 1,
						load_op: Clear,
						store_op: DontCare,
						initial_layout: ImageLayout::TransferDstOptimal,
						final_layout: ImageLayout::PresentSrc
					},
					depthUI: {
						format: depthformat,
						samples: 1,
						load_op: Clear,
						store_op: DontCare,
					},
					depthSolid: {
						format: depthformat,
						samples: 1,
						load_op: Clear,
						store_op: DontCare,
					}
				},
				passes: [
				{ // interface pixel rendering
						color: [render_UI],
						depth_stencil: {depthUI},
						input:[]
				},
				{ // world solid pixel rendering
						color: [render_WorldSolid],
						depth_stencil: {depthSolid},
						input:[]
				},
				{ // interface transparent pixel rendering
						color: [render_Full],
						depth_stencil: {},
						input:[render_UI,render_WorldSolid]
				}]
			)?;
		} else {
			render_pass = vulkano::ordered_passes_renderpass!(
				builderdevice.device.clone(),
				attachments: {
					render_UI: {
						format: imageformat,
						samples: 1,
						load_op: Clear,
						store_op: DontCare,
						final_layout: ImageLayout::ShaderReadOnlyOptimal,
					},
					render_WorldSolid: {
						format: imageformat,
						samples: 1,
						load_op: Clear,
						store_op: DontCare,
						final_layout: ImageLayout::ShaderReadOnlyOptimal,
					},
					render_Final: {
						format: imageformat,
						samples: 1,
						load_op: Clear,
						store_op: Store,
					},
					depthUI: {
						format: depthformat,
						samples: 1,
						load_op: Clear,
						store_op: DontCare,
					},
					depthSolid: {
						format: depthformat,
						samples: 1,
						load_op: Clear,
						store_op: DontCare,
					}
				},
				passes:[
				{ // interface pixel rendering
					color: [render_UI],
					depth_stencil: {depthUI},
					input:[]
				},
				{ // world solid pixel rendering
					color: [render_WorldSolid],
					depth_stencil: {depthSolid},
					input:[]
				},
				{ // interface transparent pixel rendering
					color: [render_Final],
					depth_stencil: {},
					input:[render_UI,render_WorldSolid]
				}]
			)?;
		}
		
		ManagerPipeline::singleton().pipelineRefresh(render_pass.clone());
		return Ok(render_pass);
	}
	
	fn getDefaultAllocInfos() -> StandardCommandBufferAllocatorCreateInfo
	{
		let mut stdACInfos = StandardCommandBufferAllocatorCreateInfo {
			secondary_buffer_count: 32,
			..Default::default()
		};
		if (cfg!(target_os = "android"))
		{
			stdACInfos = StandardCommandBufferAllocatorCreateInfo {
				primary_buffer_count: 8,
				secondary_buffer_count: 8,
				..Default::default()
			};
		}
		return stdACInfos;
	}
	
	fn getStatsOrCreate(&mut self, key: impl Into<String>) -> &mut TimeStats
	{
		let key = key.into();
		
		if (!self._stats.contains_key(&key))
		{
			self._stats.insert(key.clone(), TimeStats::new());
		}
		
		return self._stats.get_mut(&key).unwrap();
	}
}
