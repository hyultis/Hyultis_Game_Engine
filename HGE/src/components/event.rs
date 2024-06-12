use std::collections::BTreeMap;
use std::sync::Arc;

pub trait event_trait_add<T>: event_trait
	where T: ?Sized + Send + Sync
{
	fn event_add(&mut self, eventtype: event_type, func: impl Fn(&mut T) -> bool + Send + Sync + 'static);
}

pub trait event_trait
{
	fn event_trigger(&mut self, _: event_type) -> bool
	{
		false
	}
	
	fn event_have(&self, _: event_type) -> bool
	{
		false
	}
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum event_type
{
	IDLE,
	HOVER,
	CLICKED,
	EACH_SECOND,
	EACH_TICK,
	WINREFRESH,
	ENTER,
	EXIT
}

impl event_type
{
	pub fn emptyRefresh<T>() -> fn(&mut T) -> bool
	{
		|_|{true}
	}
}


#[derive(Clone)]
pub struct event<T>
	where T: Send + Sync
{
	_all: BTreeMap<event_type,Vec<Arc<dyn Fn(&mut T) -> bool + Send + Sync>>>,
	_haveOneEvent: bool
}

impl<T> event<T>
	where T: Send + Sync
{
	pub fn new() -> Self
	{
		return event {
			_all: BTreeMap::new(),
			_haveOneEvent: false,
		};
	}
	
	pub fn add(&mut self, eventtype: event_type, func: impl Fn(&mut T) -> bool + Send + Sync + 'static)
	{
		self._haveOneEvent = true;
		match self._all.get_mut(&eventtype) {
			None => {
				self._all.insert(eventtype,vec![Arc::new(func)]);
			},
			Some(vec) => {
				vec.push(Arc::new(func));
			}
		}
	}
	
	pub fn trigger(&self, eventtype: event_type, data: &mut T) -> bool
	{
		let mut change = false;
		if let Some(tmp) = self._all.get(&eventtype)
		{
			for func in tmp.iter() {
				if (func(data))
				{
					change = true;
				}
			}
		}
		
		return change;
	}

	pub fn haveOneEvent(&self) -> bool
	{
		return self._haveOneEvent;
	}
	
	pub fn have(&self,eventtype: event_type) -> bool
	{
		if(!self._haveOneEvent)
		{
			return false;
		}
		
		if let Some(this) = self._all.get(&eventtype)
		{
			return this.len()>0;
		}
		return false;
	}
}
