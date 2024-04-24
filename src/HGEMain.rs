extern crate vulkano;

use std::any::Any;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use anyhow::anyhow;
use arc_swap::{ArcSwap, ArcSwapOption, Guard};
use vulkano::{command_buffer::{
	AutoCommandBufferBuilder, CommandBufferUsage,
}, Version, VulkanLibrary};
use vulkano::command_buffer::{CommandBufferInheritanceInfo, SecondaryAutoCommandBuffer};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::swapchain::{Surface, SurfaceCapabilities, SurfaceTransform};
use dashmap::DashMap;
use Htrace::{HTrace, HTraceError, TSpawner};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions};
use HArcMut::HArcMut;
use Htrace::Type::Type;
use parking_lot::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use singletonThread::SingletonThread;
use crate::Animation::Animation;
use crate::BuilderDevice::BuilderDevice;
use crate::Camera::Camera;
use crate::components::window::{window_infos, window_orientation};
use crate::configs::general::HGEconfig_general;
use crate::configs::HGEconfig::HGEconfig;
use crate::Interface::ManagerInterface::ManagerInterface;
use crate::HGErendering::HGErendering;
use crate::HGEsubpass::HGEsubpassName;
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;
use crate::Interface::ManagerFont::ManagerFont;
use crate::InterpolateTimer::ManagerInterpolate;
use crate::ManagerAnimation::{AnimationHolder, ManagerAnimation};
use crate::Models3D::ManagerModels::ManagerModels;
use crate::Shaders::ShaderStruct::ShaderStruct;
use crate::Shaders::HGE_shader_2Dsimple::{HGE_shader_2Dline_holder, HGE_shader_2Dsimple, HGE_shader_2Dsimple_holder};
use crate::Shaders::HGE_shader_3Dinstance::{HGE_shader_3Dinstance, HGE_shader_3Dinstance_holder};
use crate::Shaders::HGE_shader_3Dsimple::{HGE_shader_3Dsimple, HGE_shader_3Dsimple_holder};
use crate::Shaders::HGE_shader_screen::HGE_shader_screen;
use crate::Shaders::ShaderDrawer::ShaderDrawer_Manager;
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
	_instance: ArcSwap<Instance>,
	_surface: ArcSwapOption<Surface>,
	_isSuspended: ArcSwap<bool>,
	_builderDevice: ArcSwap<BuilderDevice>,
	_rendering: Arc<RwLock<HGErendering>>,
	
	
	// app data
	_appName: ArcSwap<String>,
	_appVersion: ArcSwap<Version>,
	
	// tmp
	_stdAllocSet: ArcSwap<StandardDescriptorSetAllocator>,
	_cmdBufferTextures: Arc<DashMap<HGEMain_secondarybuffer_type, (Vec<Arc<SecondaryAutoCommandBuffer>>, Vec<Arc<dyn Fn() + Send + Sync>>)>>,
	
	//cache data
	_windowInfos: RwLock<window_infos>,
	_timeAppStart: Instant,
	_lastFrameDuration: RwLock<Duration>,
	_cameraAnimation: RwLock<Vec<Animation<Camera, [f32; 3]>>>,
	
	// loop
	_cameraC: HArcMut<Camera>,
	_mouseMode: RwLock<bool>,
	_ManagerInterpolate: RwLock<ManagerInterpolate>,
	
	// threads
	_thread_runService: Mutex<SingletonThread>,
}

static SINGLETON: OnceLock<HGEMain> = OnceLock::new();
pub struct preinit{
	_init: bool
}

impl HGEMain
{
	pub fn singleton() -> &'static Self
	{
		return SINGLETON.get().unwrap_or_else(|| { panic!("HGE have not been initialized") });
	}
	
	pub fn preinitialize(config: HGEconfig_general) -> anyhow::Result<preinit>
	{
		if (SINGLETON.get().is_some())
		{
			return Err(anyhow!("HGE already initialized"));
		}
		
		if (config.defaultShaderLoader.is_none())
		{
			HTrace!((Type::ERROR) "general configuration for loading shader is empty in \"defaultShaderLoader\"");
			return Err(anyhow!("general configuration for loading shader is empty in \"defaultShaderLoader\""));
		}
		HGEconfig::defineGeneral(config);
		
		Ok(preinit{ _init: true })
	}
	
