use std::any::Any;
use ahash::AHashMap;
use downcast_rs::Downcast;
use dyn_clone::DynClone;
use HArcMut::HArcMut;
use crate::components::event::{event, event_trait, event_type};
use crate::Interface::UiHitbox::UiHitbox;
use crate::Shaders::ShaderDrawerImpl::ShaderDrawerImpl;

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
	_content: AHashMap<String, HArcMut<Box<dyn UiPageContent + Sync + Send>>>,
	_events: event<UiPage>
}

impl UiPage
{
	pub fn new() -> Self
	{
		return UiPage
		{
			_content: AHashMap::new(),
			_events: event::new(),
		};
	}
	
	pub fn add(&mut self, name: impl Into<String>, mut content: impl UiPageContent + Any + Clone + Sync + Send + 'static) -> HArcMut<Box<dyn UiPageContent + Sync + Send>>
	{
		let name: String = name.into();
		content.cache_submit();
		let content: Box<dyn UiPageContent + Sync + Send> = Box::new(content); // need to be explicit
		let returning = HArcMut::new(content);
		if let Some(oldone) = self._content.insert(name, returning.clone())
		{
			oldone.setDrop();
		}
		
		return returning;
	}
	
	pub fn remove(&mut self, name: impl Into<String>)
	{
		let name = name.into();
		self._content.remove(&name);
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
	
	pub fn eventMouse(&mut self, x: u16, y: u16, clicked: bool) -> bool
	{
		let mut haveClicked = false;
		self._content.iter_mut()
			.filter(|(_, elem)| elem.get().getType() == UiPageContent_type::INTERACTIVE)
			.for_each(|(_, elem)| {
				let mut eventtype = event_type::IDLE;
				if (elem.get().getHitbox().isInside(x, y))
				{
					if (clicked)
					{
						eventtype = event_type::CLICKED;
					} else {
						eventtype = event_type::HOVER;
					}
				}
				elem.updateIf(|i|{
					if(eventtype==event_type::CLICKED && i.event_have(eventtype))
					{
						haveClicked = true;
					}
					i.event_trigger(eventtype)
				});
			});
		
		return haveClicked;
	}
	
	pub fn subevent_gets(&self, eventtype: event_type) -> Vec<String>
	{
		return self._content.iter()
			.filter(|(_, elem)| {
				let this = elem.get();
				this.getType() == UiPageContent_type::INTERACTIVE && this.event_have(eventtype)
			})
			.map(|(name, _)| {
				name.clone()
			}).collect::<Vec<String>>();
	}
	
	pub fn subevent_trigger(&mut self, names: Vec<String>, eventtype: event_type)
	{
		for name in names
		{
			if let Some(content) = self._content.get_mut(&name)
			{
				content.updateIf(|i|i.event_trigger(eventtype));
			}
		}
	}
	
	pub fn eventWinRefresh(&mut self)
	{
		self._content.iter_mut()
			.filter(|(_, elem)| elem.get().getType() == UiPageContent_type::INTERACTIVE)
			.for_each(|(_, elem)| {
				elem.updateIf(|i|i.event_trigger(event_type::WINREFRESH));
			});
	}
	
	pub fn cache_resubmit(&mut self)
	{
		let haveupdate = self._content.iter()
			.any(|(_, elem)| elem.get().cache_mustUpdate() || elem.isWantDrop());
		
		if (!haveupdate) {return}
		
		let havedrop = self._content.iter()
			.any(|(_, elem)| elem.isWantDrop());
		
		if(havedrop)
		{
			self._content.retain(|_, item| !item.isWantDrop());
		}
		
		self._content.iter().for_each(|(_, elem)| {
			elem.update(|i| {
				i.cache_submit();
			});
		});
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
