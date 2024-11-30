use raw_window_handle::{
	DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use std::sync::{Arc, Mutex};

pub trait HasWindowDisplayHandle: HasWindowHandle + HasDisplayHandle {}

impl<T: HasWindowHandle + HasDisplayHandle> HasWindowDisplayHandle for T {}

/// this struct iam to simplify conversion of HasWindowHandle + HasDisplayHandle into one for vulkano (because lifetime)
pub struct agnosticHandle
{
	_inner: Arc<Mutex<Box<dyn HasWindowDisplayHandle>>>,
}

impl agnosticHandle
{
	pub fn newFromHandle(handle: impl HasWindowDisplayHandle + 'static) -> Self
	{
		Self {
			_inner: Arc::new(Mutex::new(Box::new(handle))),
		}
	}
}

//unsafe impl Send for agnosticHandle {}
//unsafe impl Sync for agnosticHandle {}

impl HasDisplayHandle for agnosticHandle
{
	fn display_handle(&self) -> Result<DisplayHandle, HandleError>
	{
		self._inner.lock().unwrap().display_handle()
	}
}

impl HasWindowHandle for agnosticHandle
{
	fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError>
	{
		self._inner.lock().unwrap().window_handle()
	}
}