	pub fn initialize(required_extensions: InstanceExtensions,rawWindow: impl HasRawWindowHandle + HasRawDisplayHandle + Any + Send + Sync, preinit: anyhow::Result<preinit>) -> anyhow::Result<()>
	{
		match preinit {
			Ok(_) => {}
			Err(err) => {return Err(anyhow!("{}",err));}
		};
		
		HTrace!("Engine initialization ----");
		let instance = Self::Init_Instance(required_extensions)?;
		
		HTrace!("Engine initialization : surface build");
		let surface = Self::Init_SurfaceReload(rawWindow,instance.clone())?;
		
		HTrace!("Engine initialization : device build");
		let builderDevice = Arc::new(BuilderDevice::new(instance.clone(), surface.clone()));
		
		HTrace!("Engine initialization : Memory allocator build");
		ManagerMemoryAllocator::singleton().update(builderDevice.device.clone());
		let stdAllocSet = Arc::new(StandardDescriptorSetAllocator::new(builderDevice.device.clone(), Default::default()));
		
		HTrace!("Engine initialization : rendering build");
		let rendering = HGErendering::new(builderDevice.clone(), surface.clone())?;
		
		HTrace!("Engine initialization : HGE creation");
		let selfnew = Self::new(instance,builderDevice,surface,stdAllocSet,rendering);
		if SINGLETON.set(selfnew).is_err()
		{
			return Err(anyhow!("HGE instance set by another thread"));
		}
		
		let selfnew = SINGLETON.get().unwrap();
		selfnew.window_InfosUpdate(None);
		selfnew.engineLoad()?;
		selfnew._isSuspended.swap(Arc::new(false));
		
		HTrace!("Engine initialization end ----");
		Ok(())
	}
	
	pub fn runService(&self)
	{
		Self::singleton()._thread_runService.lock().thread_launch();
	}
	
	pub fn runRendering(&self)
	{
		if(**Self::singleton()._isSuspended.load())
		{
			return;
		}
		
		let durationFromLast = Self::singleton()._ManagerInterpolate.read().getNowFromLast();
		if(self._rendering.write().rendering(durationFromLast))
		{
			Self::singleton()._ManagerInterpolate.write().update();
		}
		
	}
	
	pub fn getCamera(&self) -> HArcMut<Camera>
	{
		return self._cameraC.clone();
	}
	
	pub fn Camera_addAnim(&self, anim: Animation<Camera, [f32; 3]>)
	{
		self._cameraAnimation.write().push(anim);
	}
	
