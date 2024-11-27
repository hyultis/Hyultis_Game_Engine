use crate::fronts::winit::front::HGEwinit;
use crate::fronts::winit::UserDefinedEventOverride::UserDefinedEventOverride;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

pub struct internalWinitState<'a>
{
	events: Option<Box<&'a mut dyn UserDefinedEventOverride>>,
}

impl<'a> internalWinitState<'a>
{
	pub fn new(userEvents: Option<&'a mut impl UserDefinedEventOverride>) -> Self
	{
		Self {
			events: userEvents.map(|e| Box::new(e as &'a mut dyn UserDefinedEventOverride)),
		}
	}
}

impl<'a> ApplicationHandler<()> for internalWinitState<'a>
{
	fn resumed(&mut self, eventloop: &ActiveEventLoop)
	{
		let mut tmp = HGEwinit::singleton().event_mut();
		if (!tmp.isInitialized())
		{
			let preinit = tmp.preInit();
			let window = HGEwinit::buildHandle(eventloop);
			tmp.init(window, preinit);

			return;
		}

		let window = HGEwinit::buildHandle(eventloop);
		HGEwinit::singleton().event_mut().resume(window);

		if let Some(tmp) = &mut self.events
		{
			tmp.resumed(eventloop);
		}
	}

	fn suspended(&mut self, eventloop: &ActiveEventLoop)
	{
		if (!HGEwinit::singleton().event_mut().suspend())
		{
			return;
		}

		if let Some(tmp) = &mut self.events
		{
			tmp.suspended(eventloop);
		}
	}

	fn window_event(&mut self, eventloop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent)
	{
		if (!HGEwinit::singleton().event_mut().isInitialized())
		{
			return;
		}

		match &event
		{
			WindowEvent::KeyboardInput { event: input, .. } =>
			{
				//println!("key input : {:?}",input);
				if let PhysicalKey::Code(key) = input.physical_key
				{
					let mut inputsC = HGEwinit::inputs_get_mut();
					inputsC.updateFromKeyboard(key, input.state);
				}
			}
			WindowEvent::Resized(winsize) =>
			{
				HGEwinit::singleton()
					.event_mut()
					.window_eventResize(winsize.width.max(1), winsize.height.max(1));
			}
			WindowEvent::CloseRequested =>
			{
				eventloop.exit();
			}
			WindowEvent::Destroyed =>
			{
				HGEwinit::singleton().event_mut().window_eventClose();
				eventloop.exit();
			}
			WindowEvent::RedrawRequested =>
			//problem with windows ? https://github.com/rust-windowing/winit/pull/3950
			//https://github.com/rust-windowing/winit/issues/3272
			{
				if let Some(tmp) = &mut self.events
				{
					tmp.about_to_render(eventloop);
				}

				HGEwinit::singleton().event_mut().window_draw(|| {
					HGEwinit::getWindow(|window| {
						window.pre_present_notify();
					});
				});
			}
			_ => (),
		}

		if (event != WindowEvent::RedrawRequested)
		{
			if let Some(tmp) = &mut self.events
			{
				tmp.window_event(eventloop, &event, window_id);
			}
		}
	}

	fn device_event(&mut self, eventloop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent)
	{
		if (!HGEwinit::singleton().event_mut().isInitialized())
		{
			return;
		}

		if let Some(tmp) = &mut self.events
		{
			tmp.device_event(eventloop, &event, device_id);
		}
	}

	fn about_to_wait(&mut self, eventloop: &ActiveEventLoop)
	{
		if (!HGEwinit::singleton().event_mut().isInitialized())
		{
			return;
		}

		HGEwinit::singleton().event_mut().runService();
		HGEwinit::getWindow(|window| {
			window.request_redraw();
		});

		if let Some(tmp) = &mut self.events
		{
			tmp.about_to_wait(eventloop);
		}
		else
		{
			if (HGEwinit::inputs_get_mut().getKeyboardStateAndSteal(KeyCode::Escape)
				== ElementState::Pressed)
			{
				eventloop.exit();
			}
		}

		eventloop.set_control_flow(ControlFlow::Poll);
	}
}
