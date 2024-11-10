pub extern crate winit;

use crate::configs::general::HGEconfig_general;
use crate::configs::HGEconfig::HGEconfig;
use crate::fronts::Inputs::Inputs;
use crate::HGEMain::HGEMain;
use anyhow::anyhow;
use parking_lot::lock_api::{RwLockReadGuard, RwLockWriteGuard};
use parking_lot::{RawRwLock, RwLock};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::any::Any;
use std::sync::OnceLock;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Fullscreen, Window};
use Hconfig::serde_json::Value as JsonValue;
use Hconfig::HConfigManager::HConfigManager;
use Htrace::{HTrace, HTraceError};
use crate::fronts::internalWinitState::internalWinitState;
use crate::fronts::UserDefinedEventOverride::UserDefinedEventOverride;

pub struct HGEwinit
{
	_funcPostInit: RwLock<Box<dyn FnMut() + Send + Sync>>,
	_funcPre_exit: RwLock<Box<dyn FnMut() + Send + Sync>>,
	_inputsC: RwLock<Inputs>,
}

static SINGLETON: OnceLock<HGEwinit> = OnceLock::new();

impl HGEwinit
{
	pub fn singleton() -> &'static Self
	{
		return SINGLETON.get_or_init(|| {
			Self::new()
		});
	}
	
	/// change func called after engine initialization
	pub fn setFunc_PostInit(&self, func: impl FnMut() + Send + Sync + 'static)
	{
		*self._funcPostInit.write() = Box::new(func);
	}
	
	/// change func called before exit
	pub fn setFunc_PreExit(&self, func: impl FnMut() + Send + Sync + 'static)
	{
		*self._funcPre_exit.write() = Box::new(func);
	}
	
	pub fn Inputs_get(&self) -> RwLockReadGuard<'_, RawRwLock, Inputs>
	{
		return self._inputsC.read();
	}
	
	pub fn Inputs_getmut(&self) -> RwLockWriteGuard<'_, RawRwLock, Inputs>
	{
		return self._inputsC.write();
	}
	
	/// run winit event, HGE engine, and connect logic of your system
	/// postEngineEvent is run between engine event and rendering
	pub fn run(eventloop: EventLoop<()>, generalConf: HGEconfig_general, userEvents: Option<&mut impl UserDefinedEventOverride>)
	{
		HTraceError!(eventloop.run_app(&mut internalWinitState::new(generalConf, userEvents)));
	}
	
	pub fn getWindow(func: impl FnOnce(&Window))
	{
		let surfaceBinding = HGEMain::singleton().getSurface();
		if let Some(surface) = &*surfaceBinding
		{
			let tmp = surface.object().unwrap().downcast_ref::<Window>().unwrap();
			func(tmp);
		}
	}
	
	pub(crate) fn inputs_get_mut<'a>() -> RwLockWriteGuard<'a, RawRwLock, Inputs>
	{
		return Self::singleton()._inputsC.write();
	}
	
	pub(crate) fn funcPostInit_get_mut<'a>() -> RwLockWriteGuard<'a, RawRwLock, Box<dyn FnMut() + Send + Sync>>
	{
		return Self::singleton()._funcPostInit.write();
	}
	
	pub(crate) fn funcPreExit_get_mut<'a>() -> RwLockWriteGuard<'a, RawRwLock, Box<dyn FnMut() + Send + Sync>>
	{
		return Self::singleton()._funcPre_exit.write();
	}
	
	//////////////////// PRIVATE /////////////////
	
	fn new() -> Self
	{
		Self {
			_funcPostInit: RwLock::new(Box::new(|| {})),
			_funcPre_exit: RwLock::new(Box::new(|| {})),
			_inputsC: RwLock::new(Inputs::new()),
		}
	}
	
	pub(crate) fn buildAgnosticWindow(eventloop: &ActiveEventLoop) -> anyhow::Result<impl HasRawWindowHandle + HasRawDisplayHandle + Any + Send + Sync>
	{
		let configBind = HGEconfig::singleton().general_get();
		
		let mut defaultwindowtype = 2; // 1 or 2 = fullscreen
		if (!HGEconfig::singleton().general_get().startFullscreen)
		{
			defaultwindowtype = 0;
		}
		
		let mut config = HConfigManager::singleton().get(configBind.configName.clone());
		let mut windowtype = config.getOrSetDefault("window/type", JsonValue::from(defaultwindowtype)).as_u64().unwrap_or(2);
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
		
		let windowattr = Window::default_attributes()
			//.with_min_inner_size(LogicalSize{ width: 640, height: 480 })
			//.with_name("Truc much", "yolo")
			.with_title(&configBind.windowTitle)
			.with_fullscreen(fullscreenmode);
		
		let window = match eventloop.create_window(windowattr)
		{
			Ok(x) => x,
			Err(err) => { return Err(anyhow!("winit create window error : {:?}",err)); }
		};
		
		let _ = config.save();
		
		
		return Ok(window);
	}
}