	pub fn getSurface(&self) -> Guard<Option<Arc<Surface>>>
	{
		return self._surface.load();
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
			self._windowInfos.write().HDPI = ratio.max(0.0);
		}
	}
	
	pub fn getWindowHDPI(&self) -> f32
	{
		return self._windowInfos.read().HDPI;
	}
	
	pub fn getWindowCorrectedMousePos(&self, mousex: &mut f64, mousey: &mut f64)
	{
		let hdpi = self.getWindowHDPI() as f64;
		let tmpx = *mousex * hdpi;
		let tmpy = *mousey * hdpi;
		
		match self._windowInfos.read().orientation {
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
		return self._stdAllocSet.load().clone();
	}
	
	pub fn getDevice(&self) -> Guard<Arc<BuilderDevice>>
	{
		return self._builderDevice.load();
	}
	
	pub fn SecondaryCmdBuffer_generate(&self) -> AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>
	{
		AutoCommandBufferBuilder::secondary(
			self._rendering.read().getAllocCmd(),
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
	
	pub fn engineResumed(&self, rawWindow: impl HasRawWindowHandle + HasRawDisplayHandle + Any + Send + Sync) -> anyhow::Result<()>
	{
		HTrace!("Engine context creation ----");
		let newsurface = Self::Init_SurfaceReload(rawWindow,self._instance.load().clone())?;
		self._surface.swap(Some(newsurface.clone()));
		
		self._rendering.write().recreate(self._builderDevice.load().clone(),newsurface);
		
		ManagerInterface::singleton().WindowRefreshed();
		self._isSuspended.swap(Arc::new(false));
		Ok(())
	}
	
	pub fn engineSuspended(&self)
	{
		HTrace!("Engine context deleted ----");
		self._surface.swap(None);
		self._isSuspended.swap(Arc::new(true));
	}
	
	pub fn engineIsSuspended(&self) -> bool
	{
		*self._isSuspended.load_full()
	}
	
	pub fn window_resize(&self, size: Option<[u32;2]>)
	{
		self.window_InfosUpdate(size);
		
		self._rendering.write().forceSwapchainRecreate();
	}
	
	///////////// PRIVATE
	
	fn new(instance: Arc<Instance>, builder_device: Arc<BuilderDevice>, surface: Arc<Surface>, stdAllocSet: Arc<StandardDescriptorSetAllocator>, rendering: HGErendering) -> Self
	{
		let config = HGEconfig::singleton().general_get();
		
		let mut threadService = SingletonThread::newFiltered(||{
			Self::singleton()._cameraAnimation.write().retain_mut(|anim| {
				!anim.ticks()
			});
			
			ManagerInterface::singleton().tickUpdate();
			ManagerModels::singleton().tickUpdate();
			ManagerFont::singleton().FontEngine_CacheUpdate();
			ManagerTexture::singleton().launchThreads();
			ManagerAnimation::singleton().ticksAll();
			
			ShaderDrawer_Manager::allholder_Update();
		},||{
			!**Self::singleton()._isSuspended.load()
		});
		threadService.setDuration(Duration::from_nanos(1));
		
		return Self
		{
			_instance: ArcSwap::new(instance),
			_surface: ArcSwapOption::new(Some(surface)),
			_isSuspended: ArcSwap::new(Arc::new(true)),
			_builderDevice: ArcSwap::new(builder_device),
			_rendering: Arc::new(RwLock::new(rendering)),
			_appName: ArcSwap::new(Arc::new(config.windowTitle.clone())),
			_appVersion: ArcSwap::new(Arc::new(config.appVersion)),
			_stdAllocSet: ArcSwap::new(stdAllocSet),
			_windowInfos: RwLock::new(window_infos::default()),
			_timeAppStart: Instant::now(),
			_lastFrameDuration: RwLock::new(Duration::from_nanos(0)),
			_cameraAnimation: RwLock::new(vec![]),
			_cameraC: HArcMut::new(Camera::new()),
			_mouseMode: RwLock::new(true),
			_ManagerInterpolate: RwLock::new(ManagerInterpolate::new()),
			_cmdBufferTextures: Arc::new(DashMap::new()),
			_thread_runService: Mutex::new(threadService),
		};
	}
	
	fn window_InfosUpdate(&self, size: Option<[u32;2]>)
	{
		let Some(surfaceCap) = self.getSurfaceCapability() else {
			return;
		};
		
		HTrace!("viewport pre size information : {:?}", size);
		let rawwidth;
		let rawheight;
		if let Some(winsize) = size
		{
			rawwidth = winsize[0];
			rawheight = winsize[1];
		}
		else
		{
			let extends = surfaceCap.current_extent.unwrap_or([100, 100]);
			rawwidth = extends[0];
			rawheight = extends[1];
		}
		
		let mut bindingWindowInfos = self._windowInfos.write();
		let hdpi = bindingWindowInfos.HDPI;
		
		HTrace!("viewport dim conv : [{},{}]", rawwidth, rawheight);
		HTrace!("viewport dim hdpi : {}", hdpi);
		HTrace!("viewport dim iswide : {}", rawwidth>rawheight);
		
		let widthF = (rawwidth as f32 * hdpi) as f32;
		let heightF = (rawheight as f32 * hdpi) as f32;
		
		*bindingWindowInfos = window_infos {
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
			HDPI: hdpi,
		};
	}
	
	fn Init_SurfaceReload(rawWindow: impl HasRawWindowHandle + HasRawDisplayHandle + Any + Send + Sync, instance: Arc<Instance>) -> anyhow::Result<Arc<Surface>>
	{
		let surface = Surface::from_window(instance, Arc::new(rawWindow)).map_err(|e| { anyhow!(e) })?;
		Ok(surface)
	}
	
	fn engineLoad(&self) -> anyhow::Result<()>
	{
		HTrace!("Engine load internal ----");
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
		
		ShaderDrawer_Manager::singleton().register::<HGE_shader_2Dsimple_holder>(HGEsubpassName::UI);
		ShaderDrawer_Manager::singleton().register::<HGE_shader_2Dline_holder>(HGEsubpassName::UI);
		ShaderDrawer_Manager::singleton().register::<HGE_shader_3Dsimple_holder>(HGEsubpassName::WORLDSOLID);
		ShaderDrawer_Manager::singleton().register::<HGE_shader_3Dinstance_holder>(HGEsubpassName::WORLDSOLID);
		
		HGE_shader_3Dinstance::createPipeline()?;
		HGE_shader_3Dsimple::createPipeline()?;
		HGE_shader_2Dsimple::createPipeline()?;
		HGE_shader_screen::createPipeline()?;
		
		self._rendering.write().window_size_dependent_setup();
		ManagerTexture::singleton().preload();
		
		let tmp = HGEconfig::singleton().general_get();
		HTraceError!(ManagerFont::singleton().FontLoad(tmp.fonts.path_fileUser.clone(),tmp.fonts.path_fileUniversel.clone(),tmp.fonts.path_fileBold.clone()));
		
		HTrace!("Engine load internal end ----");
		Ok(())
	}
	
	fn Init_Instance(required_extensions: InstanceExtensions) -> anyhow::Result<Arc<Instance>>
	{
		let library = VulkanLibrary::new().unwrap();
		let debuglayer = Vec::new();
		let config = HGEconfig::singleton().general_get();
		
		/*{
			debuglayer.push("VK_LAYER_KHRONOS_validation".to_string());
		}*/
		
		// Now creating the instance.
		let instance = Instance::new(
			library,
			InstanceCreateInfo {
				application_name: Some(config.windowTitle.clone()),
				application_version: config.appVersion,
				engine_name: Some(HGE_STRING.to_string()),
				engine_version: HGE_VERSION,
				enabled_layers: debuglayer,
				enabled_extensions: required_extensions,
				// Enable enumerating devices that use non-conformant vulkan implementations. (ex. MoltenVK)
				flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
				..Default::default()
			},
		)?;
		
		Ok(instance)
	}
	
	
	fn getSurfaceCapability(&self) -> Option<SurfaceCapabilities>
	{
		let Some(surface) = &*self._surface.load() else {
			return None;
		};
		
		let builderDevice = self._builderDevice.load();
		if let Ok(result) = builderDevice.device
			.physical_device()
			.surface_capabilities(surface, Default::default())
		{
			return Some(result);
		}
		
		return None;
	}
}
