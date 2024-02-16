use std::sync::OnceLock;
use Htrace::HTraceError;
use parking_lot::{RawRwLock, RwLock};
use parking_lot::lock_api::{RwLockReadGuard, RwLockWriteGuard};
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;
use crate::configs::general::HGEconfig_general;
use crate::fronts::Inputs::Inputs;
use crate::HGEMain::HGEMain;

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
	pub fn run(eventloop: EventLoop<()>, generalConf: HGEconfig_general, postEngineEvent: &mut impl FnMut(&Event<()>,&EventLoopWindowTarget<()>))
	{
		let mut initialized = false;
		
		let _ = eventloop.run(move |event, eventloop|
			{
				if (!initialized)
				{
					if event == Event::Resumed
					{
						HTraceError!(HGEMain::initialize(eventloop,generalConf.clone()));
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
							
							HGEMain::singleton().window_resize(Some([width,height]));
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
						Event::WindowEvent {
							event: WindowEvent::Destroyed,
							.. // window_id
						} => {
							let func = &mut *Self::singleton()._funcPre_exit.write();
							func();
							eventloop.exit();
						},
						_ => ()
					}
					
					postEngineEvent(&event,eventloop);
					
					match event
					{
						Event::WindowEvent {
							event: WindowEvent::RedrawRequested,
							..
						} => {
							HGEMain::singleton().runRendering();
							
							if let Some(surfaceBinding) = &*HGEMain::singleton().getSurface()
							{
								if let Some(window) = surfaceBinding.object().unwrap().downcast_ref::<Window>()
								{
									window.request_redraw();
								}
							}
						},
						_ => ()
					}
					
					if event == Event::AboutToWait
					{
						if (Self::singleton()._inputsC.write().getKeyboardStateAndSteal(KeyCode::Escape) == ElementState::Pressed)
						{
							eventloop.exit();
						}
						
						HGEMain::singleton().runService();
						eventloop.set_control_flow(ControlFlow::Poll);
					}
				}
			});
	}
	
	pub fn getWindow<F>(&self, func: F)
		where F: FnOnce(&Window)
	{
		let surfaceBinding = HGEMain::singleton().getSurface();
		if let Some(surface) = &*surfaceBinding
		{
			let tmp = surface.object().unwrap().downcast_ref::<Window>().unwrap();
			func(tmp);
		}
	}
	
	//////////////////// PRIVATE /////////////////
	
	fn new() -> Self
	{
		Self{
			_funcPostInit: RwLock::new(Box::new(||{})),
			_funcPre_exit: RwLock::new(Box::new(||{})),
			_inputsC: RwLock::new(Inputs::new()),
		}
	}
}
