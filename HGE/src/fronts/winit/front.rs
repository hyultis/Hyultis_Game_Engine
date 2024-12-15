use crate::configs::HGEconfig::HGEconfig;
use crate::fronts::winit::internalWinitState::internalWinitState;
use crate::fronts::winit::winit_UserDefinedEventOverride::winit_UserDefinedEventOverride;
use crate::fronts::winit::Inputs::Inputs;
use crate::fronts::EngineEvent::EngineEvent;
use raw_window_handle::HasDisplayHandle;
use std::sync::Arc;
use vulkano::instance::{Instance, InstanceExtensions};
use vulkano::swapchain::Surface;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Fullscreen, Window};
use Hconfig::serde_json::Value as JsonValue;
use Hconfig::HConfigManager::HConfigManager;
use Htrace::{HTrace, HTraceError};

pub struct HGEwinit
{
	_events: EngineEvent,
	_inputsC: Inputs,
	_window: Option<Window>,
	_instance_extensions: Option<InstanceExtensions>,
}

impl HGEwinit
{
	pub fn new() -> Self
	{
		Self {
			_events: EngineEvent::new(),
			_inputsC: Inputs::new(),
			_window: None,
			_instance_extensions: None,
		}
	}

	/// change func called after engine initialization
	pub fn event(&self) -> &EngineEvent
	{
		return &self._events;
	}

	/// change func called after engine initialization
	pub fn event_mut(&mut self) -> &mut EngineEvent
	{
		return &mut self._events;
	}

	pub fn Inputs_get(&self) -> &Inputs
	{
		return &self._inputsC;
	}

	pub fn Inputs_getmut(&mut self) -> &mut Inputs
	{
		return &mut self._inputsC;
	}

	/// run winit event, HGE engine, and connect logic of your system
	/// postEngineEvent is run between engine event and rendering
	/// Should run on the main thread only
	pub fn runDefinedLoop(
		&mut self,
		eventloop: EventLoop<()>,
		userEvents: Option<&mut impl winit_UserDefinedEventOverride>,
	)
	{
		HTraceError!(eventloop.run_app(&mut internalWinitState::new(self, userEvents)));
	}

	/// run winit event, HGE engine, and connect logic of your system
	/// postEngineEvent is run between engine event and rendering
	/// EventLoop::new() is defined to default value
	/// Should run on the main thread only
	pub fn run(&mut self, userEvents: Option<&mut impl winit_UserDefinedEventOverride>)
	{
		let eventloop = EventLoop::new().unwrap();
		self.runDefinedLoop(eventloop, userEvents);
	}

	pub fn getWindow(&self) -> &Option<Window>
	{
		return &self._window;
	}

	pub fn getInstanceExtensions(&self) -> &Option<InstanceExtensions>
	{
		return &self._instance_extensions;
	}

	pub fn getSurface(&self, instance: Arc<Instance>) -> Arc<Surface>
	{
		return unsafe {
			Surface::from_window_ref(instance, self._window.as_ref().unwrap()).unwrap()
		};
	}

	pub fn rebuildWindow(&mut self, eventloop: &ActiveEventLoop)
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

		let instance_extensions =
			Surface::required_extensions(&window.display_handle().unwrap()).unwrap();

		self._instance_extensions = Some(instance_extensions);
		self._window = Some(window);
	}
}
