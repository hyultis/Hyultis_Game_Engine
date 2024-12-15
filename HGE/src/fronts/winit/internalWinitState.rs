use crate::fronts::winit::front::HGEwinit;
use crate::fronts::winit::winit_UserDefinedEventOverride::winit_UserDefinedEventOverride;
use crate::HGEMain::HGEMain;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;
use Htrace::HTrace;
use Htrace::Type::Type;

pub struct internalWinitState<'a>
{
	events: Option<Box<&'a mut dyn winit_UserDefinedEventOverride>>,
	winit_root: &'a mut HGEwinit,
}

impl<'a> internalWinitState<'a>
{
	pub fn new(
		winit_root: &'a mut HGEwinit,
		userEvents: Option<&'a mut impl winit_UserDefinedEventOverride>,
	) -> Self
	{
		Self {
			events: userEvents.map(|e| Box::new(e as &'a mut dyn winit_UserDefinedEventOverride)),
			winit_root,
		}
	}
}

impl<'a> ApplicationHandler<()> for internalWinitState<'a>
{
	fn resumed(&mut self, eventloop: &ActiveEventLoop)
	{
		let tmpWinit = &mut self.winit_root;
		if (!tmpWinit.event_mut().isInitialized())
		{
			let preinit = tmpWinit.event_mut().preInit();

			tmpWinit.rebuildWindow(eventloop);
			match preinit
			{
				Ok(preinitcontent) =>
				{
					let instanceExtensions = tmpWinit.getInstanceExtensions().unwrap();
					let preinit = preinitcontent.setInstance(instanceExtensions);
					let surface =
						tmpWinit.getSurface(preinit.as_ref().unwrap().getInstance().clone());
					tmpWinit.event_mut().init(surface, preinit);
				}
				Err(err) =>
				{
					HTrace!((Type::ERROR) err);
					panic!("{}", err);
				}
			}

			return;
		}

		tmpWinit.rebuildWindow(eventloop);
		let tmpsurface = tmpWinit.getSurface(HGEMain::singleton().getInstance().clone());
		tmpWinit.event_mut().resume(tmpsurface);

		if let Some(tmp) = &mut self.events
		{
			tmp.resumed(self.winit_root, eventloop);
		}
	}

	fn suspended(&mut self, eventloop: &ActiveEventLoop)
	{
		if (!self.winit_root.event_mut().suspend())
		{
			return;
		}

		if let Some(tmp) = &mut self.events
		{
			tmp.suspended(self.winit_root, eventloop);
		}
	}

	fn window_event(&mut self, eventloop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent)
	{
		if (!self.winit_root.event_mut().isInitialized())
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
					let inputsC = self.winit_root.Inputs_getmut();
					inputsC.updateFromKeyboard(key, input.state);
				}
			}
			WindowEvent::Resized(winsize) =>
			{
				self.winit_root
					.event_mut()
					.window_eventResize(winsize.width.max(1), winsize.height.max(1));
			}
			WindowEvent::CloseRequested =>
			{
				eventloop.exit();
			}
			WindowEvent::Destroyed =>
			{
				self.winit_root.event_mut().window_eventClose();
				eventloop.exit();
			}
			WindowEvent::RedrawRequested =>
			//problem with windows ? https://github.com/rust-windowing/winit/pull/3950
			//https://github.com/rust-windowing/winit/issues/3272
			{
				if let Some(tmp) = &mut self.events
				{
					tmp.about_to_render(self.winit_root, eventloop);
				}

				let window = self.winit_root.getWindow().as_ref().unwrap();
				self.winit_root.event().window_draw(|| {
					window.pre_present_notify();
				});
			}
			_ => (),
		}

		if (event != WindowEvent::RedrawRequested)
		{
			if let Some(tmp) = &mut self.events
			{
				tmp.window_event(self.winit_root, eventloop, &event, window_id);
			}
		}
	}

	fn device_event(&mut self, eventloop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent)
	{
		if (!self.winit_root.event_mut().isInitialized())
		{
			return;
		}

		if let Some(tmp) = &mut self.events
		{
			tmp.device_event(self.winit_root, eventloop, &event, device_id);
		}
	}

	fn about_to_wait(&mut self, eventloop: &ActiveEventLoop)
	{
		if (!self.winit_root.event_mut().isInitialized())
		{
			return;
		}

		self.winit_root.event_mut().runService();
		if let Some(window) = self.winit_root.getWindow()
		{
			window.request_redraw();
		}

		if let Some(tmp) = &mut self.events
		{
			tmp.about_to_wait(self.winit_root, eventloop);
		}
		else
		{
			if (self
				.winit_root
				.Inputs_getmut()
				.getKeyboardStateAndSteal(KeyCode::Escape)
				== ElementState::Pressed)
			{
				eventloop.exit();
			}
		}

		eventloop.set_control_flow(ControlFlow::Poll);
	}
}
