use std::any::Any;
use std::sync::{Arc};
use parking_lot::RwLock;
use crate::components::event::{event_trait, event_type};
use crate::components::hideable::hideable;
use crate::Interface::UiHitbox::UiHitbox;
use crate::Interface::UiPage::{UiPageContent, UiPageContent_type};
use crate::Shaders::StructAllCache::StructAllCache;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiButtonState
{
	IDLE,
	HOVER,
	PRESSED
}

#[derive(Clone)]
pub struct UiButton
{
	_hitbox: UiHitbox,
	_content: Vec<Box<dyn UiPageContent + Send + Sync>>,
	_pressedFn: Arc<RwLock<Option<Box<dyn FnMut(&mut UiButton) + Send + Sync>>>>,
	_state: UiButtonState,
	_hide: bool,
	_cacheUpdated: bool,
	_cache: StructAllCache
}

impl UiButton
{
	pub fn new() -> Self
	{
		return UiButton
		{
			_hitbox: UiHitbox::new(),
			_content: Vec::new(),
			_pressedFn: Arc::new(RwLock::new(None)),
			_state: UiButtonState::IDLE,
			_hide: false,
			_cacheUpdated: true,
			_cache: StructAllCache::new(),
		}
	}
	
	// add a ui content to drawing
	pub fn add(&mut self, content: impl UiPageContent + Send + Sync +'static)
	{
		self._content.push(Box::new(content));
	}
	
	pub fn setClickedFn(&mut self, func: impl FnMut(&mut UiButton) + Send + Sync + 'static)
	{
		*self._pressedFn.write() = Some(Box::new(func));
	}
	
	pub fn getState(&self) -> UiButtonState
	{
		return self._state;
	}
	
	pub fn boxed(self) -> Box<UiButton>
	{
		return Box::new(self);
	}
	
	pub fn content_mut(&mut self) -> &mut Vec<Box<dyn UiPageContent + Send + Sync>>
	{
		&mut self._content
	}
	
	///////////////// PRIVATE ////////////////
	
	fn setCacheToIdle(&mut self)
	{
		//println!("set to idle");
		self._state = UiButtonState::IDLE;
		self._cacheUpdated = true;
	}
	
	fn setCacheToHover(&mut self)
	{
		//println!("set to hover");
		self._state = UiButtonState::HOVER;
		self._cacheUpdated = true;
	}
	
	
	fn setCacheToPressed(&mut self)
	{
		//println!("set to pressed");
		self._state = UiButtonState::PRESSED;
		self._cacheUpdated = true;
	}
	
	fn checkContentUpdate(&self) -> bool
	{
		self._content.iter().any(|x|{
			x.cache_isUpdated()
		})
	}
	
}

impl event_trait for UiButton {
	fn event_trigger(&mut self, eventtype: event_type) -> bool
	{
		match eventtype {
			event_type::IDLE => {
				if(self._state != UiButtonState::IDLE)
				{
					self._content.iter_mut().for_each(|item|{
						item.event_trigger(eventtype);
					});
					self.setCacheToIdle();
					return true;
				}
				return false;
			}
			event_type::HOVER => {
				if(self._state != UiButtonState::HOVER)
				{
					self._content.iter_mut().for_each(|item|{
						item.event_trigger(eventtype);
					});
					self.setCacheToHover();
					return true;
				}
				return false;
			}
			event_type::CLICKED => {
				if(self._state != UiButtonState::PRESSED)
				{
					self._content.iter_mut().for_each(|item|{
						item.event_trigger(eventtype);
					});
					self.setCacheToPressed();
					let selfbinding = self._pressedFn.clone();
					let mut binding = selfbinding.write();
					if let Some(func) = binding.as_mut()
					{
						func(self);
					}
					return true;
				}
				return false;
			}
			event_type::EACH_SECOND => return false,
			event_type::EACH_TICK => return false,
			event_type::WINREFRESH => {
				let mut update = false;
				for x in self._content.iter_mut()
				{
					x.cache_update();
					if(x.event_trigger(eventtype.clone()))
					{
						update = true;
					}
				}
				
				return update;
			}
			_ => return false
		};
	}
	
	fn event_have(&self, _eventtype: event_type) -> bool
	{
		true
	}
}

impl hideable for UiButton
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

impl UiPageContent for UiButton
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
		
		if(self._state==UiButtonState::IDLE || self._hitbox.isEmpty())
		{
			if(newHitbox.isEmpty())
			{
				return;
			}
			self._hitbox = newHitbox;
		}
		
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
