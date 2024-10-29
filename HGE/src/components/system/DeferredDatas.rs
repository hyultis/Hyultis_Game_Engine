use parking_lot::lock_api::RwLockWriteGuard;
use parking_lot::{RawRwLock, RwLock};
use singletonThread::SingletonThread;
use std::sync::Arc;

/**
 * this aim to transfer data between thread, without any latence on output thread (with steal())
 * it use a SingletonThread and ArcSwapOption, to transfer data
 */
pub struct DeferredData<T>
	where
		T: Send + Sync + 'static
{
	_input: Arc<RwLock<Option<T>>>,
	_output: Arc<RwLock<Option<T>>>,
	_thread: RwLock<SingletonThread>
}

impl<T> DeferredData<T>
	where
		T: Send + Sync + 'static
{
	pub fn new() -> Self
	{
		let inputArc = Arc::new(RwLock::new(None));
		let outputArc = Arc::new(RwLock::new(None));
		
		let copyArcInput = inputArc.clone();
		let copyArcOutput = outputArc.clone();
		let thread = SingletonThread::new(move || {
			if (!copyArcOutput.clone().read().is_none()) // if output is already full, we are doing nothing
			{
				return;
			}
			
			let Some(tmp) = copyArcInput.clone().write().take() else { return };
			*copyArcOutput.clone().write() = Some(tmp);
		});
		
		Self {
			_input: inputArc,
			_output: outputArc,
			_thread: RwLock::new(thread),
		}
	}
	
	/**
	 * force a manual transfer, beware, it overrides the output data, even if input is none
	 */
	pub fn force_transfer(&self)
	{
		match self._input.write().take()
		{
			None => *self._output.write() = None,
			Some(tmp) => *self._output.write() = Some(tmp)
		}
	}
	
	pub fn thread_launch(&self)
	{
		self._thread.write().thread_launch();
	}
	
	/**
	 * allow to update the Option<data>
	 */
	pub fn inputMut(&self) -> RwLockWriteGuard<'_, RawRwLock, Option<T>>
	{
		return self._input.write();
	}
	
	/**
	 * steal the output data, setting it to none, allowing to the next thread_launch to override it
	 */
	pub fn steal(&self) -> Option<T>
	{
		return self._output.write().take();
	}
}
