use std::any::Any;
use std::collections::BTreeMap;
use downcast_rs::Downcast;
use dyn_clone::DynClone;
use HArcMut::HArcMut;
use crate::components::event::{event, event_trait, event_type};
use crate::Interface::UiHitbox::UiHitbox;
use crate::Shaders::ShaderDrawerImpl::ShaderDrawerImpl;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum UiPageContent_type
{
	IDLE,
	// no interaction
	INTERACTIVE,        // Interactive function will be called (hover/clicked)
}

pub enum UiEvent
{
	CLICKED,
	HOVER,
}

pub trait UiPageContent: DynClone + event_trait + ShaderDrawerImpl + Downcast
{
	fn getType(&self) -> UiPageContent_type
	{
		return UiPageContent_type::IDLE;
	}
	
	fn getHitbox(&self) -> UiHitbox;
}

dyn_clone::clone_trait_object!(UiPageContent);

#[derive(Clone)]
pub struct UiPage
{
	_content: BTreeMap<String, HArcMut<Box<dyn UiPageContent + Sync + Send>>>,
	_events: event<UiPage>
}

impl UiPage
{
	pub fn new() -> Self
	{
		return UiPage
		{
			_content: BTreeMap::new(),
			_events: event::new(),
		};
	}
	
	pub fn add(&mut self, name: impl Into<String>, content: impl UiPageContent + Any + Clone + Sync + Send + 'static) -> HArcMut<Box<dyn UiPageContent + Sync + Send>>
	{
		let name: String = name.into();
		let content: Box<dyn UiPageContent + Sync + Send> = Box::new(content); // need to be explicit
		let returning = HArcMut::new(content);
		if let Some(oldone) = self._content.insert(name, returning.clone())
		{
			oldone.setDrop();
			oldone.get_mut().cache_remove();
		}
		
		return returning;
	}
	
	pub fn remove(&mut self, name: impl Into<String>)
	{
		let name = name.into();
		if let Some(entity) = self._content.remove(&name)
		{
			entity.get_mut().cache_remove();
		}
	}

	pub fn get(&self, name: impl Into<String>) -> Option<HArcMut<Box<dyn UiPageContent + Sync + Send>>>
	{
		self._content.get(&name.into()).map(|item|item.clone())
	}
	
	pub fn eventEnter(&mut self, func: impl Fn(&mut UiPage) -> bool + Send + Sync + 'static)
	{
		self._events.add(event_type::ENTER, func);
	}
	
	pub fn eventExit(&mut self, func: impl Fn(&mut UiPage) -> bool + Send + Sync + 'static)
	{
		self._events.add(event_type::EXIT, func);
	}
	
	pub fn eventMouse(&self, x: u16, y: u16, clicked: bool) -> bool
	{
		let haveClicked = Arc::new(AtomicBool::new(false));
		self._content.par_iter()
			.filter(|(_, elem)| {
				let tmp = elem.get();
				tmp.getType() == UiPageContent_type::INTERACTIVE && (tmp.event_have(event_type::IDLE) || tmp.event_have(event_type::CLICKED) || tmp.event_have(event_type::HOVER))
			})
			.for_each(|(_, elem)| {
				let sub_haveClicked = haveClicked.clone();
				elem.updateIf(move |i|
				{
					let mut eventtype = event_type::IDLE;
					if (i.getHitbox().isInside(x, y))
					{
						if (clicked)
						{
							eventtype = event_type::CLICKED;
						} else {
							eventtype = event_type::HOVER;
						}
					}
					
					let eventok = i.event_trigger(eventtype);
					if(eventtype==event_type::CLICKED && eventok)
					{
						sub_haveClicked.clone().store(true,Ordering::Relaxed);
					}
					eventok
				});
			});
		
		return haveClicked.load(Ordering::Relaxed);
	}
	
	pub fn subevent_trigger(&self, eventtype: event_type)
	{
		for (_,content) in self._content.iter().filter(|(_,elem)|{
			let tmp = elem.get();
			tmp.event_have(eventtype)
		})
		{
			content.updateIf(|i|{i.event_trigger(eventtype)});
		}
	}
	
	pub fn eventWinRefresh(&self)
	{
		self._content.iter()
			.filter(|(_, elem)| {
				let tmp = elem.get();
				tmp.getType() == UiPageContent_type::INTERACTIVE && tmp.event_have(event_type::WINREFRESH)
			})
			.for_each(|(_, elem)| {
				elem.updateIf(|i|i.event_trigger(event_type::WINREFRESH));
			});
	}
	
	pub fn cache_clear(&self)
	{
		self._content.iter().for_each(|(_, elem)| {
			elem.update(|i| {
				i.cache_remove();
			});
		});
	}
	
	pub fn cache_check(&mut self)
	{
		self._content.iter()
			.for_each(|(_, elem)| {
				elem.updateIf(|i|{
					let mut returning = false;
					if(i.cache_mustUpdate())
					{
						i.cache_submit();
						returning = true;
					}
					returning
				});
			});
	}
	
	pub fn cache_resubmit(&mut self)
	{
		println!("page resubmit");
		
		let havedrop = self._content.iter()
			.any(|(_, elem)| elem.isWantDrop());
		
		if(havedrop)
		{
			self._content.retain(|_, item| !item.isWantDrop());
		}
		println!("page havedrop{}",havedrop);
		
		self._content.iter().for_each(|(_, elem)| {
			elem.update(|i| {
				i.cache_submit();
			});
		});
		println!("page end resubmit");
	}
}

impl event_trait for UiPage
{
	fn event_trigger(&mut self, event_type: event_type) -> bool
	{
		return self._events.clone().trigger(event_type,self);
	}
	
	fn event_have(&self, event_type: event_type) -> bool
	{
		return self._events.have(event_type);
	}
}
