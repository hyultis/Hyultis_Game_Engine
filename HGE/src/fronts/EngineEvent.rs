use crate::configs::general::HGEconfig_general;
use crate::HGEMain::{preinit, HGEMain};
use anyhow::anyhow;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::any::Any;
use std::sync::Arc;
use vulkano::swapchain::Surface;
use Htrace::HTraceError;

pub struct EngineEvent
{
	_funcPostInit: Box<dyn FnMut() + Send + Sync>,
	_funcPreExit: Box<dyn FnMut() + Send + Sync>,
	_initialized: bool,
	config: HGEconfig_general,
}

impl EngineEvent
{
	pub fn new() -> EngineEvent
	{
		EngineEvent {
			_funcPostInit: Box::new(|| {}),
			_funcPreExit: Box::new(|| {}),
			_initialized: false,
			config: HGEconfig_general::default(),
		}
	}

	pub fn isInitialized(&self) -> bool
	{
		self._initialized
	}

	pub fn setConfig(&mut self, config: HGEconfig_general)
	{
		self.config = config;
	}

	/// change func called after engine initialization
	pub fn setFunc_PostInit(&mut self, func: impl FnMut() + Send + Sync + 'static)
	{
		self._funcPostInit = Box::new(func);
	}

	/// change func called before exit
	pub fn setFunc_PreExit(&mut self, func: impl FnMut() + Send + Sync + 'static)
	{
		self._funcPreExit = Box::new(func);
	}

	/// HGE action when suspend
	/// return false if engine is not initialized
	pub fn suspend(&mut self) -> bool
	{
		if (!self._initialized)
		{
			return false;
		}

		HGEMain::singleton().engineSuspended();
		return true;
	}

	pub fn preInit(&mut self) -> anyhow::Result<preinit>
	{
		if (self._initialized)
		{
			return Err(anyhow!("already initialized"));
		}

		return HGEMain::preinitialize(self.config.clone());
	}

	pub fn init(
		&mut self,
		window: Arc<impl HasWindowHandle + HasDisplayHandle + Any + Send + Sync>,
		preinit: anyhow::Result<preinit>,
	)
	{
		if (self._initialized)
		{
			return;
		}

		HTraceError!(HGEMain::initialize(
			Surface::required_extensions(&window.display_handle().unwrap()).unwrap(),
			window,
			preinit
		));
		self._initialized = true;
		let func = &mut self._funcPostInit;
		func();
	}

	/// HGE action when resume (or first launch)
	/// return true if initialized just happened
	pub fn resume(
		&mut self,
		window: Arc<impl HasWindowHandle + HasDisplayHandle + Any + Send + Sync>,
	)
	{
		if (!self._initialized)
		{
			return;
		}

		HTraceError!(HGEMain::singleton().engineResumed(window));
	}

	/// when window resize
	pub fn window_eventResize(&mut self, mut width: u32, mut height: u32)
	{
		if (!self._initialized)
		{
			return;
		}

		if (width > 7680)
		{
			width = 1;
		}
		if (height > 4320)
		{
			height = 1;
		}

		if (cfg!(target_os = "android"))
		{
			HGEMain::singleton().setWindowHDPI((1080.0 / height as f32).min(1.0));
		}

		HGEMain::singleton().window_resize(Some([width, height]));
	}

	/// when exiting game
	pub fn window_eventClose(&mut self)
	{
		if (!self._initialized)
		{
			return;
		}

		let func = &mut self._funcPreExit;
		func();
	}

	/**
	 * @param func : function called just before image swap
	 */
	pub fn window_draw(&mut self, func: impl Fn())
	{
		if (!self._initialized)
		{
			return;
		}

		HGEMain::singleton().runRendering(|| {
			func();
		});
	}

	/// when engine have time to launch internal service
	pub fn runService(&mut self)
	{
		if (!self._initialized)
		{
			return;
		}

		HGEMain::singleton().runService();
	}
}
