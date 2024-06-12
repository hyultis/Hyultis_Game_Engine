use std::any::Any;
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::{Duration, Instant};
use cgmath::num_traits::Pow;
use crate::ManagerAnimation::AnimationHolder;
use HArcMut::HArcMut;
use Htrace::TSpawner;

enum AnimationRepeat
{
    /// optionnaly on last progress, call the function
    NOREPEAT(Option<Arc<dyn Fn() + Sync + Send>>),
    /// repeat indefinitely
    REPEAT,
    /// like repeat, be each other is mirrored
    MIRROR,
    /// repeat x times
    REPEAT_TIME(u32,Option<Arc<dyn Fn() + Sync + Send>>)
}

pub struct Animation<A, B = A>
	where A: Clone + Send + Sync
{
    _startTime: Instant,
    _duration: Duration,
	_durationu128: u128,
    _repeat: AnimationRepeat,
    pub source: HArcMut<A>,
    pub startState: B,
    pub endState: B,
    _fnTick: Box<dyn Fn(&Animation<A, B>,f32) + Sync + Send>,
    _fnEnd: Option<Arc<dyn Fn() + Sync + Send>>,
	_isEnd: bool
}

impl<A, B> Animation<A, B>
	where A: Clone + Send + Sync
{
    pub fn new(duration: Duration, source: HArcMut<A>, startState: B, endState: B, fnTick: impl Fn(&Animation<A, B>,f32) + Sync + Send + 'static) -> Self
    {
        return Animation{
            _startTime: Instant::now(),
	        _durationu128: duration.as_nanos(),
            _duration: duration,
	        _repeat: AnimationRepeat::NOREPEAT(None),
            startState,
            source,
            endState,
            _fnTick: Box::new(fnTick),
            _fnEnd: None,
	        _isEnd: false,
        };
    }

    /// set NOREPEAT MODE for this animation (default)
    /// a function called at the end can be set
    pub fn setModeNoRepeat(&mut self, func: Option<impl Fn() + Sync + Send + 'static>)
    {
        self._repeat = match func {
            None => AnimationRepeat::NOREPEAT(None),
            Some(func) => {
                AnimationRepeat::NOREPEAT(Some(Arc::new(func)))
            }
        };
    }

    /// set REPEAT_TIME MODE for this animation
    /// a function called at the end (after "nb") can be set
    pub fn setModeRepeatXTime(&mut self, nb: u32, func: Option<impl Fn() + Sync + Send + 'static>)
    {
        self._repeat = match func {
            None => AnimationRepeat::REPEAT_TIME(nb,None),
            Some(func) => {
                AnimationRepeat::REPEAT_TIME(nb, Some(Arc::new(func)))
            }
        };
    }

    /// set REPEAT MODE for this animation
    pub fn setModeRepeat(&mut self)
    {
        self._repeat = AnimationRepeat::REPEAT;
    }

    /// set MIRROR MODE for this animation
    pub fn setModeMirror(&mut self)
    {
        self._repeat = AnimationRepeat::MIRROR;
    }
	
	/// change duration, reset start timer
	pub fn setDuration(&mut self,duration: Duration)
	{
		self._duration = duration;
		self._startTime = Instant::now();
	}

    /// run a tick, calculate time progression.
    /// if MODE is NOREPEAT or REPEAT_TIME, it will return true if the end time its reached.
    /// other MODE will always return false
    pub fn tick(&mut self) -> bool
    {
	    if(self._isEnd)
	    {
		    return true;
	    }
	    
        let now = Instant::now();
        let durationFromStart = now.duration_since(self._startTime).as_nanos();
	    let mut endcaller = None;
	    
        let progress = {
            if(self._durationu128==0)
            {
                1.0
            }
            else if(durationFromStart>self._durationu128)
            {
                match &self._repeat {
                    AnimationRepeat::NOREPEAT(func) => {
	                    if(func.is_some())
	                    {
		                    endcaller = Some(func.clone().unwrap().clone());
	                    }
	                    self._isEnd = true;
                        1.0
                    }
                    AnimationRepeat::REPEAT => {
                        let loopremain = durationFromStart%self._durationu128;
                        loopremain as f32 / self._durationu128 as f32
                    }
                    AnimationRepeat::MIRROR => {
                        let loopremain = durationFromStart%self._durationu128;
                        loopremain as f32 / self._durationu128 as f32
                    }
                    AnimationRepeat::REPEAT_TIME(nb,func) => {
                        let looped = (durationFromStart/self._durationu128) as u32;
                        if(looped > *nb)
                        {
	                        if(func.is_some())
	                        {
		                        endcaller = Some(func.clone().unwrap());
	                        }
	                        self._isEnd = true;
                            1.0
                        }
                        else
                        {
                            let loopremain = durationFromStart%self._durationu128;
                            loopremain as f32 / self._durationu128 as f32
                        }
                    }
                }
            }
            else
            {
                durationFromStart as f32 / self._durationu128 as f32
            }
        };

        let tmpfn = &*self._fnTick;
        tmpfn(&self, progress);
	    
	    if let Some(func) = endcaller
	    {
		    let _ = TSpawner!(||{
			   func();
		    });
	    }
	    
	    return self._isEnd;
    }
}

impl<A> Animation<A, A>
	where A: Clone + Send + Sync
{
	pub fn newFromSource(duration: Duration, source: HArcMut<A>, endState: A, fnTick: impl Fn(&Animation<A, A>, f32) + Sync + Send + 'static) -> Self
	{
		return Animation {
			_startTime: Instant::now(),
			_durationu128: duration.as_nanos(),
			_duration: duration,
			_repeat: AnimationRepeat::NOREPEAT(None),
			//startState: {let x = source.get().clone();x},
			startState: {let x: A = (**source.get()).clone();x},
			source,
			endState,
			_fnTick: Box::new(fnTick),
			_fnEnd: None,
			_isEnd: false,
		};
	}
}

impl<A, B> AnimationHolder for Animation<A, B>
    where A: Clone + Send + Sync + 'static,
          B: Send + Sync + 'static
{
    fn ticks(&mut self) -> bool {
        self.tick()
    }
	
	fn checkDrop(&mut self) -> bool
	{
		self.source.isWantDrop()
	}
	fn as_any(&self) -> &dyn Any {
		self
	}
	
	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}
}


pub struct AnimationUtils
{

}

impl AnimationUtils
{
    /// linear animation
    pub fn linear(start: f32, end: f32, mut progress: f32) -> f32
    {
        progress = progress.clamp(0.0,1.0);
        return AnimationUtils::internal_linear(start,end,progress);
    }

    /// multiply progress by itself, resulting in slow start, fast ending animation
    /// pow need to be at least 2 to take effect, and result in slower start
    pub fn pow(start: f32, end: f32, mut progress: f32, pow: u16) -> f32
    {
        progress = progress.clamp(0.0,1.0);
        return AnimationUtils::internal_linear(start, end, progress.pow(pow));
    }

    /// inverse of pow
    /// iter need to be at least 1 to take effect, and result in faster start
    /// use sqrt, this it slow !
    pub fn sqrt(start: f32, end: f32, mut progress: f32, iter: u16) -> f32
    {
        progress = progress.clamp(0.0,1.0);
        let mut finalprogress = progress;
        for _ in 1..iter {
            finalprogress = finalprogress.sqrt();
        }
        return AnimationUtils::internal_linear(start, end, progress.sqrt());
    }

    /// slow start and end, but faster middle
    /// https://en.wikipedia.org/wiki/Smoothstep
    pub fn smoothstep(start: f32, end: f32, mut progress: f32) -> f32
    {
        progress = progress.clamp(0.0,1.0);
        let tmp = progress*progress*(3.0-2.0*progress);
        return AnimationUtils::internal_linear(start, end, tmp);
    }

    /// parabola animation, end is on center of animation and return to start after
    /// intensity >=2 = smooth parabola (more is reducing the duration of the "center")
    /// intensity 1 (min) = rude parabola
    pub fn parabola(start: f32, end: f32, mut progress: f32, mut intensity: u16) -> f32
    {
        progress = progress.clamp(0.0,1.0);
        intensity = intensity.min(1);
        let finalprogress = ( 4.0*progress*(1.0-progress)).pow(intensity);
        let finalprogress = finalprogress.pow(intensity);
        return AnimationUtils::internal_linear(start, end, finalprogress);
    }

    /// elastic animation, vibrating at start, stopping at end
    pub fn elastic(start: f32, end: f32, mut progress: f32) -> f32
    {
        progress = progress.clamp(0.0,1.0);
        let tmp = (-29.0 * (progress+1.0) * PI/2.0).sin(); // normally -13.0 ?
        let tmp = tmp * (2.0 as f32).pow(-10.0*progress)+1.0;
        return AnimationUtils::internal_linear(start, end, tmp);
    }

    /////// PRIVATE ///////////

    fn internal_linear(start: f32, end: f32, progress: f32) -> f32
    {
        return (end*progress) + (start*(1.0-progress))
    }
}
