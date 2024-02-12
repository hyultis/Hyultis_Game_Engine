extern crate vulkano;

use std::convert::TryFrom;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use anyhow::anyhow;
use arc_swap::{ArcSwap, ArcSwapOption};
use vulkano::{command_buffer::{
	allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
}, sync::{self, GpuFuture}, Validated, Version, VulkanError, VulkanLibrary};
use vulkano::command_buffer::{BlitImageInfo, CommandBufferInheritanceInfo, ImageBlit, PrimaryCommandBufferAbstract, SecondaryAutoCommandBuffer, SubpassEndInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::render_pass::RenderPass;
use vulkano::swapchain::{acquire_next_image, Surface, SurfaceCapabilities, SurfaceTransform, SwapchainPresentInfo};
use winit::{
	event::{Event, WindowEvent},
	window::{Window, WindowBuilder},
};
use dashmap::DashMap;
use Htrace::{HTrace, HTraceError, TSpawner};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use HArcMut::HArcMut;
use Hconfig::HConfigManager::HConfigManager;
use Htrace::Type::Type;
use json::JsonValue;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocatorCreateInfo;
use vulkano::image::ImageLayout;
use vulkano::image::sampler::Filter;
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::PhysicalKey;
use winit::window::Fullscreen;
use crate::Animation::Animation;
use crate::BuilderDevice::BuilderDevice;
use crate::Camera::Camera;
use crate::components::window::{window_infos, window_orientation};
use crate::configs::general::HGEconfig_general;
use crate::configs::HGEconfig::HGEconfig;
use crate::Inputs::Inputs;
use crate::Interface::ManagerInterface::ManagerInterface;
use crate::HGEFrame::HGEFrame;
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;
use crate::HGESwapchain::HGESwapchain;
use crate::HGEsubpass::HGEsubpass;
use crate::Interface::ManagerFont::ManagerFont;
use crate::InterpolateTimer::ManagerInterpolate;
use crate::ManagerAnimation::{AnimationHolder, ManagerAnimation};
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Shaders::ShaderStruct::ShaderStruct;
use crate::Shaders::Shs_2DVertex::HGE_shader_2Dsimple;
use crate::Shaders::Shs_3Dinstance::HGE_shader_3Dinstance;
use crate::Shaders::Shs_3DVertex::HGE_shader_3Dsimple;
use crate::Shaders::Shs_screen::HGE_shader_screen;
use crate::Textures::Manager::ManagerTexture;
//use crate::steam::steam;

const HGE_STRING: &str = "HGE";
const HGE_VERSION: Version = Version {
	major: 0,
	minor: 1,
	patch: 0,
};

#[derive(Eq, PartialEq, Hash)]
pub enum HGEMain_secondarybuffer_type
{
	TEXTURE,
	GRAPHIC
}

pub struct HGEMain
{
	_instance: RwLock<Option<Arc<Instance>>>,
	_surface: RwLock<Option<Arc<Surface>>>,
	_isSuspended: ArcSwap<bool>,
	_builderDevice: RwLock<Option<BuilderDevice>>,
	_appName: RwLock<Option<String>>,
	_appVersion: RwLock<Version>,
	
	_swapChainC: RwLock<Option<HGESwapchain>>,
	_Frame: RwLock<Option<HGEFrame>>,
	_cmdBufferTextures: Arc<DashMap<HGEMain_secondarybuffer_type, (Vec<Arc<SecondaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>, Vec<Arc<dyn Fn() + Send + Sync>>)>>,
	
	// tmp
	_renderpassC: RwLock<Option<Arc<RenderPass>>>,
	_stdAllocSet: ArcSwapOption<StandardDescriptorSetAllocator>,
	_stdAllocCommand: ArcSwapOption<StandardCommandBufferAllocator>,
	
	//cache data
	_windowInfos: RwLock<window_infos>,
	_windowDimentionF32: RwLock<[f32; 4]>,
	_windowHDPI: RwLock<f32>,
	_windowDimentionF32Raw: RwLock<[f32; 2]>,
	_windowOrientation: RwLock<window_orientation>,
	_timeAppStart: Instant,
	_lastFrameDuration: RwLock<Duration>,
	_cameraAnimation: RwLock<Vec<Animation<Camera, [f32; 3]>>>,
	_isGeneratingImg: ArcSwap<bool>,
	
	// loop
	_fps: RwLock<u32>,
	_cameraC: HArcMut<Camera>,
	_inputsC: RwLock<Inputs>,
	_mouseMode: RwLock<bool>,
	_recreatSwapChain: RwLock<bool>,
	_ManagerInterpolate: RwLock<ManagerInterpolate>,
	_previousFrameEnd: RwLock<Option<Box<dyn GpuFuture + Send + Sync>>>
}

static SINGLETON: OnceLock<HGEMain> = OnceLock::new();

impl HGEMain
{
	pub fn singleton() -> &'static HGEMain
	{
		return SINGLETON.get_or_init(|| {
			HGEMain::new()
		});
	}
	
	pub fn engineInitialize(&self, eventloop: &EventLoopWindowTarget<()>, config: HGEconfig_general) -> anyhow::Result<()>
	{
		if(config.defaultShaderLoader.is_none())
		{
			HTrace!((Type::ERROR) "general configuration for loading shader is empty in \"defaultShaderLoader\"");
			return Err(anyhow!("general configuration for loading shader is empty in \"defaultShaderLoader\""));
		}
		HGEconfig::defineGeneral(config);
		
		HTrace!("Engine init ----");
		self.BuildInstance(eventloop)?;
		HTrace!("Engine init end ----");
		
		self.engineSurfaceReload(eventloop)?;
		HTrace!("Engine creation : surface build + stored");
		
		let surface = self._surface.read().clone().unwrap();
		let builderDevice = BuilderDevice::new(self._instance.read().clone().unwrap(), surface.clone());
		*self._builderDevice.write() = Some(builderDevice.clone());
		self.window_resize(None,None);
		
		HTrace!("Engine creation : device build + stored");
		
		*self._swapChainC.write() = Some(HGESwapchain::new(builderDevice.clone(), surface));
		HTrace!("Engine creation : swapchain build + stored");
		HTrace!("Engine creation end ----");
		
		self.engineLoad()?;
		self._isSuspended.swap(Arc::new(false));
		Ok(())
	}
	
	pub fn windowEventLoop(&self, event: &Event<()>)
	{
		match event
		{
			Event::WindowEvent {
				event: WindowEvent::KeyboardInput {
					event: input,
					..
				}, ..
			} => {
				//println!("key input : {:?}",input);
				if let PhysicalKey::Code(key) = input.physical_key
				{
					let mut inputsC = self._inputsC.write();
					inputsC.updateFromKeyboard(key, input.state);
				}
			},
			Event::WindowEvent {
				event: WindowEvent::Resized(winsize), ..
			} => {
				
				println!("window event resize : {:?}",winsize);
				
				let mut width = winsize.width.max(1);
				if(width>7680)
				{
					width = 1;
				}
				let mut height = winsize.height.max(1);
				if(height>4320)
				{
					height = 1;
				}
				
				#[cfg(target_os = "android")]
				{
					HGEMain::singleton().setWindowHDPI((1080.0/height as f32).min(1.0));
				}
				
				self.window_resize(Some(width), Some(height));
			}
			_ => ()
		}
	}
	
	pub fn windowEventLoopGraphic(&self)
	{
		// vsync for wayland ? <== block to 60fps (compositor limit ?) still limit to 144fps without)
		//self._previousFrameEnd.write().unwrap().as_mut().unwrap().cleanup_finished();
		if **self._isSuspended.load()
		{
			return;
		}
		
		{
			if let Some(swapchain) = &*self._swapChainC.read()
			{
				if(swapchain.getFpsLimiter()>0)
				{
					let durationFromLast = self._ManagerInterpolate.read().getNowFromLast();
					if (durationFromLast.as_millis() < 1000 / (swapchain.getFpsLimiter() as u128))
					{
						return;
					}
				}
			}
		}
		
		/*let isGenerating= {*self._isGeneratingImg.load_full()};
		if(isGenerating)
		{
			return;
		}*/
		
		// Whenever the window resizes we need to recreate everything dependent on the window size.
		// In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
		{
			let mut tmp = self._recreatSwapChain.write();
			if *tmp
			{
				if let Some(swapchain) = &mut *self._swapChainC.write()
				{
					swapchain.recreate();
				}
				self.window_size_dependent_setup();
				*tmp = false;
			};
		}
		
		self._isGeneratingImg.swap(Arc::new(true));
		
		//thread::spawn(||{
			HGEMain::singleton().SwapchainGenerateImg();
		//});
		
	}
	
	pub fn getCamera(&self) -> HArcMut<Camera>
	{
		return self._cameraC.clone();
	}
	
	pub fn Camera_addAnim(&self, anim: Animation<Camera, [f32; 3]>)
	{
		self._cameraAnimation.write().push(anim);
	}
	
	pub fn getWindow<F>(&self, func: F)
		where F: FnOnce(&Window)
	{
		let surfaceBinding = self._surface.read().clone().unwrap();
		let tmp = surfaceBinding.object().unwrap().downcast_ref::<Window>().unwrap();
		func(tmp);
	}
	
	pub fn getInputs(&self) -> RwLockReadGuard<'_, Inputs>
	{
		return self._inputsC.read();
	}
	
	pub fn getTimer(&self) -> RwLockReadGuard<'_, ManagerInterpolate>
	{
		return self._ManagerInterpolate.read();
	}
	
	pub fn getTimerMut(&self) -> RwLockWriteGuard<'_, ManagerInterpolate>
	{
		return self._ManagerInterpolate.write();
	}
	
	pub fn getDurationFromStart(&self) -> Duration
	{
		return self._timeAppStart.elapsed();
	}
	
	pub fn setWindowOrientation(&self, surface_orientation: SurfaceTransform)
	{
		let surface_orientation = window_orientation::from(surface_orientation);
		self._windowInfos.write().orientation = surface_orientation;
	}
	
	pub fn getWindowInfos(&self) -> window_infos
	{
		return self._windowInfos.read().clone();
	}
	
	pub fn setWindowHDPI(&self, ratio: f32)
	{
		#[cfg(feature = "dynamicresolution")]
		{
			*self._windowHDPI.write() = ratio.max(0.0);
		}
	}
	
	pub fn getWindowHDPI(&self) -> f32
	{
		return *self._windowHDPI.write();
	}
	
	pub fn getWindowCorrectedMousePos(&self, mousex: &mut f64, mousey: &mut f64)
	{
		let hdpi = self.getWindowHDPI() as f64;
		let tmpx = *mousex*hdpi;
		let tmpy = *mousey*hdpi;
		
		match *self._windowOrientation.read() {
			window_orientation::NORMAL | window_orientation::ROT_180 => {
				*mousex = tmpx;
				*mousey = tmpy;
			}
			window_orientation::ROT_90 | window_orientation::ROT_270 => {
				*mousex = tmpy;
				*mousey = tmpx;
			}
		}
	}
	
	pub fn getAllocatorSet(&self) -> Arc<StandardDescriptorSetAllocator>
	{
		return self._stdAllocSet.load_full().unwrap();
	}
	
	pub fn getAllocatorCommand(&self) -> Arc<StandardCommandBufferAllocator>
	{
		return self._stdAllocCommand.load_full().unwrap();
	}
	
	pub fn getDevice(&self) -> BuilderDevice
	{
		return self._builderDevice.read().clone().unwrap();
	}
	
	pub fn SecondaryCmdBuffer_generate(&self) -> AutoCommandBufferBuilder<SecondaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>
	{
		AutoCommandBufferBuilder::secondary(
			&HGEMain::singleton().getAllocatorCommand(),
			HGEMain::singleton().getDevice().getQueueGraphic().queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
			CommandBufferInheritanceInfo {
				..Default::default()
			},
		).unwrap()
	}
	
	pub fn SecondaryCmdBuffer_add(sbtype: HGEMain_secondarybuffer_type, cmdBuffer: Arc<SecondaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>, callback: impl Fn() + Send + Sync + 'static)
	{
		let _ = TSpawner!(move || {
			if let Some(mut arrayvec) = Self::singleton()._cmdBufferTextures.clone().get_mut(&sbtype)
			{
				arrayvec.0.push(cmdBuffer);
				arrayvec.1.push(Arc::new(callback));
				return;
			}
			
			Self::singleton()._cmdBufferTextures.clone().insert(sbtype, (vec![cmdBuffer], vec![Arc::new(callback)]));
		});
	}
	
	pub fn askForRefresh(&self)
	{
		*self._recreatSwapChain.write() = true;
	}
	
	pub fn engineResumed(&self, eventloop: &EventLoopWindowTarget<()>) -> anyhow::Result<()>
	{
		HTrace!("Engine context creation ----");
		self.engineSurfaceReload(eventloop)?;
		
		*self._swapChainC.write() = Some(HGESwapchain::new(
			self._builderDevice.read().clone().unwrap(),
			self._surface.read().clone().unwrap())
		);
		
		ManagerInterface::singleton().WindowRefreshed();
		self._isSuspended.swap(Arc::new(false));
		Ok(())
	}
	
	pub fn engineSuspended(&self)
	{
		HTrace!("Engine context deleted ----");
		*self._surface.write() = None;
		self._isSuspended.swap(Arc::new(true));
	}
	
	pub fn engineIsSuspended(&self) -> bool
	{
		*self._isSuspended.load_full()
	}
	
	///////////// PRIVATE
	
	fn new() -> HGEMain
	{
		return HGEMain
		{
			_instance: RwLock::new(None),
			_surface: RwLock::new(None),
			_isSuspended: ArcSwap::new(Arc::new(true)),
			_builderDevice: RwLock::new(None),
			_appName: RwLock::new(None),
			_appVersion: RwLock::new(Version {
				major: 0,
				minor: 0,
				patch: 0,
			}),
			_swapChainC: RwLock::new(None),
			_renderpassC: RwLock::new(None),
			_stdAllocSet: ArcSwapOption::new(None),
			_stdAllocCommand: ArcSwapOption::new(None),
			_windowInfos: RwLock::new(window_infos::default()),
			_windowDimentionF32: RwLock::new([1.0, 1.0, 1.0, 1.0]),
			_windowHDPI: RwLock::new(1.0),
			_windowDimentionF32Raw: RwLock::new([1.0, 1.0]),
			_windowOrientation: RwLock::new(Default::default()),
			_timeAppStart: Instant::now(),
			_lastFrameDuration: RwLock::new(Duration::from_nanos(0)),
			_cameraAnimation: RwLock::new(vec![]),
			_isGeneratingImg:  ArcSwap::new(Arc::new(false)),
			_fps: RwLock::new(0),
			_cameraC: HArcMut::new(Camera::new()),
			_inputsC: RwLock::new(Inputs::new()),
			_mouseMode: RwLock::new(true),
			_recreatSwapChain: RwLock::new(true),
			_ManagerInterpolate: RwLock::new(ManagerInterpolate::new()),
			_previousFrameEnd: RwLock::new(None),
			_Frame: RwLock::new(None),
			_cmdBufferTextures: Arc::new(DashMap::new()),
		};
	}
	
	fn BuildInstance(&self, event_loop: &EventLoopWindowTarget<()>) -> anyhow::Result<()>
	{
		let library = VulkanLibrary::new().unwrap();
		let required_extensions = Surface::required_extensions(event_loop);
		let debuglayer = Vec::new();
		
		/*{
			debuglayer.push("VK_LAYER_KHRONOS_validation".to_string());
		}*/
		
		// Now creating the instance.
		let instance = Instance::new(
			library,
			InstanceCreateInfo {
				application_name: self._appName.read().clone(),
				application_version: *self._appVersion.read(),
				engine_name: Some(HGE_STRING.to_string()),
				engine_version: HGE_VERSION,
				enabled_layers: debuglayer,
				enabled_extensions: required_extensions,
				// Enable enumerating devices that use non-conformant vulkan implementations. (ex. MoltenVK)
				flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
				..Default::default()
			},
		)?;
		
		
		*self._instance.write() = Some(instance);
		Ok(())
	}
	
	fn window_size_dependent_setup(&self)
	{
		let images = {
			self._swapChainC.read().clone().unwrap().getImages()
		};
		
		let renderpass = self._renderpassC.read().clone().unwrap();
		let mut frameBinding = self._Frame.write();
		if let Some(frame) = frameBinding.as_mut()
		{
			frame.add(images, renderpass.clone());
		};
		
		ManagerPipeline::singleton().pipelineRefresh(renderpass.clone());
	}
	
	fn window_resize(&self, parentwidth: Option<u32>, parentheight: Option<u32>)
	{
		let Some(surfaceCap) = self.getSurfaceCapability() else {
			return;
		};
		
		HTrace!("viewport parent : [{:?},{:?}]", parentwidth, parentheight);
		let rawwidth ;
		let rawheight ;
		if(parentwidth.is_none() || parentheight.is_none())
		{
			let extends = surfaceCap.current_extent.unwrap_or([1, 1]);
			rawwidth = extends[0];
			rawheight = extends[1];
		}
		else
		{
			rawwidth = parentwidth.unwrap_or(1);
			rawheight = parentheight.unwrap_or(1);
		}
		
		let hdpi = *self._windowHDPI.read();
		
		HTrace!("viewport dim conv : [{},{}]", rawwidth, rawheight);
		HTrace!("viewport dim hdpi : {}", hdpi);
		HTrace!("viewport dim iswide : {}", rawwidth>rawheight);
		
		let widthF = (rawwidth as f32 * hdpi) as f32;
		let heightF = (rawheight as f32 * hdpi) as f32;
		
		*self._windowInfos.write() = window_infos{
			originx: 0.0,
			originy: 0.0,
			width: widthF as u32,
			height: heightF as u32,
			widthF,
			heightF,
			raw_width: rawwidth,
			raw_height: rawheight,
			raw_widthF: rawwidth as f32,
			raw_heightF: rawheight as f32,
			ratio_w2h: widthF/heightF,
			ratio_h2w: heightF/widthF,
			orientation: window_orientation::from(surfaceCap.current_transform),
			isWide: rawwidth>rawheight,
		};
		
		*self._recreatSwapChain.write() = true;
	}
	
	/// applied dynamic resolution system (move last image to swapimage with blit operation, return true if something gone wrong
	fn dynamic_resolution(&self, image_index: u32) -> AutoCommandBufferBuilder<SecondaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>
	{
		let mut cmdBuffer = AutoCommandBufferBuilder::secondary(
			&HGEMain::singleton().getAllocatorCommand(),
			HGEMain::singleton().getDevice().getQueueGraphic().queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
			CommandBufferInheritanceInfo {
				..Default::default()
			},
		).unwrap();
		
		if let Some(swapchain) = &*self._swapChainC.read()
		{
			if let Some(frame) = &*self._Frame.read()
			{
				let winInfos = self.getWindowInfos();
				
				//for imageswapchain in swapchain.getImages()
				if let Some(imageswapchain) = swapchain.getImages().get(image_index as usize)
				{
					let tmpfull = frame.getImgFull();
					let result = cmdBuffer
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
					
					if (result.is_err())
					{
						return cmdBuffer;
					}
				}
			}
		}
		return cmdBuffer;
	}
	
	fn engineSurfaceReload(&self, eventloop: &EventLoopWindowTarget<()>) -> anyhow::Result<()>
	{
		let instance = self._instance.read().clone().unwrap();
		let configBind = HGEconfig::singleton().general_get();
		
		let mut defaultwindowtype = 2;// 1 or 2 = fullscreen
		if(!HGEconfig::singleton().general_get().startFullscreen)
		{
			defaultwindowtype = 0;
		}
		
		let mut config = HConfigManager::singleton().get("config");
		let mut windowtype = config.getOrSetDefault("window/type",JsonValue::from(defaultwindowtype)).as_u32().unwrap_or(2);
		let mut fullscreenmode = None;
		if(windowtype==1 && eventloop.primary_monitor().is_none())
		{
			windowtype=2;
		}
		if(configBind.isSteamdeck || configBind.isAndroid) // config ignored for steam deck and android
		{
			windowtype = 1;
			config.set("window/type",JsonValue::from(windowtype));
		}
		
		if(windowtype==1)
		{
			let mut video_mode = eventloop.primary_monitor().unwrap().video_modes().collect::<Vec<_>>();
			HTrace!("video modes : {:?}",video_mode);
			video_mode.sort_by(|a, b| {
				use std::cmp::Ordering::*;
				match b.size().width.cmp(&a.size().width) {
					Equal => match b.size().height.cmp(&a.size().height) {
						Equal => b
							.refresh_rate_millihertz()
							.cmp(&a.refresh_rate_millihertz()),
						default => default,
					},
					default => default,
				}
			});
			fullscreenmode = Some(Fullscreen::Exclusive(video_mode.first().unwrap().clone()));
		}
		if(windowtype==2)
		{
			fullscreenmode = Some(Fullscreen::Borderless(None));
		}
		
		let window = WindowBuilder::new()
			//.with_min_inner_size(LogicalSize{ width: 640, height: 480 })
			//.with_name("Truc much", "yolo")
			.with_title(&configBind.windowTitle)
			.with_fullscreen(fullscreenmode)
			.build(eventloop)?;
		
		
		let surface = Surface::from_window(instance,Arc::new(window)).map_err(|e|{anyhow!(e)})?;
		*self._surface.write() = Some(surface.clone());
		self.window_resize(None,None);
		
		let _ = config.save();
		Ok(())
	}
	
	fn engineLoad(&self) -> anyhow::Result<()>
	{
		HTrace!("Engine load internal ----");
		if (self._instance.read().is_none())
		{
			return Err(anyhow!("engineInitialize function must be called first"));
		}
		
		let builderDevice = self._builderDevice.read().clone().unwrap();
		
		{
			ManagerMemoryAllocator::singleton().add(builderDevice.device.clone());
			self._stdAllocCommand.swap(Some(Arc::new(StandardCommandBufferAllocator::new(builderDevice.device.clone(), StandardCommandBufferAllocatorCreateInfo {
				primary_buffer_count: 8,
				secondary_buffer_count: 8,
				..Default::default()
			} ))));
			self._stdAllocSet.swap(Some(Arc::new(StandardDescriptorSetAllocator::new(builderDevice.device.clone(), Default::default() ))));
		}
		
		
		{
			self._cameraC.get_mut().setPositionXYZ(1.0, 1.0, 100.0);
		}
		
		
		/*let vs = {
			let mut f = File::open("./static/shaders/vert3D.glsl")
				.expect("./static/shaders/vert3D.glsl This example needs to be run from the root of the example crate.");
			let mut v = vec![];
			f.read_to_end(&mut v).unwrap();
			// Create a ShaderModule on a device the same Shader::load does it.
			// NOTE: You will have to verify correctness of the data by yourself!
			unsafe { ShaderModule::from_bytes(device.clone(), &v) }.unwrap()
		};*/ // https://github.com/vulkano-rs/vulkano/commit/fe01ddd5e3f178b971ed102dd5fdd93cee5d87b9#diff-d246486211c651344a5f0381a9258a41abeea7678bb10c3e4c855372f8b9b8e4
		
		{
			let loadingdExternalShader = HGEconfig::singleton().general_get().defaultShaderLoader.clone().unwrap();
			loadingdExternalShader();
			println!("instance");
			HGE_shader_3Dinstance::createPipeline()?;
			println!("3d");
			HGE_shader_3Dsimple::createPipeline()?;
			println!("2d");
			HGE_shader_2Dsimple::createPipeline()?;
			println!("screen");
			HGE_shader_screen::createPipeline()?;
		}
		
		// At this point, OpenGL initialization would be finished. However in Vulkan it is not. OpenGL
		// implicitly does a lot of computation whenever you draw. In Vulkan, you have to do all this
		// manually.
		
		// The next step is to create a *render pass*, which is an object that describes where the
		// output of the graphics pipeline will go. It describes the layout of the images
		// where the colors, depth and/or stencil information will be written.
		{
			let depthformat = HGEMain::singleton().getDevice().depthformat;
			let imageformat = self._swapChainC.read().clone().unwrap().getImageFormat();
			#[cfg(feature = "dynamicresolution")]
				let render_pass = vulkano::ordered_passes_renderpass!(
				builderDevice.device.clone(),
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
			
			#[cfg(not(feature = "dynamicresolution"))]
				let render_pass = vulkano::ordered_passes_renderpass!(
				builderDevice.device.clone(),
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
						format: Format::D32_SFLOAT,
						samples: 1,
						load_op: Clear,
						store_op: DontCare,
					},
					depthSolid: {
						format: Format::D32_SFLOAT,
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
					color: [render_Final],
					depth_stencil: {},
					input:[render_UI,render_WorldSolid]
				}]
			)?;
			
			ManagerPipeline::singleton().pipelineRefresh(render_pass.clone());
			*self._renderpassC.write() = Some(render_pass);
		}
		
		
		{
			*self._Frame.write() = Some(HGEFrame::new(self._swapChainC.read().clone().unwrap().getImageFormat()));
		}
		
		self.window_size_dependent_setup();
		ManagerTexture::singleton().preload();
		
		let primaryCommandBuffer = AutoCommandBufferBuilder::primary(
			&self._stdAllocCommand.load_full().unwrap(),
			builderDevice.getQueueGraphic().queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
		)
			.unwrap();
		*self._previousFrameEnd.write() = Some(
			primaryCommandBuffer.build()
				.unwrap()
				.execute(builderDevice.getQueueGraphic())
				.unwrap()
				.boxed_send_sync()
		);
		
		let lang = "world";
		//ManagerTranslate::get("font");
		HTraceError!(ManagerFont::singleton().FontLoad(lang));
		
		HTrace!("Engine load internal end ----");
		Ok(())
	}
	
	fn SwapchainGenerateImg(&self)
	{
		if let Some(previous) = &mut *self._previousFrameEnd.write()
		{
			previous.cleanup_finished();
		}
		
		let builderDevice = self._builderDevice.read().clone().unwrap();
		if let Some(swapchain) = self._swapChainC.read().clone()
		{
			self._ManagerInterpolate.write().update();
			
			// Before we can draw on the output, we have to *acquire* an image from the swapchain. If
			// no image is available (which happens if you submit draw commands too quickly), then the
			// function will block.
			// This operation returns the index of the image that we are allowed to draw upon.
			//
			// This function can block if no image is available. The parameter is an optional timeout
			// after which the function call will return an error.
			let (image_index, suboptimal, acquire_future) =
				match acquire_next_image(swapchain.get(), None).map_err(Validated::unwrap) {
					Ok(r) => r,
					Err(VulkanError::OutOfDate) => {
						*self._recreatSwapChain.write() = true;
						//self._isGeneratingImg.swap(Arc::new(false));
						return;
					}
					Err(e) => {
						*self._recreatSwapChain.write() = true;
						HTrace!((Type::WARNING) "acquire_next_image {}", e);
						//self._isGeneratingImg.swap(Arc::new(false));
						return;
					},
				};
			
			// acquire_next_image can be successful, but suboptimal. This means that the swapchain image
			// will still work, but it may not display correctly. With some drivers this can be when
			// the window resizes, but it may not cause the swapchain to become out of date.
			if suboptimal {
				*self._recreatSwapChain.write() = true;
			}
			
			
			//println!("HGEMain: ManagerFont");
			ManagerFont::singleton().FontEngine_CacheUpdate();
			//println!("HGEMain: ManagerAnimation");
			ManagerAnimation::singleton().ticksAll();
			//println!("HGEMain: ManagerTexture");
			ManagerTexture::singleton().launchThreads();
			
			self._cameraAnimation.write().retain_mut(|anim| {
				//println!("one cam anim");
				!anim.ticks()
			});
			
			//println!("HGEMain: SecondaryCmdBuffer");
			let mut cmdBufTexture = AutoCommandBufferBuilder::primary(
				&self._stdAllocCommand.load_full().unwrap(),
				builderDevice.getQueueGraphic().queue_family_index(),
				CommandBufferUsage::OneTimeSubmit,
			)
				.unwrap();
			
			match self._cmdBufferTextures.remove(&HGEMain_secondarybuffer_type::TEXTURE) {
				None => {}
				Some((_, entry)) => {
					for x in entry.0 {
						cmdBufTexture.execute_commands(x).unwrap();
					}
					for x in entry.1 {
						x();
					}
				}
			}
			
			//println!("HGEMain: stdAllocCommand");
			let mut cmdBuf = AutoCommandBufferBuilder::primary(
				&self._stdAllocCommand.load_full().unwrap(),
				builderDevice.getQueueGraphic().queue_family_index(),
				CommandBufferUsage::OneTimeSubmit,
			)
				.unwrap();
			
			//println!("HGEMain: begin_render_pass");
			
			//println!("HGEMain: HGEsubpass");
			if let Some(frame) = &*self._Frame.read()
			{
				frame.clearBuffer(&mut cmdBuf, image_index);
				
				HGEsubpass::singleton().ExecAllPass(self._renderpassC.read().clone().unwrap(), &mut cmdBuf, frame);
				
				HTraceError!(cmdBuf.end_render_pass(SubpassEndInfo::default()));
			}
			
			
			let mut previousFrameEndBinding = self._previousFrameEnd.write();
			//println!("HGEMain: future");
			let future = previousFrameEndBinding
				.take()
				.unwrap()
				.join(acquire_future)
				.then_execute(builderDevice.getQueueGraphic().clone(), cmdBufTexture.build().unwrap()).unwrap()
				.then_execute(builderDevice.getQueueGraphic().clone(), cmdBuf.build().unwrap()).unwrap();
			
			
			let mut cmdBuf = AutoCommandBufferBuilder::primary(
				&self._stdAllocCommand.load_full().unwrap(),
				builderDevice.getQueueGraphic().queue_family_index(),
				CommandBufferUsage::OneTimeSubmit,
			)
				.unwrap();
				
			#[cfg(feature = "dynamicresolution")]
			{
				let _ = cmdBuf.execute_commands(self.dynamic_resolution(image_index).build().unwrap());
				
				//println!("dynamicresolution : {}",lastinstant.elapsed().as_nanos());
			}
			
			let future = future.then_execute(builderDevice.getQueueGraphic().clone(), cmdBuf.build().unwrap()).unwrap();
			let futured = future
				.then_swapchain_present(
					builderDevice.getQueueGraphic().clone(),
					SwapchainPresentInfo::swapchain_image_index(self._swapChainC.read().clone().unwrap().get(), image_index),
				)
				.then_signal_fence();
			
			//println!("after future : {}", debugstart.elapsed().as_nanos());
			//.wait(None)
			
			match futured.wait(None).map_err(Validated::unwrap) { // Some(Duration::from_millis(16))
				Ok(_) => {
					*previousFrameEndBinding = Some(futured.boxed_send_sync());
				}
				Err(VulkanError::OutOfDate) => {
					*self._recreatSwapChain.write() = true;
					*previousFrameEndBinding = Some(sync::now(builderDevice.device.clone()).boxed_send_sync());
				}
				Err(e) => {
					HTrace!("Failed to flush future: {:?}", e);
					*previousFrameEndBinding = Some(sync::now(builderDevice.device.clone()).boxed_send_sync());
				}
			}
			
			//self._isGeneratingImg.swap(Arc::new(false));
		}
	}
	
	fn getSurfaceCapability(&self) -> Option<SurfaceCapabilities>
	{
		if let Some(builderDevice) = &*self._builderDevice.read()
		{
			if let Some(surface) = &*self._surface.read()
			{
				if let Ok(result) = builderDevice.device
				.physical_device()
				.surface_capabilities(surface, Default::default())
				{
					return Some(result);
				}
			}
		}
		return None;
	}
}
