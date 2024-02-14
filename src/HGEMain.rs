extern crate vulkano;

use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use anyhow::anyhow;
use arc_swap::{ArcSwap, ArcSwapOption};
use vulkano::{command_buffer::{
	AutoCommandBufferBuilder, CommandBufferUsage,
}, Version, VulkanLibrary};
use vulkano::command_buffer::{CommandBufferInheritanceInfo, SecondaryAutoCommandBuffer};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::swapchain::{Surface, SurfaceCapabilities, SurfaceTransform};
use winit::{
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
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Fullscreen;
use crate::Animation::Animation;
use crate::BuilderDevice::BuilderDevice;
use crate::Camera::Camera;
use crate::components::window::{window_infos, window_orientation};
use crate::configs::general::HGEconfig_general;
use crate::configs::HGEconfig::HGEconfig;
use crate::Interface::ManagerInterface::ManagerInterface;
use crate::HGErendering::HGErendering;
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;
use crate::Interface::ManagerFont::ManagerFont;
use crate::InterpolateTimer::ManagerInterpolate;
use crate::ManagerAnimation::{AnimationHolder, ManagerAnimation};
use crate::Shaders::ShaderStruct::ShaderStruct;
use crate::Shaders::HGE_shader_2Dsimple::HGE_shader_2Dsimple;
use crate::Shaders::HGE_shader_3Dinstance::HGE_shader_3Dinstance;
use crate::Shaders::HGE_shader_3Dsimple::HGE_shader_3Dsimple;
use crate::Shaders::HGE_shader_screen::HGE_shader_screen;
use crate::Textures::Manager::ManagerTexture;

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
	// instance stuff
	_instance: RwLock<Option<Arc<Instance>>>,
	_surface: RwLock<Option<Arc<Surface>>>,
	_isSuspended: ArcSwap<bool>,
	_builderDevice: RwLock<Option<BuilderDevice>>,
	_rendering: RwLock<Option<HGErendering>>,
	
	
	// app data
	_appName: RwLock<Option<String>>,
	_appVersion: RwLock<Version>,
	
	// tmp
	_stdAllocSet: ArcSwapOption<StandardDescriptorSetAllocator>,
	_cmdBufferTextures: Arc<DashMap<HGEMain_secondarybuffer_type, (Vec<Arc<SecondaryAutoCommandBuffer>>, Vec<Arc<dyn Fn() + Send + Sync>>)>>,
	
	//cache data
	_windowInfos: RwLock<window_infos>,
	_windowDimentionF32: RwLock<[f32; 4]>,
	_windowHDPI: RwLock<f32>,
	_windowDimentionF32Raw: RwLock<[f32; 2]>,
	_windowOrientation: RwLock<window_orientation>,
	_timeAppStart: Instant,
	_lastFrameDuration: RwLock<Duration>,
	_cameraAnimation: RwLock<Vec<Animation<Camera, [f32; 3]>>>,
	
	// loop
	_fps: RwLock<u32>,
	_cameraC: HArcMut<Camera>,
	_mouseMode: RwLock<bool>,
	_ManagerInterpolate: RwLock<ManagerInterpolate>,
}

static SINGLETON: OnceLock<HGEMain> = OnceLock::new();

impl HGEMain
{
	pub fn singleton() -> &'static Self
	{
		return SINGLETON.get_or_init(|| {
			Self::new()
		});
	}
	
	pub fn engineInitialize(&self, eventloop: &EventLoopWindowTarget<()>, config: HGEconfig_general) -> anyhow::Result<()>
	{
		if (config.defaultShaderLoader.is_none())
		{
			HTrace!((Type::ERROR) "general configuration for loading shader is empty in \"defaultShaderLoader\"");
			return Err(anyhow!("general configuration for loading shader is empty in \"defaultShaderLoader\""));
		}
		HGEconfig::defineGeneral(config);
		
		HTrace!("Engine init ----");
		self.BuildInstance(eventloop)?;
		
		HTrace!("Engine creation : surface build");
		self.engineSurfaceReload(eventloop)?;
		HTrace!("Engine creation : surface stored");
		
		HTrace!("Engine creation : device build");
		let surface = self._surface.read().clone().unwrap();
		let builderDevice = BuilderDevice::new(self._instance.read().clone().unwrap(), surface.clone());
		*self._builderDevice.write() = Some(builderDevice.clone());
		self.window_resize(None, None);
		HTrace!("Engine creation : device stored");
		
		HTrace!("Engine creation : Memory allocator build");
		ManagerMemoryAllocator::singleton().update(builderDevice.device.clone());
		self._stdAllocSet.swap(Some(Arc::new(StandardDescriptorSetAllocator::new(builderDevice.device.clone(), Default::default()))));
		HTrace!("Engine creation : Memory allocator stored");
		
		HTrace!("Engine creation : rendering build");
		*self._rendering.write() = Some(HGErendering::new(builderDevice, surface)?);
		HTrace!("Engine creation : rendering stored");
		HTrace!("Engine creation end ----");
		
		self.engineLoad()?;
		self._isSuspended.swap(Arc::new(false));
		Ok(())
	}
	
