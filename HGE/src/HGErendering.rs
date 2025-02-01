use crate::components::system::TimeStats::TimeStatsStorage;
use crate::BuilderDevice::BuilderDevice;
use crate::HGEFrame::HGEFrame;
use crate::HGEMain::{HGEMain, HGEMain_secondarybuffer_type};
use crate::HGESwapchain::HGESwapchain;
use crate::HGEsubpass::HGEsubpass;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use std::sync::Arc;
use std::time::Duration;
use vulkano::command_buffer::{
	AutoCommandBufferBuilder, BlitImageInfo, CommandBufferExecFuture, CommandBufferInheritanceInfo, CommandBufferUsage, ImageBlit, SecondaryAutoCommandBuffer, SubpassEndInfo,
};
use vulkano::device::Queue;
use vulkano::image::sampler::Filter;
use vulkano::image::ImageLayout;
use vulkano::render_pass::RenderPass;
use vulkano::swapchain::{Surface, Swapchain, SwapchainPresentInfo};
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

	// running data
	_previousFrameEnd: Option<Box<dyn GpuFuture + Send + Sync + 'static>>,
	_recreatSwapChain: bool,
	_generating: bool,
}

impl HGErendering
{
	pub fn new(builderDevice: Arc<BuilderDevice>, surface: Arc<Surface>) -> anyhow::Result<Self>
	{
		let HGEswapchain = HGESwapchain::new(builderDevice.clone(), surface.clone());
		let frame_format = HGEswapchain.getImageFormat();
		let render_pass = Self::define_renderpass(&builderDevice, &HGEswapchain)?;

		Ok(Self {
			_swapChainC: HGEswapchain,
			_Frame: HGEFrame::new(frame_format, builderDevice.depthformat),
			_builderDevice: builderDevice,
			_renderpassC: render_pass,
			_surface: surface,
			_previousFrameEnd: None,
			_recreatSwapChain: true,
			_generating: false,
		})
	}

