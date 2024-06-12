use std::time::{Duration, Instant};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum InterpolateTimer_type
{
	CLAMP,
	REPEAT
}

#[derive(Copy, Clone)]
pub struct InterpolateTimer
{
	_startVal: f32,
	_endVal: f32,
	_duration: u128,
	_startTime: Instant,
	_type: InterpolateTimer_type
}

impl InterpolateTimer
{
	pub fn getValueInterpolated(&self) -> f32
	{
		let mut durationFromStart = Instant::now().duration_since(self._startTime).as_nanos();
		match self._type {
			InterpolateTimer_type::CLAMP => {
				durationFromStart = durationFromStart.clamp(0,self._duration);
			}
			InterpolateTimer_type::REPEAT => {
				durationFromStart = durationFromStart%self._duration;
			}
		}
		
		let valdistance = self._endVal-self._startVal;
		let percent = durationFromStart as f32/self._duration as f32;
		return valdistance*percent;
	}
}

pub struct ManagerInterpolate
{
	_lastTime: Instant,
	_now: Instant,
}

impl ManagerInterpolate
{
	pub fn new() -> ManagerInterpolate
	{
		return ManagerInterpolate {
			_lastTime: Instant::now(),
			_now: Instant::now(),
		};
	}
	
	pub fn update(&mut self)
	{
		self._lastTime = self._now;
		self._now = Instant::now();
	}
	
	pub fn getNowFromLast(&self) -> Duration
	{
		return Instant::now().duration_since(self._now);
	}
	
	pub fn getFps(&self) -> u32
	{
		let nanos = self._now.duration_since(self._lastTime).as_nanos();
		return (f64::from(1_000_000_000.0) / nanos as f64) as u32;
	}
	
	pub fn getInterpolatedValue(&self, value: f32, duration: Duration) -> f32
	{
		let durationFromLast = self._now.duration_since(self._lastTime).as_nanos() as f64;
		let diff = durationFromLast / duration.as_nanos() as f64;
		return (value as f64*diff) as f32;
	}
	
	pub fn FromTo(&self, start: f32, end: f32, duree: Duration) -> InterpolateTimer
	{
		let nanos = duree.as_nanos();
		return InterpolateTimer{
			_startVal: start,
			_endVal: end,
			_duration: nanos,
			_startTime: self._now,
			_type: InterpolateTimer_type::CLAMP,
		};
	}
}
