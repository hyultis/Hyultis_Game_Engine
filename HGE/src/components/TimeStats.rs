use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};

pub struct TimeStats
{
	_last: Instant,
	_datas: Vec<Duration>,
	_lastDuration: Duration
}

impl TimeStats
{
	pub(crate) fn new() -> Self {
		return Self {
			_last: Instant::now(),
			_datas: Vec::new(),
			_lastDuration: Default::default(),
		};
	}
	
	pub fn setNow(&mut self)
	{
		self._last = Instant::now();
	}
	
	pub fn putElapsed(&mut self)
	{
		let lastDuration = self._last.elapsed();
		self._last = Instant::now();
		self._datas.push(lastDuration);
		self._lastDuration = lastDuration;
		if(self._datas.len() > 100)
		{
			self._datas.remove(0);
		}
	}
	
	pub fn getStats(&self) -> u128 {
		self._datas.iter().map(|x| x.as_micros()).sum::<u128>() / self._datas.len() as u128
	}
}

impl Display for TimeStats {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:.>6}", self.getStats())
	}
}
