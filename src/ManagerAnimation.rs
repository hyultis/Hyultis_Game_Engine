use std::any::Any;
use std::sync::{Arc, OnceLock};
use dashmap::DashMap;
use Htrace::HTracer::HTracer;
use parking_lot::{Mutex, RwLock};
use singletonThread::SingletonThread;

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
    _animations: Arc<DashMap<usize,Box<dyn AnimationHolder>>>,
	_threadLoading: Mutex<SingletonThread>,
	_threadDrop: Mutex<SingletonThread>
}

static SINGLETON: OnceLock<ManagerAnimation> = OnceLock::new();

impl ManagerAnimation
{
    fn new() -> Self {
	    let mut singThread = SingletonThread::new(||{
		    HTracer::threadSetName("ManagerAnimation");
		    ManagerAnimation::singleton().internal_ticks();
	    });
	    singThread.setDuration_FPS(120);
	    singThread.setThreadName("animation");
	    singThread.setLoop(true);
	    
	    let mut singThreadDrop = SingletonThread::new(||{
		    HTracer::threadSetName("ManagerAnimation");
		    ManagerAnimation::singleton()._animations.clone().retain(|_,animation|{
			    return !animation.checkDrop(); // return true for need to drop, so we need to return false for "no retain it"
		    });
	    });
	    singThreadDrop.setThreadName("animation");
	    singThreadDrop.setDuration_FPS(1);
	    
        return ManagerAnimation {
	        _maxid: RwLock::new(0),
	        _animations: Arc::new(DashMap::default()),
	        _threadLoading: Mutex::new(singThread),
	        _threadDrop: Mutex::new(singThreadDrop)
        };
    }

    pub fn singleton() -> &'static ManagerAnimation
    {
        return SINGLETON.get_or_init(|| {
            ManagerAnimation::new()
        });
    }

    pub fn append(&self, newAnim: impl AnimationHolder + 'static) -> usize
    {
	    let mut old = self._maxid.write();
	    *old += 1;
        self._animations.insert( *old,Box::new(newAnim));
	    return *old;
    }
	
	pub fn remove(&self, id: usize)
	{
		self._animations.remove(&id);
	}
	
	pub fn replace(&self, id: usize, newAnim: impl AnimationHolder + 'static)
	{
		let newAnim = Box::new(newAnim);
		match self._animations.get_mut(&id) {
			None => {self._animations.insert( id,newAnim);},
			Some(mut old) => *old.value_mut() = newAnim
		};
	}
	
	pub fn anim_mut(&self) -> &DashMap<usize, Box<dyn AnimationHolder>>
	{
		&self._animations
	}

    pub fn ticksAll(&self)
    {
	    self._threadLoading.lock().thread_launch();
	    self._threadDrop.lock().thread_launch();
    }
	
	fn internal_ticks(&self)
	{
		self._animations.clone().retain(|_,animation|{
			return !animation.ticks();
		});
	}
}
