use crate::configs::HGEconfig::HGEconfig;
use crate::fronts::agnosticHandle::agnosticHandle;
use crate::fronts::EngineEvent::EngineEvent;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::{Sdl, VideoSubsystem};
use std::any::Any;
use std::sync::Arc;
use Hconfig::serde_json::Value as JsonValue;
use Hconfig::HConfigManager::HConfigManager;

pub struct HGEsdl
{
	_sdl_context: Sdl,
	_sdl_video: VideoSubsystem,
	_events: EngineEvent,
}

impl HGEsdl
{
	pub fn new() -> Self
	{
		let sdl_context = sdl2::init().unwrap();
		let video_subsystem = sdl_context.video().unwrap();

		Self {
			_sdl_context: sdl_context,
			_sdl_video: video_subsystem,
			_events: EngineEvent::new(),
		}
	}

	/// change func called on specific engine events
	pub fn events_mut(&mut self) -> &mut EngineEvent
	{
		&mut self._events
	}

	pub fn sdlContext_get(&self) -> &Sdl
	{
		&self._sdl_context
	}

	/// run winit event, HGE engine, and connect logic of your system
	/// postEngineEvent is run between engine event and rendering
	pub fn run(&mut self)
	{
		let preinit = self._events.preInit();
		self._events.init(self.buildHandle(), preinit); // buildHandle do not call buildHandle fedore preInit()

		let mut event_pump = self._sdl_context.event_pump().unwrap();
		'running: loop
		{
			for event in event_pump.poll_iter()
			{
				if (self.internalEvent(&event))
				{
					break 'running;
				}
			}

			self._events.window_draw(|| {});
		}
	}

	//////////////////// PRIVATE /////////////////

	fn internalEvent(&mut self, event: &Event) -> bool
	{
		match event
		{
			Event::AppDidEnterBackground { .. } | Event::AppWillEnterBackground { .. } =>
			{
				self._events.suspend();
			}
			Event::AppDidEnterForeground { .. } | Event::AppWillEnterForeground { .. } =>
			{
				let window = self.buildHandle();
				self._events.resume(window);
			}
			Event::Window {
				window_id,
				win_event: WindowEvent::Resized(width, height),
				..
			} =>
			{
				self._events
					.window_eventResize(*width as u32, *height as u32);
			}
			Event::Quit { .. }
			| Event::KeyDown {
				keycode: Some(Keycode::Escape),
				..
			} =>
			{
				self._events.window_eventClose();
				return true;
			}
			_ =>
			{}
		}

		return false;
	}

	fn buildHandle(&self) -> Arc<impl HasWindowHandle + HasDisplayHandle + Any + Send + Sync>
	{
		let configBind = HGEconfig::singleton().general_get();

		let mut windowbuilder = self._sdl_video.window(&configBind.windowTitle, 800, 600);
		windowbuilder.position_centered();
		windowbuilder.resizable();
		windowbuilder.vulkan();

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

		/*if (windowtype == 1 && eventloop.primary_monitor().is_none())
		{
			windowtype = 2;
		}*/
		if (configBind.isSteamdeck || configBind.isAndroid)
		// config ignored for steam deck and android
		{
			windowtype = 1;
			config.set("window/type", JsonValue::from(windowtype));
		}

		if (windowtype == 1)
		{
			windowbuilder.fullscreen_desktop();
		}
		if (windowtype == 2)
		{
			windowbuilder.borderless();
		}

		let window = match windowbuilder.build()
		{
			Ok(x) => x,
			Err(err) =>
			{
				panic!("sdl2 create window error : {:?}", err);
			}
		};

		return Arc::new(agnosticHandle::newFromHandle(window));
	}
}
