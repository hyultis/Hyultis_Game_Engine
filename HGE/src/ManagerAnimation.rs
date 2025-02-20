use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use singletonThread::SingletonThread;
use std::any::Any;
use std::sync::OnceLock;
use Htrace::HTracer::HTracer;

pub trait AnimationHolder: Send + Sync
{
	fn ticks(&mut self) -> bool;
	fn checkDrop(&mut self) -> bool;

	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct ManagerAnimation
{
	_maxid: RwLock<usize>,
	_animations: DashMap<usize, Box<dyn AnimationHolder>>,
	_threadLoading: Mutex<SingletonThread>,
	_threadDrop: Mutex<SingletonThread>,
}

static SINGLETON: OnceLock<ManagerAnimation> = OnceLock::new();

impl ManagerAnimation
{
	fn new() -> Self
	{
		let mut singThread = SingletonThread::new(|| {
			HTracer::threadSetName("ManagerAnimation");
			ManagerAnimation::singleton().internal_ticks();
		});
		singThread.setThreadName("animation");

		let mut singThreadDrop = SingletonThread::new(|| {
			HTracer::threadSetName("ManagerAnimation");
			ManagerAnimation::singleton()._animations.retain(|_, animation| {
				return !animation.checkDrop(); // return true for need to drop, so we need to return false for "no retain it"
			});
		});
		singThreadDrop.setThreadName("animation");

		return ManagerAnimation {
			_maxid: RwLock::new(0),
			_animations: DashMap::default(),
			_threadLoading: Mutex::new(singThread),
			_threadDrop: Mutex::new(singThreadDrop),
		};
	}

	pub fn singleton() -> &'static ManagerAnimation
	{
		return SINGLETON.get_or_init(|| ManagerAnimation::new());
	}

	pub fn append(&self, newAnim: impl AnimationHolder + 'static) -> usize
	{
		let mut old = self._maxid.write();
		*old += 1;
		self._animations.insert(*old, Box::new(newAnim));
		return *old;
	}

	pub fn remove(&self, id: usize)
	{
		self._animations.remove(&id);
	}

	pub fn replace(&self, id: usize, newAnim: impl AnimationHolder + 'static)
	{
		let newAnim = Box::new(newAnim);
		match self._animations.get_mut(&id)
		{
			None =>
			{
				self._animations.insert(id, newAnim);
			}
			Some(mut old) => *old.value_mut() = newAnim,
		};
	}

	pub fn anim_mut(&self) -> &DashMap<usize, Box<dyn AnimationHolder>>
	{
		&self._animations
	}

	pub fn ticksAll(&self)
	{
		if let Some(mut t) = self._threadLoading.try_lock()
		{
			t.thread_launch();
		}
		if let Some(mut t) = self._threadDrop.try_lock()
		{
			t.thread_launch();
		}
	}

	fn internal_ticks(&self)
	{
		self._animations.retain(|_, animation| {
			let returned = !animation.ticks();
			return returned;
		});
	}
}
