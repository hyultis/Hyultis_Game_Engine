use raw_window_handle::{
	DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};

pub trait HasWindowDisplayHandle: HasWindowHandle + HasDisplayHandle {}

impl<T: HasWindowHandle + HasDisplayHandle> HasWindowDisplayHandle for T {}

/// this struct iam to simplify conversion of HasWindowHandle + HasDisplayHandle into one for vulkano (because lifetime)
pub struct agnosticHandle
{
	_inner: Box<dyn HasWindowDisplayHandle>,
}

impl agnosticHandle
{
	pub fn newFromHandle(handle: impl HasWindowDisplayHandle + 'static) -> Self
	{
		Self {
			_inner: Box::new(handle),
		}
	}
}

unsafe impl Send for agnosticHandle {}
unsafe impl Sync for agnosticHandle {}

impl HasDisplayHandle for agnosticHandle
{
	fn display_handle(&self) -> Result<DisplayHandle, HandleError>
	{
		self._inner.display_handle()
	}
}

impl HasWindowHandle for agnosticHandle
{
	fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError>
	{
		self._inner.window_handle()
	}
}