	pub fn drawStats(&self)
	{
		println!("Stats : {}", TimeStatsStorage::singleton());
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

	pub fn rendering(&mut self, durationFromLast: Duration, preSwapFunc: impl Fn()) -> bool
	{
		if (self._generating)
		{
			return false;
		}

		// clear last rendering
		TimeStatsStorage::forceNow("R_Clean");
		if let Some(x) = &mut self._previousFrameEnd
		{
			x.cleanup_finished();
		}
		TimeStatsStorage::update("R_Clean");

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
		TimeStatsStorage::forceNow("R_main");
		self.SwapchainGenerateImg(preSwapFunc);
		TimeStatsStorage::update("R_main");
		self._generating = false;
		return true;
	}

	//////////// PRIVATE ///////////////

	fn SwapchainGenerateImg(&mut self, preSwapFunc: impl Fn())
	{
		TimeStatsStorage::forceNow("R_Clones");
		let queueGraphic = self._builderDevice.getQueueGraphic();
		let device = self._builderDevice.device.clone();
		let swapchain = self._swapChainC.get();
		TimeStatsStorage::update("R_Clones");

		// Before we can draw on the output, we have to *acquire* an image from the swapchain. If
		// no image is available (which happens if you submit draw commands too quickly), then the
		// function will block.
		// This operation returns the index of the image that we are allowed to draw upon.
		//
		// This function can block if no image is available. The parameter is an optional timeout
		// after which the function call will return an error.

		TimeStatsStorage::forceNow("R_NextImg");
		let (image_index, acquire_future) = match vulkano::swapchain::acquire_next_image(swapchain.clone(), None).map_err(Validated::unwrap)
		{
			Ok((image_index, suboptimal, acquire_future)) =>
			{
				// acquire_next_image can be successful, but suboptimal. This means that the swapchain image
				if suboptimal
				{
					self._recreatSwapChain = true;
				}
				(image_index, acquire_future)
			}
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
			}
		};
		TimeStatsStorage::update("R_NextImg");

		let future = match self._previousFrameEnd.take()
		{
			None => sync::now(device.clone()).boxed_send_sync(),
			Some(x) => x,
		};
		let future = future.join(acquire_future);

		//println!("HGEMain: SecondaryCmdBuffer");
		TimeStatsStorage::forceNow("R_CrtTex");
		let mut cmdBufTexture = match AutoCommandBufferBuilder::primary(
			HGEMain::singleton().getCmdAllocatorSet(),
			queueGraphic.queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
		)
		{
			Ok(r) => r,
			Err(err) =>
			{
				HTrace!("Cannot crate primary command buffer for texture : {}", err);
				return;
			}
		};
		TimeStatsStorage::update("R_CrtTex");

		TimeStatsStorage::forceNow("R_CmdDrain");
		println!("drain-pre");
		let mut callbackCmdBuffer = Vec::new();
		if let Some(mut entry) = HGEMain::SecondaryCmdBuffer_drain(HGEMain_secondarybuffer_type::TEXTURE)
		{
			for x in entry.0.drain(0..)
			{
				cmdBufTexture.execute_commands(x).unwrap();
			}
			callbackCmdBuffer.append(&mut entry.1);
		}
		println!("drain-post");

		// execute callback of updated cmdBuffer
		let _ = namedThread!(move || {
			for func in callbackCmdBuffer
			{
				func();
			}
		});

		let future = future.then_execute(queueGraphic.clone(), cmdBufTexture.build().unwrap()).unwrap();
		TimeStatsStorage::update("R_CmdDrain");

		TimeStatsStorage::forceNow("R_CrtDraw");
		let mut cmdBuf = match AutoCommandBufferBuilder::primary(
			HGEMain::singleton().getCmdAllocatorSet(),
			queueGraphic.queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
		)
		{
			Ok(r) => r,
			Err(err) =>
			{
				HTrace!("Cannot crate primary command buffer for mesh : {}", err);
				return;
			}
		};
		TimeStatsStorage::update("R_CrtDraw");

		TimeStatsStorage::forceNow("R_AllPass");
		self._Frame.clearBuffer(&mut cmdBuf, image_index);
		HGEsubpass::singleton().ExecAllPass(self._renderpassC.clone(), &mut cmdBuf, &self._Frame, HGEMain::singleton().getCmdAllocatorSet());
		HTraceError!(cmdBuf.end_render_pass(SubpassEndInfo::default()));

		let future = future.then_signal_fence()
		                   .then_execute(queueGraphic.clone(), cmdBuf.build().unwrap())
		                   .unwrap();
		TimeStatsStorage::update("R_AllPass");

		self.dynamic_resolution_try_apply(future, queueGraphic, image_index, swapchain, preSwapFunc);

	}

	fn dynamic_resolution_try_apply<T: GpuFuture + Send + Sync + 'static>(&mut self, future: CommandBufferExecFuture<T>, queueGraphic: Arc<Queue>, image_index: u32, swapchain: Arc<Swapchain>, preSwapFunc: impl Fn())
	{
		if (cfg!(feature = "dynamicresolution"))
		{
			TimeStatsStorage::forceNow("R_DynRes");
			let mut cmdBufDynamicRes = match AutoCommandBufferBuilder::primary(
				HGEMain::singleton().getCmdAllocatorSet(),
				queueGraphic.queue_family_index(),
				CommandBufferUsage::OneTimeSubmit,
			)
			{
				Ok(r) => r,
				Err(err) =>
				{
					HTrace!("Cannot crate primary command buffer dynamic resolution : {}", err);
					return;
				}
			};
			let _ = cmdBufDynamicRes.execute_commands(self.dynamic_resolution(image_index).build().unwrap());

			//println!("HGEMain: fence");
			let future = future
				.then_signal_fence()
				.then_execute(queueGraphic.clone(), cmdBufDynamicRes.build().unwrap())
				.unwrap();
			TimeStatsStorage::update("R_DynRes");
			self.rendering_end(future, queueGraphic, image_index, swapchain, preSwapFunc);
		}
		else
		{
			self.rendering_end(future, queueGraphic, image_index, swapchain, preSwapFunc);
		}
	}

