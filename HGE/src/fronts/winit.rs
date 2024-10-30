pub extern crate winit;

use crate::components::system::TimeStats::TimeStatsStorage;
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
use vulkano::swapchain::Surface;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window};
use Hconfig::serde_json::Value as JsonValue;
use Hconfig::HConfigManager::HConfigManager;
use Htrace::{HTrace, HTraceError};

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
	pub fn run(eventloop: EventLoop<()>, generalConf: HGEconfig_general, postEngineEvent: &mut impl FnMut(&Event<()>, &ActiveEventLoop))
	{
		let mut initialized = false;
		
		let _ = eventloop.run(move |event, eventloop|
			{
				if (!initialized)
				{
					if event == Event::Resumed
					{
						let preinit = HGEMain::preinitialize(generalConf.clone());
						let window = match Self::buildAgnosticWindow(eventloop) {
							Ok(x) => x,
							Err(err) => { panic!("cannot get window from winit : {}", err); }
						};
						HTraceError!(HGEMain::initialize(Surface::required_extensions(eventloop),window,preinit));
						initialized = true;
						
						let func = &mut *Self::singleton()._funcPostInit.write();
						func();
					}
				} else if HGEMain::singleton().engineIsSuspended()
				{
					if event == Event::Resumed
					{
						let window = match Self::buildAgnosticWindow(eventloop) {
							Ok(x) => x,
							Err(err) => { panic!("cannot get window from winit : {}", err); }
						};
						HTraceError!(HGEMain::singleton().engineResumed(window));
					}
				} else {
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
							
							HGEMain::singleton().window_resize(Some([width, height]));
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
					
					postEngineEvent(&event, eventloop);
					
					match event
					{
						Event::WindowEvent {
							event: WindowEvent::RedrawRequested,
							..
						} => {
							HGEMain::singleton().runRendering(|| {
								if let Some(surfaceBinding) = &*HGEMain::singleton().getSurface()
								{
									if let Some(window) = surfaceBinding.object().unwrap().downcast_ref::<Window>()
									{
										window.pre_present_notify();
									}
								}
							});
							
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
		where
			F: FnOnce(&Window)
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
		Self {
			_funcPostInit: RwLock::new(Box::new(|| {})),
			_funcPre_exit: RwLock::new(Box::new(|| {})),
			_inputsC: RwLock::new(Inputs::new()),
		}
	}
	
	fn buildAgnosticWindow(eventloop: &ActiveEventLoop) -> anyhow::Result<impl HasRawWindowHandle + HasRawDisplayHandle + Any + Send + Sync>
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
