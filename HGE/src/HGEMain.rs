extern crate vulkano;

use crate::components::system::DeferredDatas::DeferredData;
use crate::components::system::TimeStats::TimeStatsStorage;
use crate::components::window::{window_infos, window_orientation};
use crate::configs::HGEconfig::HGEconfig;
use crate::Animation::Animation;
use crate::BuilderDevice::BuilderDevice;
use crate::Camera::Camera;
use crate::HGEMain_preinit::{HGEMain_preinitState, Initial, Ready};
use crate::HGErendering::HGErendering;
use crate::HGEsubpass::HGEsubpassName;
use crate::Interface::ManagerFont::ManagerFont;
use crate::Interface::ManagerInterface::ManagerInterface;
use crate::InterpolateTimer::ManagerInterpolate;
use crate::ManagerAnimation::{AnimationHolder, ManagerAnimation};
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;
use crate::Models3D::ManagerModels::ManagerModels;
use crate::Shaders::HGE_shader_2Dsimple::{HGE_shader_2Dline_holder, HGE_shader_2Dsimple, HGE_shader_2Dsimple_holder};
use crate::Shaders::HGE_shader_3Dinstance::{HGE_shader_3Dinstance, HGE_shader_3Dinstance_holder};
use crate::Shaders::HGE_shader_3Dsimple::{HGE_shader_3Dsimple, HGE_shader_3Dsimple_holder};
use crate::Shaders::HGE_shader_screen::HGE_shader_screen;
use crate::Shaders::ShaderDrawer::ShaderDrawer_Manager;
use crate::Shaders::ShaderStruct::{ShaderStruct, ShaderStructHolder};
use crate::Textures::Manager::ManagerTexture;
use crate::Textures::TextureDescriptor::TextureDescriptor;
use crate::Textures::TextureDescriptor_type::{TextureDescriptor_exclude, TextureDescriptor_process, TextureDescriptor_type};
use anyhow::anyhow;
use arc_swap::{ArcSwap, ArcSwapOption, Guard};
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use singletonThread::SingletonThread;
use std::ops::Range;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{CommandBufferInheritanceInfo, SecondaryAutoCommandBuffer};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::instance::Instance;
use vulkano::swapchain::{Surface, SurfaceCapabilities, SurfaceTransform};
use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
	Version,
};
use HArcMut::HArcMut;
use Hconfig::HConfigManager::HConfigManager;
use Htrace::{namedThread, HTrace, HTraceError};