	fn rendering_end<T: GpuFuture + Send + Sync + 'static>(&mut self, future: CommandBufferExecFuture<T>, queueGraphic: Arc<Queue>, image_index: u32, swapchain: Arc<Swapchain>, preSwapFunc: impl Fn())
	{
		// func to execute something just before prsent
		TimeStatsStorage::forceNow("R_preSwapFunc");
		preSwapFunc();
		TimeStatsStorage::update("R_preSwapFunc");

		TimeStatsStorage::forceNow("R_swapchain");

		println!("tata01");
		let future = future
			.then_swapchain_present(queueGraphic.clone(), SwapchainPresentInfo::swapchain_image_index(swapchain, image_index));

		println!("tata02");
		let future = future.then_signal_fence_and_flush();
		println!("tata03");
		TimeStatsStorage::update("R_swapchain");

		/*let Some(fence) = self.fence_result(fence)
		else
		{
			return;
		};
		//(cfg!(target_os = "linux") &&
		if (self._builderDevice.isNvidia)  // multiple platform problem
		{
			TimeStatsStorage::forceNow("R_Nvidiafix");
			let _ = fence.wait(None); // if not present, make cleanup_finished() crash.
			TimeStatsStorage::update("R_Nvidiafix");
		}
		self._previousFrameEnd = Some(fence.boxed_send_sync());*/

		match future.map_err(Validated::unwrap)
		{
			Ok(future) =>
				{
					self._previousFrameEnd = Some(future.boxed_send_sync());
				}
			Err(VulkanError::OutOfDate) =>
				{
					self._recreatSwapChain = true;
					self._previousFrameEnd = Some(sync::now(self._builderDevice.device.clone()).boxed_send_sync());
				}
			Err(e) =>
				{
					panic!("failed to flush future: {e}");
					// previous_frame_end = Some(sync::now(device.clone()).boxed());
				}
		}
	}

	fn fence_result<T>(&mut self, result: Result<T, Validated<VulkanError>>) -> Option<T>
	{
		match result.map_err(Validated::unwrap)
		{
			Ok(fence) =>
			{
				return Some(fence);
			}
			Err(VulkanError::OutOfDate) =>
			{
				self._recreatSwapChain = true;
			}
			Err(e) =>
			{
				HTrace!("Failed to flush future: {:?}", e);
			}
		}

		return None;
	}

	/// applied dynamic resolution system (move last image to swapimage with blit operation, return true if something gone wrong
	fn dynamic_resolution(&self, image_index: u32) -> AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>
	{
		let mut cmdBuffer = AutoCommandBufferBuilder::secondary(
			HGEMain::singleton().getCmdAllocatorSet(),
			self._builderDevice.getQueueGraphic().queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
			CommandBufferInheritanceInfo { ..Default::default() },
		)
		.unwrap();

		let winInfos = HGEMain::singleton().getWindowInfos();

		//for imageswapchain in swapchain.getImages()
		let binding = self._swapChainC.getImages();
		let Some(imageswapchain) = binding.get(image_index as usize)
		else
		{
			return cmdBuffer;
		};

		let tmpfull = self._Frame.getImgFull();
		let _ = cmdBuffer.blit_image(BlitImageInfo {
			src_image_layout: ImageLayout::TransferSrcOptimal,
			dst_image_layout: ImageLayout::TransferDstOptimal,
			regions: [ImageBlit {
				src_subresource: tmpfull.image().subresource_layers(),
				src_offsets: [[0, 0, 0], [winInfos.width, winInfos.height, 1]],
				dst_subresource: imageswapchain.image().subresource_layers(),
				dst_offsets: [[0, 0, 0], [winInfos.raw_width, winInfos.raw_height, 1]],
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
		}
		else
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
}
