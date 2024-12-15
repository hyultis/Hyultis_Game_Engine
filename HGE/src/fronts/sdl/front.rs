use crate::configs::HGEconfig::HGEconfig;
use crate::fronts::sdl::sdl_UserDefinedEventOverride::sdl_UserDefinedEventOverride;
use crate::fronts::EngineEvent::EngineEvent;
use crate::HGEMain::HGEMain;
use sdl2::event::{Event, WindowEvent};
use sdl2::video::Window;
use sdl2::{Sdl, VideoSubsystem};
use std::sync::Arc;
use vulkano::instance::{Instance, InstanceExtensions};
use vulkano::swapchain::Surface;
use Hconfig::serde_json::Value as JsonValue;
use Hconfig::HConfigManager::HConfigManager;

pub struct HGEsdl
{
	_sdl_context: Sdl,
	_sdl_video: VideoSubsystem,
	_events: EngineEvent,
	_window: Option<Window>,
	_instance_extensions: Option<InstanceExtensions>,
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
			_window: None,
			_instance_extensions: None,
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
	pub fn run(
		&mut self,
		mut userEvent: Option<&mut impl sdl_UserDefinedEventOverride>,
	) -> anyhow::Result<()>
	{
		let preinit = self._events.preInit();
		self.rebuildWindow();
		let preinit = preinit?.setInstance(self._instance_extensions.unwrap());
		let preinit = preinit?;
		let surface = self.getSurface(preinit.getInstance());
		self._events.init(surface, Ok(preinit)); // buildHandle do not call buildHandle before preInit()

		let mut event_pump = self._sdl_context.event_pump().unwrap();
		'running: loop
		{
			for sdlevent in event_pump.poll_iter()
			{
				if (self.internalEvent(&sdlevent, &mut userEvent))
				{
					break 'running;
				}
				if let Some(event) = &mut userEvent
				{
					if (event.event(&sdlevent))
					{
						break 'running;
					}
				}
			}

			if let Some(event) = &mut userEvent
			{
				println!("dfsdf");
				event.about_to_render();
			}
			self._events.window_draw(|| {
				println!("dsfdff");
			});
			if let Some(event) = &mut userEvent
			{
				event.about_to_wait();
			}
		}

		return Ok(());
	}

	//////////////////// PRIVATE /////////////////

	fn internalEvent(
		&mut self,
		event: &Event,
		userEvent: &mut Option<&mut impl sdl_UserDefinedEventOverride>,
	) -> bool
	{
		match event
		{
			Event::AppDidEnterBackground { .. } | Event::AppWillEnterBackground { .. } =>
			{
				self._events.suspend();

				if let Some(event) = userEvent
				{
					event.suspended();
				}
			}
			Event::AppDidEnterForeground { .. } | Event::AppWillEnterForeground { .. } =>
			{
				let surface = self.getSurface(HGEMain::singleton().getInstance().clone());
				self._events.resume(surface);

				if let Some(event) = userEvent
				{
					event.resumed();
				}
			}
			Event::Window {
				win_event: WindowEvent::Resized(width, height),
				..
			} =>
			{
				self._events
					.window_eventResize(*width as u32, *height as u32);
			}
			Event::Quit { .. } =>
			{
				self._events.window_eventClose();
				return true;
			}
			_ =>
			{}
		}

		return false;
	}

	fn getSurface(&self, instance: Arc<Instance>) -> Arc<Surface>
	{
		return unsafe {
			Surface::from_window_ref(instance.clone(), &self._window.as_ref().unwrap()).unwrap()
		};
	}

	fn rebuildWindow(&mut self)
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

		let instance_extensions =
			InstanceExtensions::from_iter(window.vulkan_instance_extensions().unwrap());

		self._instance_extensions = Some(instance_extensions);
		self._window = Some(window);
	}
}
