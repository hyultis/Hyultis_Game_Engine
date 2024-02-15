use std::any::Any;
use ahash::AHashMap;
use dyn_clone::DynClone;
use HArcMut::HArcMut;
use crate::components::event::{event, event_trait, event_type};
use crate::Interface::UiHitbox::UiHitbox;
use crate::Shaders::StructAllCache::StructAllCache;

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

pub trait UiPageContent: DynClone + event_trait + Any
{
	fn getType(&self) -> UiPageContent_type
	{
		return UiPageContent_type::IDLE;
	}
	
	fn getHitbox(&self) -> UiHitbox;
	
	fn cache_isUpdated(&self) -> bool;
	fn cache_update(&mut self);
	fn getCache(&self) -> &StructAllCache;

	fn as_any_mut(&mut self) -> &mut dyn Any;
}

dyn_clone::clone_trait_object!(UiPageContent);

#[derive(Clone)]
pub struct UiPage
{
	_content: AHashMap<String, HArcMut<Box<dyn UiPageContent + Sync + Send>>>,
	_cachedStaticContent: StructAllCache,
	_cache: StructAllCache,
	_events: event<UiPage>
}

impl UiPage
{
	pub fn new() -> Self
	{
		return UiPage
		{
			_content: AHashMap::new(),
			_cachedStaticContent: StructAllCache::new(),
			_cache: StructAllCache::new(),
			_events: event::new(),
		};
	}
	
	pub fn add(&mut self, name: impl Into<String>, mut content: impl UiPageContent + Any + Clone + Sync + Send + 'static) -> HArcMut<Box<dyn UiPageContent + Sync + Send>>
	{
		let name: String = name.into();
		content.cache_update();
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
	
	pub fn cacheUpdate(&mut self) -> bool
	{
		let haveupdate = self._content.iter()
			.any(|(_, elem)| elem.get().cache_isUpdated() || elem.isWantDrop());
		
		if (haveupdate)
		{
			let havedrop = self._content.iter()
				.any(|(_, elem)| elem.isWantDrop());
			if(havedrop)
			{
				self._content.retain(|_, item| !item.isWantDrop());
			}
			
			let mut cache = StructAllCache::new();
			self._content.iter().for_each(|(_, elem)| {
				elem.updateIf(|i| {
					let mut haveupdated = false;
					if (i.cache_isUpdated())
					{
						haveupdated = true;
						i.cache_update();
					}
					cache.append(i.getCache());
					haveupdated
				});
			});
			cache.holderUpdate();
			self._cache = cache;
			return true;
		}
		
		return false;
	}
	
	pub fn cache_get(&self) -> &StructAllCache
	{
		return &self._cache;
	}
	
	///////////// PRIVATE ////////////
	
	/*fn checkCacheStaticUpdate(&mut self)
	{
		let mut havedrop = false;
		let haveStaticUpdated = self._content.iter_mut()
			.filter(|(_, item)| item.get().getType() == UiPageContent_type::IDLE)
			.find_map(|(_, item)| {
				let mut returning = None;
				item.updateIf(|i| {
					if (i.getCacheUpdated())
					{
						returning = Some(true);
						return true;
					}
					return false;
				});
				if(item.isWantDrop())
				{
					havedrop = true;
				}
				return returning;
			}).is_some();

		if(havedrop)
		{
			self._content.retain(|_,item|!item.isWantDrop());
		}
		
		if (havedrop || haveStaticUpdated)
		{
			self.reloadCacheStatic();
		}
	}
	
	fn reloadCacheStatic(&mut self)
	{
		let mut newcache = StructAllCache::new();
		self._content.iter_mut()
			.filter(|(_, item)| item.get().getType() == UiPageContent_type::IDLE)
			.for_each(|(_, item)| {
				item.updateIf(|i| {
					i.getCacheUpdated();
					newcache.append(i.getCache());
					true
				});
			});
		self._cachedStaticContent = newcache;
	}*/
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
