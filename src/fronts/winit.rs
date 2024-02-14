use std::sync::{Arc, OnceLock};
use arc_swap::ArcSwapOption;
use Htrace::HTraceError;
use parking_lot::RwLock;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use crate::configs::general::HGEconfig_general;
use crate::fronts::Inputs::Inputs;
use crate::HGEMain::HGEMain;

pub struct HGEwinit
{
	_funcPostInit: RwLock<Box<dyn FnMut() + Send + Sync>>,
	_funcPostEngineEvent: RwLock<Box<dyn FnMut(&Event<()>) + Send + Sync>>,
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
	
	pub fn setFunc_PostInit(&self, func: impl FnMut() + Send + Sync + 'static)
	{
		*self._funcPostInit.write() = Box::new(func);
	}
	
	pub fn setFunc_PostEngineEvent(&self, func: impl FnMut(&Event<()>) + Send + Sync + 'static)
	{
		*self._funcPostEngineEvent.write() = Box::new(func);
	}
	
	pub fn run(eventloop: EventLoop<()>, generalConf: HGEconfig_general)
	{
		let mut initialized = false;
		
		let _ = eventloop.run(move |event, eventloop|
			{
				if (!initialized)
				{
					if event == Event::Resumed
					{
						HTraceError!(HGEMain::singleton().engineInitialize(eventloop,generalConf.clone()));
						Self::singleton();
						initialized = true;
						
						let func = &mut *Self::singleton()._funcPostInit.write();
						func();
					}
				}
				else if HGEMain::singleton().engineIsSuspended()
				{
					if event ==	Event::Resumed
					{
						HTraceError!(HGEMain::singleton().engineResumed(eventloop));
					}
				}
				else
				{
					match &event
					{
						Event::WindowEvent {
							event: WindowEvent::KeyboardInput {
								event: input,
								..
							}, ..
						} => {
							//println!("key input : {:?}",input);
							if let PhysicalKey::Code(key) = input.physical_key
							{
								let mut inputsC = Self::singleton()._inputsC.write();
								inputsC.updateFromKeyboard(key, input.state);
							}
						},
						Event::WindowEvent {
							event: WindowEvent::Resized(winsize), ..
						} => {
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
							
							#[cfg(target_os = "android")]
							{
								Self::singleton().setWindowHDPI((1080.0 / height as f32).min(1.0));
							}
							
							HGEMain::singleton().window_resize(Some(width), Some(height));
						},
						Event::Suspended => {
							HGEMain::singleton().engineSuspended();
						},
						Event::WindowEvent {
							event: WindowEvent::CloseRequested,
							.. // window_id
						} => {
							eventloop.exit();
						},
						_ => ()
					}
					
					{
						let func = &mut *Self::singleton()._funcPostEngineEvent.write();
						func(&event);
					}
					
					if event == Event::AboutToWait
					{
						println!("fps : {}",HGEMain::singleton().getTimer().getFps());
						if (Self::singleton()._inputsC.write().getKeyboardStateAndSteal(KeyCode::Escape) == ElementState::Pressed)
						{
							eventloop.exit();
						}
						
						HGEMain::singleton().runService();
						HGEMain::singleton().runRendering();
						eventloop.set_control_flow(ControlFlow::Poll);
					}
				}
			});
	}
	
	//////////////////// PRIVATE /////////////////
	
	fn new() -> Self
	{
		Self{
			_funcPostInit: RwLock::new(Box::new(||{})),
			_funcPostEngineEvent: RwLock::new(Box::new(|_|{})),
			_inputsC: RwLock::new(Inputs::new()),
		}
	}
}
