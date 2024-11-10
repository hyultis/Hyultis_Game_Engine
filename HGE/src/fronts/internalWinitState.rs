use Htrace::HTraceError;
use vulkano::swapchain::Surface;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;
use crate::configs::general::HGEconfig_general;
use crate::fronts::UserDefinedEventOverride::UserDefinedEventOverride;
use crate::fronts::winit::HGEwinit;
use crate::HGEMain::HGEMain;

pub struct internalWinitState<'a> {
	initialized: bool,
	config: HGEconfig_general,
	events: Option<Box<&'a mut dyn UserDefinedEventOverride>>
}

impl<'a> internalWinitState<'a>
{
	pub fn new(config: HGEconfig_general, userEvents: Option<&'a mut impl UserDefinedEventOverride>) -> Self
	{
		
		Self{
			initialized: false,
			config,
			events: userEvents.map(|e| Box::new(e as &'a mut dyn UserDefinedEventOverride)),
		}
	}
}


impl<'a> ApplicationHandler<()> for internalWinitState<'a> {
	fn resumed(&mut self, eventloop: &ActiveEventLoop) {
		if(!self.initialized)
		{
			let preinit = HGEMain::preinitialize(self.config.clone());
			let window = match HGEwinit::buildAgnosticWindow(eventloop) {
				Ok(x) => x,
				Err(err) => { panic!("cannot get window from winit : {}", err); }
			};
			HTraceError!(HGEMain::initialize(Surface::required_extensions(eventloop),window,preinit));
			self.initialized = true;
			
			let func = &mut *HGEwinit::funcPostInit_get_mut();
			func();
		}
		else
		{
			let window = match HGEwinit::buildAgnosticWindow(eventloop) {
				Ok(x) => x,
				Err(err) => { panic!("cannot get window from winit : {}", err); }
			};
			HTraceError!(HGEMain::singleton().engineResumed(window));
			
			if let Some(tmp) = &mut self.events
			{
				tmp.resumed(eventloop);
			}
		}
		
	}
	
	fn suspended(&mut self, eventloop: &ActiveEventLoop) {
		if(!self.initialized)
		{
			return;
		}
		
		HGEMain::singleton().engineSuspended();
		
		if let Some(tmp) = &mut self.events
		{
			tmp.suspended(eventloop);
		}
	}
	
	fn window_event(&mut self, eventloop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
		if(!self.initialized)
		{
			return;
		}
		
		match &event
		{
			WindowEvent::KeyboardInput {
				event: input,
				..
			} => {
				//println!("key input : {:?}",input);
				if let PhysicalKey::Code(key) = input.physical_key
				{
					let mut inputsC = HGEwinit::inputs_get_mut();
					inputsC.updateFromKeyboard(key, input.state);
				}
			},
			WindowEvent::Resized(winsize) =>
			{
				let mut width = winsize.width.max(1);
				if (width > 7680)
				{
					width = 1;
				}
				let mut height = winsize.height.max(1);
				if (height > 4320)
				{
					height = 1;
				}
				
				if(cfg!(target_os = "android"))
				{
					HGEMain::singleton().setWindowHDPI((1080.0 / height as f32).min(1.0));
				}
				
				HGEMain::singleton().window_resize(Some([width, height]));
			},
			WindowEvent::CloseRequested =>
			{
				eventloop.exit();
			},
			WindowEvent::Destroyed =>
			{
				let func = &mut *HGEwinit::funcPreExit_get_mut();
				func();
				eventloop.exit();
			},
			WindowEvent::RedrawRequested => //problem with windows ? https://github.com/rust-windowing/winit/pull/3950
			//https://github.com/rust-windowing/winit/issues/3272
			{
				if let Some(tmp) = &mut self.events
				{
					tmp.about_to_render(eventloop);
				}
				
				HGEMain::singleton().runRendering(|| {
					HGEwinit::getWindow(|window| {
						window.pre_present_notify();
					});
				});
			}
			_ => ()
		}
		
		if(event!=WindowEvent::RedrawRequested)
		{
			if let Some(tmp) = &mut self.events
			{
				tmp.window_event(eventloop, &event, window_id);
			}
		}
	}
	
	fn device_event(&mut self, eventloop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
		if let Some(tmp) = &mut self.events
		{
			tmp.device_event(eventloop, &event, device_id);
		}
	}
	
	fn about_to_wait(&mut self, eventloop: &ActiveEventLoop) {
		if(!self.initialized)
		{
			return;
		}
		
		HGEMain::singleton().runService();
		HGEwinit::getWindow(|window| {
			window.request_redraw();
		});
		
		if let Some(tmp) = &mut self.events
		{
			tmp.about_to_wait(eventloop);
		}
		else
		{
			if (HGEwinit::inputs_get_mut().getKeyboardStateAndSteal(KeyCode::Escape) == ElementState::Pressed)
			{
				eventloop.exit();
			}
		}
		
		eventloop.set_control_flow(ControlFlow::Poll);
	}
}
