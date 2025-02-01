use singletonThread::SingletonThread;
use std::sync::{Arc, Mutex, RwLock, RwLockWriteGuard};

/**
 * this aim to transfer data between thread, without any latence on output thread (with steal())
 * it use a SingletonThread and ArcSwapOption, to transfer data
 */
pub struct DeferredData<T>
where
	T: Send + Sync + 'static,
{
	_input: Arc<RwLock<Option<T>>>,
	_output: Arc<Mutex<Option<T>>>,
	_thread: Mutex<SingletonThread>,
}

impl<T> DeferredData<T>
where
	T: Send + Sync + 'static,
{
	pub fn new() -> Self
	{
		let inputArc = Arc::new(RwLock::new(None));
		let outputArc = Arc::new(Mutex::new(None));

		let copyArcInput = inputArc.clone();
		let copyArcOutput = outputArc.clone();
		let thread = SingletonThread::new(move || {
			let bindingOutput = copyArcOutput.clone();
			let Ok(mut outputlock) = bindingOutput.try_lock()
			else
			{
				return;
			};

			// if output is already full, we are doing nothing
			if (!outputlock.is_none())
			{
				return;
			}

			let bindingInput = copyArcInput.clone();
			let Ok(mut inputlock) = bindingInput.try_write()
			else
			{
				return;
			};

			let Some(tmp) = inputlock.take()
			else
			{
				return;
			};
			*outputlock = Some(tmp);
		});

		Self {
			_input: inputArc,
			_output: outputArc,
			_thread: Mutex::new(thread),
		}
	}

	/**
	 * force a manual transfer, beware, it overrides the output data, even if input is none
	 */
	pub fn force_transfer(&self)
	{
		match self._input.write().unwrap().take()
		{
			None => *self._output.lock().unwrap() = None,
			Some(tmp) => *self._output.lock().unwrap() = Some(tmp),
		}
	}

	pub fn thread_launch(&self)
	{
		if let Ok(mut t) = self._thread.try_lock()
		{
			t.thread_launch();
		}
	}

	/**
	 * allow to update the Option<data>
	 * waiting for input to be writable
	 */
	pub fn inputMut(&self) -> RwLockWriteGuard<Option<T>>
	{
		return self._input.write().unwrap();
	}

	/**
	 * steal the output data, setting it to none, allowing to the next thread_launch to override it
	 */
	pub fn steal(&self) -> Option<T>
	{
		return match self._output.try_lock()
		{
			Ok(mut e) => e.take(),
			Err(_) => None,
		};
	}
}
