use crate::configs::HGEconfig::HGEconfig;
use crate::fronts::winit::internalWinitState::internalWinitState;
use crate::fronts::winit::Inputs::Inputs;
use crate::fronts::winit::UserDefinedEventOverride::UserDefinedEventOverride;
use crate::fronts::EngineEvent::EngineEvent;
use crate::HGEMain::HGEMain;
use parking_lot::lock_api::{RwLockReadGuard, RwLockWriteGuard};
use parking_lot::{RawRwLock, RwLock};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::any::Any;
use std::sync::{Arc, OnceLock};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Fullscreen, Window};
use Hconfig::serde_json::Value as JsonValue;
use Hconfig::HConfigManager::HConfigManager;
use Htrace::{HTrace, HTraceError};

pub struct HGEwinit
{
	_events: RwLock<EngineEvent>,
	_inputsC: RwLock<Inputs>,
}

static SINGLETON: OnceLock<HGEwinit> = OnceLock::new();

impl HGEwinit
{
	pub fn singleton() -> &'static Self
	{
		return SINGLETON.get_or_init(|| Self::new());
	}

	/// change func called after engine initialization
	pub fn event_mut(&self) -> RwLockWriteGuard<'_, RawRwLock, EngineEvent>
	{
		self._events.write()
	}

	pub fn Inputs_get(&self) -> RwLockReadGuard<'_, RawRwLock, Inputs>
	{
		self._inputsC.read()
	}

	pub fn Inputs_getmut(&self) -> RwLockWriteGuard<'_, RawRwLock, Inputs>
	{
		return self._inputsC.write();
	}

	/// run winit event, HGE engine, and connect logic of your system
	/// postEngineEvent is run between engine event and rendering
	/// Should run on the main thread only
	pub fn runDefinedLoop(
		eventloop: EventLoop<()>,
		userEvents: Option<&mut impl UserDefinedEventOverride>,
	)
	{
		HTraceError!(eventloop.run_app(&mut internalWinitState::new(userEvents)));
	}

	/// run winit event, HGE engine, and connect logic of your system
	/// postEngineEvent is run between engine event and rendering
	/// EventLoop::new() is defined to default value
	/// Should run on the main thread only
	pub fn run(userEvents: Option<&mut impl UserDefinedEventOverride>)
	{
		let eventloop = EventLoop::new().unwrap();
		Self::runDefinedLoop(eventloop, userEvents);
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

	//////////////////// PRIVATE /////////////////

	fn new() -> Self
	{
		Self {
			_events: RwLock::new(EngineEvent::new()),
			_inputsC: RwLock::new(Inputs::new()),
		}
	}

	pub(crate) fn buildHandle(
		eventloop: &ActiveEventLoop,
	) -> Arc<impl HasWindowHandle + HasDisplayHandle + Any + Send + Sync>
	{
		let configBind = HGEconfig::singleton().general_get();

		let mut defaultwindowtype = 2; // 1 or 2 = fullscreen
		if (!configBind.startFullscreen)
		{
			defaultwindowtype = 0;
		}

		let mut config = HConfigManager::singleton().get(configBind.configName.clone());
		let mut windowtype = config
			.getOrSetDefault("window/type", JsonValue::from(defaultwindowtype))
			.as_u64()
			.unwrap_or(2);
		let mut fullscreenmode = None;
		if (windowtype == 1 && eventloop.primary_monitor().is_none())
		{
			windowtype = 2;
		}
		if (configBind.isSteamdeck || configBind.isAndroid)
		// config ignored for steam deck and android
		{
			windowtype = 1;
			config.set("window/type", JsonValue::from(windowtype));
		}

		if (windowtype == 1)
		{
			let mut video_mode = eventloop
				.primary_monitor()
				.unwrap()
				.video_modes()
				.collect::<Vec<_>>();
			HTrace!("video modes : {:?}", video_mode);
			video_mode.sort_by(|a, b| {
				use std::cmp::Ordering::*;
				match b.size().width.cmp(&a.size().width)
				{
					Equal => match b.size().height.cmp(&a.size().height)
					{
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
			Err(err) =>
			{
				panic!("winit create window error : {:?}", err);
			}
		};

		let _ = config.save();

		return Arc::new(window);
	}
}