pub(crate) const HGE_STRING: &str = "HGE";
pub(crate) const HGE_VERSION: Version = Version { major: 1, minor: 0, patch: 0 };

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum HGEMain_secondarybuffer_type
{
	TEXTURE,
	GRAPHIC,
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
	_stdCmdAllocSet: ArcSwap<StandardCommandBufferAllocator>,
	_stdDescAllocSet: ArcSwap<StandardDescriptorSetAllocator>,
	_cmdBufferTextures: DashMap<HGEMain_secondarybuffer_type, DeferredData<(Vec<Arc<SecondaryAutoCommandBuffer>>, Vec<Arc<dyn Fn() + Send + Sync>>)>>,

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

impl HGEMain
{
	pub fn singleton() -> &'static Self
	{
		return SINGLETON.get().unwrap_or_else(|| panic!("HGE have not been initialized"));
	}

	pub fn preInitialize() -> anyhow::Result<HGEMain_preinitState<Initial>>
	{
		if (SINGLETON.get().is_some())
		{
			return Err(anyhow!("HGE already initialized"));
		}

		return Ok(HGEMain_preinitState::new());
	}

	pub fn initialize(surface: Arc<Surface>, preinit: anyhow::Result<HGEMain_preinitState<Ready>>) -> anyhow::Result<()>
	{
		let instance = match preinit
		{
			Ok(preinitdata) => preinitdata.getInstance(),
			Err(err) =>
			{
				return Err(anyhow!("toto {}", err));
			}
		};

		HTrace!("Engine initialization : device build");
		let builderDevice = Arc::new(BuilderDevice::new(instance.clone(), surface.clone()));

		/// user information
		let mut config = HConfigManager::singleton().get(HGEconfig::singleton().general_get().configName.clone());
		config.set(
			"system/swapchain/presentmode_allowed",
			builderDevice
				.surfaceCapabilities
				.as_ref()
				.map(|s| s.compatible_present_modes.iter().map(|p| format!("{:?}", p)).collect::<Vec<String>>())
				.unwrap_or(vec!["fifo".to_string()]),
		);

		HTrace!("Engine initialization : Memory allocator build");
		ManagerMemoryAllocator::singleton().update(builderDevice.device.clone());
		let stdAllocSet = Arc::new(StandardDescriptorSetAllocator::new(builderDevice.device.clone(), Default::default()));

		HTrace!("Engine initialization : rendering build");
		let rendering = HGErendering::new(builderDevice.clone(), surface.clone())?;

		HTrace!("Engine initialization : HGE creation");
		let selfnew = Self::new(instance, builderDevice, surface, stdAllocSet, rendering);
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
		TimeStatsStorage::forceNow("R_service");
		Self::singleton()._thread_runService.lock().thread_launch();
		TimeStatsStorage::update("R_service");
	}

	pub fn runRendering(&self, preSwapFunc: impl Fn())
	{
		if (**Self::singleton()._isSuspended.load())
		{
			return;
		}

		let durationFromLast = Self::singleton()._ManagerInterpolate.read().getNowFromLast();
		let mut tmp = self._rendering.write();
		if (tmp.rendering(durationFromLast, preSwapFunc))
		{
			tmp.drawStats();
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

	pub fn getInstance(&self) -> Guard<Arc<Instance>>
	{
		return self._instance.load();
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

		match self._windowInfos.read().orientation
		{
			window_orientation::NORMAL | window_orientation::ROT_180 =>
			{
				*mousex = tmpx;
				*mousey = tmpy;
			}
			window_orientation::ROT_90 | window_orientation::ROT_270 =>
			{
				*mousex = tmpy;
				*mousey = tmpx;
			}
		}
	}

	pub fn getCmdAllocatorSet(&self) -> Arc<StandardCommandBufferAllocator>
	{
		return self._stdCmdAllocSet.load().clone();
	}

	pub fn getDescAllocatorSet(&self) -> Arc<StandardDescriptorSetAllocator>
	{
		return self._stdDescAllocSet.load().clone();
	}

	pub fn getDevice(&self) -> Guard<Arc<BuilderDevice>>
	{
		return self._builderDevice.load();
	}

	pub fn SecondaryCmdBuffer_generate(&self) -> AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>
	{
		AutoCommandBufferBuilder::secondary(
			self.getCmdAllocatorSet(),
			Self::singleton().getDevice().getQueueGraphic().queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
			CommandBufferInheritanceInfo { ..Default::default() },
		)
		.unwrap()
	}

	pub fn SecondaryCmdBuffer_add(sbtype: HGEMain_secondarybuffer_type, cmdBuffer: Arc<SecondaryAutoCommandBuffer>, callback: impl Fn() + Send + Sync + 'static)
	{
		let _ = namedThread!(move || {
			if (!Self::singleton()._cmdBufferTextures.contains_key(&sbtype))
			{
				Self::singleton()._cmdBufferTextures.insert(sbtype, DeferredData::new());
			}

			let Some(binding) = Self::singleton()._cmdBufferTextures.get_mut(&sbtype)
			else
			{
				return;
			};
			let mut arrayvec = binding.inputMut();
			match &mut *arrayvec
			{
				None =>
				{
					*arrayvec = Some((vec![cmdBuffer], vec![Arc::new(callback)]));
				}
				Some((contentCommand, contentCallback)) =>
				{
					contentCommand.push(cmdBuffer);
					contentCallback.push(Arc::new(callback));
				}
			}
		});
	}

	pub(crate) fn SecondaryCmdBuffer_drain(typed: HGEMain_secondarybuffer_type) -> Option<(Vec<Arc<SecondaryAutoCommandBuffer>>, Vec<Arc<dyn Fn() + Send + Sync>>)>
	{
		let Some(dataBinding) = Self::singleton()._cmdBufferTextures.get(&typed)
		else
		{
			return None;
		};
		let Some(data) = dataBinding.steal()
		else
		{
			return None;
		};

		return Some(data);
	}

	pub fn engineResumed(&self, surface: Arc<Surface>) -> anyhow::Result<()>
	{
		HTrace!("Engine context creation ----");
		self._surface.swap(Some(surface.clone()));

		self._rendering.write().recreate(self._builderDevice.load().clone(), surface);

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

	pub fn window_resize(&self, size: Option<[u32; 2]>)
	{
		self.window_InfosUpdate(size);

		self._rendering.write().forceSwapchainRecreate();
		ManagerInterface::singleton().WindowRefreshed();
	}

	///////////// PRIVATE

	fn new(instance: Arc<Instance>, builder_device: Arc<BuilderDevice>, surface: Arc<Surface>, stdAllocSet: Arc<StandardDescriptorSetAllocator>, rendering: HGErendering) -> Self
	{
		let config = HGEconfig::singleton().general_get();

		let mut threadService = SingletonThread::newFiltered(
			|| {
				/*if (**Self::singleton()._isSuspended.load())
				{
					return;
				}*/

				Self::singleton()._cameraAnimation.write().retain_mut(|anim| !anim.ticks());
				Self::singleton()._cmdBufferTextures.iter().for_each(|x| x.thread_launch());

				ManagerInterface::singleton().tickUpdate();
				ManagerModels::singleton().tickUpdate();
				ManagerFont::singleton().FontEngine_CacheUpdate();
				ManagerTexture::singleton().launchThreads();
				ManagerAnimation::singleton().ticksAll();

				ShaderDrawer_Manager::allholder_Update();
			},
			|| !**Self::singleton()._isSuspended.load(),
		);
		//threadService.setLoop(true);

		let stdAlloccommand = StandardCommandBufferAllocator::new(builder_device.device.clone(), Self::getDefaultAllocInfos());

		return Self {
			_instance: ArcSwap::new(instance),
			_surface: ArcSwapOption::new(Some(surface)),
			_isSuspended: ArcSwap::new(Arc::new(true)),
			_builderDevice: ArcSwap::new(builder_device),
			_rendering: Arc::new(RwLock::new(rendering)),
			_appName: ArcSwap::new(Arc::new(config.windowTitle.clone())),
			_appVersion: ArcSwap::new(Arc::new(config.appVersion)),
			_stdCmdAllocSet: ArcSwap::new(Arc::new(stdAlloccommand)),
			_stdDescAllocSet: ArcSwap::new(stdAllocSet),
			_windowInfos: RwLock::new(window_infos::default()),
			_timeAppStart: Instant::now(),
			_lastFrameDuration: RwLock::new(Duration::from_nanos(0)),
			_cameraAnimation: RwLock::new(vec![]),
			_cameraC: HArcMut::new(Camera::new()),
			_mouseMode: RwLock::new(true),
			_ManagerInterpolate: RwLock::new(ManagerInterpolate::new()),
			_cmdBufferTextures: DashMap::new(),
			_thread_runService: Mutex::new(threadService),
		};
	}

	fn window_InfosUpdate(&self, size: Option<[u32; 2]>)
	{
		let Some(surfaceCap) = self.getSurfaceCapability()
		else
		{
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
		HTrace!("viewport dim iswide : {}", rawwidth > rawheight);

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
			surfaceCapabilities: Some(surfaceCap),
		};
	}

	fn engineLoad(&self) -> anyhow::Result<()>
	{
		HTrace!("Engine load internal ----");

		/*let vs = {
			let mut f = File::open("./static/shaders/vert3D.glsl")
				.expect("./static/shaders/vert3D.glsl This example needs to be run from the root of the example crate.");
			let mut v = vec![];
			f.read_to_end(&mut v).unwrap();
			// Create a ShaderModule on a device the same Shader::load does it.
			// NOTE: You will have to verify correctness of the data by yourself!
			unsafe { ShaderModule::from_bytes(device.clone(), &v) }.unwrap()
		};*/
		// https://github.com/vulkano-rs/vulkano/commit/fe01ddd5e3f178b971ed102dd5fdd93cee5d87b9#diff-d246486211c651344a5f0381a9258a41abeea7678bb10c3e4c855372f8b9b8e4

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

		let mut texturesize = 256;
		let mut texturesizebig = 1024;
		if (cfg!(target_os = "android"))
		{
			texturesize = 64;
			texturesizebig = 256;
		}

		ManagerTexture::singleton().preload();
		ManagerTexture::singleton().descriptorSet_create(
			"HGE_set0",
			TextureDescriptor::new(
				TextureDescriptor_type::<Range<u16>>::ONE("font".to_string()),
				TextureDescriptor_exclude::NONE,
				HGE_shader_2Dsimple_holder::pipelineName(),
				0,
				"default",
			),
		);
		ManagerTexture::singleton().descriptorSet_create(
			"HGE_set1",
			TextureDescriptor::new(
				TextureDescriptor_type::SIZE_DEPENDENT(0..512, TextureDescriptor_process::RESIZE(texturesize, texturesize)),
				TextureDescriptor_exclude::ARRAY(vec!["font".to_string()]),
				HGE_shader_2Dsimple_holder::pipelineName(),
				1,
				"default",
			),
		);
		ManagerTexture::singleton().descriptorSet_create(
			"HGE_set2",
			TextureDescriptor::new(
				TextureDescriptor_type::SIZE_MIN(512.., TextureDescriptor_process::RESIZE(texturesizebig, texturesizebig)),
				TextureDescriptor_exclude::ARRAY(vec!["font".to_string()]),
				HGE_shader_2Dsimple_holder::pipelineName(),
				2,
				"default",
			),
		);

		let tmp = HGEconfig::singleton().general_get();
		HTraceError!(ManagerFont::singleton().FontLoad(tmp.fonts.path_fileUser.clone(), tmp.fonts.path_fileUniversel.clone(), tmp.fonts.path_fileBold.clone()));

		HTrace!("Engine load internal end ----");
		Ok(())
	}

	fn getSurfaceCapability(&self) -> Option<SurfaceCapabilities>
	{
		let Some(surface) = &*self._surface.load()
		else
		{
			return None;
		};

		let builderDevice = self._builderDevice.load();
		if let Ok(result) = builderDevice.device.physical_device().surface_capabilities(surface, Default::default())
		{
			return Some(result);
		}

		return None;
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
}