	pub fn runService(&self)
	{
		if **self._isSuspended.load()
		{
			return;
		}
		
		self._ManagerInterpolate.write().update();
		ManagerFont::singleton().FontEngine_CacheUpdate();
		ManagerAnimation::singleton().ticksAll();
		ManagerTexture::singleton().launchThreads();
		
		self._cameraAnimation.write().retain_mut(|anim| {
			//println!("one cam anim");
			!anim.ticks()
		});
	}
	
	pub fn runRendering(&self)
	{
		if **self._isSuspended.load()
		{
			return;
		}
		
		//let _ = TSpawner!(||{
		if let Some(rendering) = &mut *self._rendering.write()
		{
			let durationFromLast = self._ManagerInterpolate.read().getNowFromLast();
			rendering.rendering(durationFromLast);
		}
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
		let tmpx = *mousex * hdpi;
		let tmpy = *mousey * hdpi;
		
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
	
	pub fn getDevice(&self) -> BuilderDevice
	{
		return self._builderDevice.read().clone().unwrap();
	}
	
	pub fn SecondaryCmdBuffer_generate(&self) -> AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>
	{
		let cmd = &*self._rendering.read();
		AutoCommandBufferBuilder::secondary(
			cmd.as_ref().unwrap().getAllocCmd(),
			Self::singleton().getDevice().getQueueGraphic().queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
			CommandBufferInheritanceInfo {
				..Default::default()
			},
		).unwrap()
	}
	
	pub fn SecondaryCmdBuffer_add(sbtype: HGEMain_secondarybuffer_type, cmdBuffer: Arc<SecondaryAutoCommandBuffer>, callback: impl Fn() + Send + Sync + 'static)
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
	
	pub(crate) fn SecondaryCmdBuffer_drain(typed: HGEMain_secondarybuffer_type) -> Option<(Vec<Arc<SecondaryAutoCommandBuffer>>, Vec<Arc<dyn Fn() + Send + Sync>>)>
	{
		return Self::singleton()._cmdBufferTextures.clone().remove(&typed).map(|(_,x)|x);
	}
	
	pub fn engineResumed(&self, eventloop: &EventLoopWindowTarget<()>) -> anyhow::Result<()>
	{
		HTrace!("Engine context creation ----");
		self.engineSurfaceReload(eventloop)?;
		
		if let Some(rendering) = &mut *self._rendering.write()
		{
			rendering.recreate(self._builderDevice.read().clone().unwrap(),self._surface.read().clone().unwrap());
		}
		
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
	
	fn new() -> Self
	{
		return Self
		{
			_instance: RwLock::new(None),
			_surface: RwLock::new(None),
			_isSuspended: ArcSwap::new(Arc::new(true)),
			_builderDevice: RwLock::new(None),
			_rendering: RwLock::new(None),
			_appName: RwLock::new(None),
			_appVersion: RwLock::new(Version {
				major: 0,
				minor: 0,
				patch: 0,
			}),
			_stdAllocSet: ArcSwapOption::new(None),
			_windowInfos: RwLock::new(window_infos::default()),
			_windowDimentionF32: RwLock::new([1.0, 1.0, 1.0, 1.0]),
			_windowHDPI: RwLock::new(1.0),
			_windowDimentionF32Raw: RwLock::new([1.0, 1.0]),
			_windowOrientation: RwLock::new(Default::default()),
			_timeAppStart: Instant::now(),
			_lastFrameDuration: RwLock::new(Duration::from_nanos(0)),
			_cameraAnimation: RwLock::new(vec![]),
			_fps: RwLock::new(0),
			_cameraC: HArcMut::new(Camera::new()),
			_mouseMode: RwLock::new(true),
			_ManagerInterpolate: RwLock::new(ManagerInterpolate::new()),
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
	
	
	pub fn window_resize(&self, parentwidth: Option<u32>, parentheight: Option<u32>)
	{
		let Some(surfaceCap) = self.getSurfaceCapability() else {
			return;
		};
		
		HTrace!("viewport parent : [{:?},{:?}]", parentwidth, parentheight);
		let rawwidth;
		let rawheight;
		if (parentwidth.is_none() || parentheight.is_none())
		{
			let extends = surfaceCap.current_extent.unwrap_or([1, 1]);
			rawwidth = extends[0];
			rawheight = extends[1];
		} else {
			rawwidth = parentwidth.unwrap_or(1);
			rawheight = parentheight.unwrap_or(1);
		}
		
		let hdpi = *self._windowHDPI.read();
		
		HTrace!("viewport dim conv : [{},{}]", rawwidth, rawheight);
		HTrace!("viewport dim hdpi : {}", hdpi);
		HTrace!("viewport dim iswide : {}", rawwidth>rawheight);
		
		let widthF = (rawwidth as f32 * hdpi) as f32;
		let heightF = (rawheight as f32 * hdpi) as f32;
		
		*self._windowInfos.write() = window_infos {
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
			ratio_w2h: widthF / heightF,
			ratio_h2w: heightF / widthF,
			orientation: window_orientation::from(surfaceCap.current_transform),
			isWide: rawwidth > rawheight,
		};
		
		if let Some(rendering) = &mut *self._rendering.write()
		{
			rendering.forceSwapchainRecreate();
		}
	}
	
	fn engineSurfaceReload(&self, eventloop: &EventLoopWindowTarget<()>) -> anyhow::Result<()>
	{
		let instance = self._instance.read().clone().unwrap();
		let configBind = HGEconfig::singleton().general_get();
		
		let mut defaultwindowtype = 2;// 1 or 2 = fullscreen
		if (!HGEconfig::singleton().general_get().startFullscreen)
		{
			defaultwindowtype = 0;
		}
		
		let mut config = HConfigManager::singleton().get("config");
		let mut windowtype = config.getOrSetDefault("window/type", JsonValue::from(defaultwindowtype)).as_u32().unwrap_or(2);
		let mut fullscreenmode = None;
		if (windowtype == 1 && eventloop.primary_monitor().is_none())
		{
			windowtype = 2;
		}
		if (configBind.isSteamdeck || configBind.isAndroid) // config ignored for steam deck and android
		{
			windowtype = 1;
			config.set("window/type", JsonValue::from(windowtype));
		}
		
		if (windowtype == 1)
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
		if (windowtype == 2)
		{
			fullscreenmode = Some(Fullscreen::Borderless(None));
		}
		
		let window = WindowBuilder::new()
			//.with_min_inner_size(LogicalSize{ width: 640, height: 480 })
			//.with_name("Truc much", "yolo")
			.with_title(&configBind.windowTitle)
			.with_fullscreen(fullscreenmode)
			.build(eventloop)?;
		
		
		let surface = Surface::from_window(instance, Arc::new(window)).map_err(|e| { anyhow!(e) })?;
		*self._surface.write() = Some(surface.clone());
		self.window_resize(None, None);
		
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
		
		self._cameraC.get_mut().setPositionXYZ(1.0, 1.0, 100.0);
		
		/*let vs = {
			let mut f = File::open("./static/shaders/vert3D.glsl")
				.expect("./static/shaders/vert3D.glsl This example needs to be run from the root of the example crate.");
			let mut v = vec![];
			f.read_to_end(&mut v).unwrap();
			// Create a ShaderModule on a device the same Shader::load does it.
			// NOTE: You will have to verify correctness of the data by yourself!
			unsafe { ShaderModule::from_bytes(device.clone(), &v) }.unwrap()
		};*/ // https://github.com/vulkano-rs/vulkano/commit/fe01ddd5e3f178b971ed102dd5fdd93cee5d87b9#diff-d246486211c651344a5f0381a9258a41abeea7678bb10c3e4c855372f8b9b8e4
		
		
		let loadingdExternalShader = HGEconfig::singleton().general_get().defaultShaderLoader.clone().unwrap();
		loadingdExternalShader();
		HGE_shader_3Dinstance::createPipeline()?;
		HGE_shader_3Dsimple::createPipeline()?;
		HGE_shader_2Dsimple::createPipeline()?;
		HGE_shader_screen::createPipeline()?;
		
		if let Some(rendering) = &mut *self._rendering.write()
		{
			rendering.window_size_dependent_setup();
		}
		ManagerTexture::singleton().preload();
		
		let lang = "world";
		//ManagerTranslate::get("font");
		HTraceError!(ManagerFont::singleton().FontLoad(lang));
		
		HTrace!("Engine load internal end ----");
		Ok(())
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
