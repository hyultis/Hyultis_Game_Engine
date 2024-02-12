use std::sync::Arc;
use ahash::AHashMap;

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

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
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
	_all: AHashMap<event_type,Vec<Arc<dyn Fn(&mut T) -> bool + Send + Sync>>>,
	_haveOneEvent: bool
}

impl<T> event<T>
	where T: Send + Sync
{
	pub fn new() -> Self
	{
		let mut tmp = AHashMap::new();
		tmp.insert(event_type::IDLE, Vec::new());
		tmp.insert(event_type::HOVER, Vec::new());
		tmp.insert(event_type::CLICKED, Vec::new());
		tmp.insert(event_type::EACH_SECOND, Vec::new());
		tmp.insert(event_type::EACH_TICK, Vec::new());
		tmp.insert(event_type::WINREFRESH, Vec::new());
		tmp.insert(event_type::ENTER, Vec::new());
		tmp.insert(event_type::EXIT, Vec::new());
		
		return event {
			_all: tmp,
			_haveOneEvent: false,
		};
	}
	
	pub fn add(&mut self, eventtype: event_type, func: impl Fn(&mut T) -> bool + Send + Sync + 'static)
	{
		self._haveOneEvent = true;
		self._all.get_mut(&eventtype).unwrap().push(Arc::new(func));
	}
	
	pub fn trigger(&self, eventtype: event_type, data: &mut T) -> bool
	{
		let mut change = false;
		for func in self._all.get(&eventtype).unwrap().iter() {
			if(func(data))
			{
				change = true;
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
