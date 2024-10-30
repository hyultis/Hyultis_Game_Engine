use dashmap::iter::Iter;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::fmt::{Display, Formatter};
use std::hash::RandomState;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

pub struct TimeStatsStorage
{
	_datas: DashMap<String, RwLock<TimeStats>>
}
static SINGLETON: OnceLock<TimeStatsStorage> = OnceLock::new();

impl TimeStatsStorage
{
	pub fn singleton() -> &'static Self
	{
		return SINGLETON.get_or_init(|| Self {
			_datas: Default::default(),
		});
	}
	
	/**
	 * force the time to be now
	 */
	pub fn forceNow(key: impl Into<String>)
	{
		let key = key.into();
		
		if (!Self::singleton()._datas.contains_key(&key))
		{
			Self::singleton()._datas.insert(key.clone(), RwLock::new(TimeStats::new()));
		}
		
		Self::singleton()._datas.get_mut(&key).unwrap().write().setNow();
	}
	
	/**
	 * update the stats, from the elapsed time (from the last now) and the set the new now
	 */
	pub fn update(key: impl Into<String>)
	{
		let key = key.into();
		
		if (!Self::singleton()._datas.contains_key(&key))
		{
			Self::singleton()._datas.insert(key.clone(), RwLock::new(TimeStats::new()));
		}
		
		Self::singleton()._datas.get_mut(&key).unwrap().write().putElapsed();
	}
	
	pub fn get<'a>() -> Iter<'a, String, RwLock<TimeStats>, RandomState, DashMap<String, RwLock<TimeStats>>>
	{
		return Self::singleton()._datas.iter();
	}
}

impl Display for TimeStatsStorage {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self._datas.iter().for_each(|k| {
			let _ = write!(f, "| {} = {}", k.key(), &*k.read());
		});
		
		return Ok(());
	}
}

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
		if (self._datas.len() > 100)
		{
			self._datas.remove(0);
		}
	}
	
	pub fn getStats(&self) -> u128 {
		if (self._datas.len() == 0)
		{
			return 0;
		}
		self._datas.iter().map(|x| x.as_micros()).sum::<u128>() / self._datas.len() as u128
	}
}

impl Display for TimeStats {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:.>6}", self.getStats())
	}
}
