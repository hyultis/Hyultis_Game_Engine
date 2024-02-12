use std::any::Any;
use crate::components::event::{event, event_trait, event_trait_add, event_type};
use crate::components::hideable::hideable;
use crate::Interface::UiHitbox::UiHitbox;
use crate::Interface::UiPage::{UiPageContent, UiPageContent_type};
use crate::Shaders::StructAllCache::StructAllCache;


#[derive(Clone)]
pub struct UiHidable
{
	_hitbox: UiHitbox,
	_content: Vec<Box<dyn UiPageContent + Send + Sync>>,
	_hide: bool,
	_cacheUpdated: bool,
	_event: event<Self>,
	_cache: StructAllCache
}

impl UiHidable
{
	pub fn new() -> Self
	{
		return Self
		{
			_hitbox: UiHitbox::new(),
			_content: Vec::new(),
			_hide: false,
			_cacheUpdated: true,
			_event: event::new(),
			_cache: StructAllCache::new(),
		}
	}
	
	// add a ui content to drawing
	pub fn add(&mut self, content: impl UiPageContent + Send + Sync +'static)
	{
		self._content.push(Box::new(content));
	}
	
	pub fn boxed(self) -> Box<UiHidable>
	{
		return Box::new(self);
	}
	
	pub fn content_mut(&mut self) -> &mut Vec<Box<dyn UiPageContent + Send + Sync>>
	{
		&mut self._content
	}
	
	///////////////// PRIVATE ////////////////
	
	fn checkContentUpdate(&self) -> bool
	{
		self._content.iter().any(|x|{
			x.cache_isUpdated()
		})
	}
	
}

impl event_trait_add<UiHidable> for UiHidable
{
	fn event_add(&mut self, eventtype: event_type, func: impl Fn(&mut UiHidable) -> bool + Send + Sync + 'static) {
		self._event.add(eventtype,func);
	}
}

impl event_trait for UiHidable {
	fn event_trigger(&mut self, eventtype: event_type) -> bool
	{
		let mut returned = self._event.clone().trigger(eventtype,self);
		self._content.iter_mut().for_each(|item|{
			if(item.event_trigger(eventtype))
			{
				returned = true;
			}
		});
		
		return returned;
	}
	
	fn event_have(&self, _eventtype: event_type) -> bool
	{
		true
	}
}

impl hideable for UiHidable
{
	fn hide(&mut self) {
		self._hide = true;
		self._cacheUpdated = true;
	}
	
	fn show(&mut self) {
		self._hide = false;
		self._cacheUpdated = true;
	}
	
	fn isShow(&self) -> bool {
		!self._hide
	}
}

impl UiPageContent for UiHidable
{
	fn getType(&self) -> UiPageContent_type
	{
		return UiPageContent_type::INTERACTIVE;
	}
	
	fn getHitbox(&self) -> UiHitbox {
		self._hitbox.clone()
	}
	
	fn cache_isUpdated(&self) -> bool {
		self._cacheUpdated || self.checkContentUpdate()
	}
	
	fn cache_update(&mut self) {
		if(self._hide)
		{
			self._cache = StructAllCache::new();
			self._cacheUpdated = false;
			return;
		}
		
		let mut newcache = StructAllCache::new();
		let mut newHitbox = UiHitbox::new();
		for x in self._content.iter_mut()
		{
			if(x.cache_isUpdated() || x.getHitbox().isEmpty())
			{
				x.cache_update();
			}
			newcache.append(x.getCache());
			newHitbox.updateFromHitbox(x.getHitbox());
		}
		
		self._hitbox = newHitbox;
		self._cache = newcache;
		self._cacheUpdated = false;
	}
	
	fn getCache(&self) -> &StructAllCache
	{
		&self._cache
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}

}
