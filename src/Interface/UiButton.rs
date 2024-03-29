use std::sync::{Arc};
use parking_lot::RwLock;
use uuid::Uuid;
use crate::components::event::{event_trait, event_type};
use crate::components::hideable::hideable;
use crate::Interface::UiHitbox::UiHitbox;
use crate::Interface::UiPage::{UiPageContent, UiPageContent_type};
use crate::Shaders::HGE_shader_2Dsimple::{HGE_shader_2Dsimple_def, HGE_shader_2Dsimple_holder};
use crate::Shaders::ShaderDrawer::ShaderDrawer_Manager;
use crate::Shaders::ShaderDrawerImpl::{ShaderDrawerImpl, ShaderDrawerImplReturn, ShaderDrawerImplStruct};


pub trait UiButton_content: UiPageContent + ShaderDrawerImplReturn<HGE_shader_2Dsimple_def>{}
dyn_clone::clone_trait_object!(UiButton_content);

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
	_content: Vec<Box<dyn UiButton_content + Send + Sync>>,
	_pressedFn: Arc<RwLock<Option<Box<dyn FnMut(&mut UiButton) + Send + Sync>>>>,
	_state: UiButtonState,
	_hide: bool,
	_cacheUpdated: bool,
	_uuidStorage: Option<Uuid>
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
			_uuidStorage: None
		}
	}
	
	// add a ui content to drawing
	pub fn add(&mut self, content: impl UiButton_content + Send + Sync +'static)
	{
		self._content.push(Box::new(content));
		self._cacheUpdated = true;
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
	
	pub fn content_mut(&mut self) -> &mut Vec<Box<dyn UiButton_content + Send + Sync>>
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
			x.cache_mustUpdate()
		})
	}
}

impl event_trait for UiButton {
	fn event_trigger(&mut self, eventtype: event_type) -> bool
	{
		let mut returning = false;
		match eventtype {
			event_type::IDLE => {
				if(self._state != UiButtonState::IDLE)
				{
					self._content.iter_mut().for_each(|item|{
						item.event_trigger(eventtype);
					});
					self.setCacheToIdle();
					returning = true;
				}
			}
			event_type::HOVER => {
				if(self._state != UiButtonState::HOVER)
				{
					self._content.iter_mut().for_each(|item|{
						item.event_trigger(eventtype);
					});
					self.setCacheToHover();
					returning = true;
				}
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
					returning = true;
				}
			}
			event_type::EACH_SECOND => return false,
			event_type::EACH_TICK => return false,
			event_type::WINREFRESH => {
				let mut update = false;
				for x in self._content.iter_mut()
				{
					if(x.event_trigger(eventtype.clone()))
					{
						update = true;
					}
				}
				
				if(update)
				{
					returning = true;
				}
			},
			_ => ()
		};
		
		if(self._uuidStorage.is_some() && returning)
		{
			self.cache_submit();
		}
		
		return returning;
	}
	
	fn event_have(&self, eventtype: event_type) -> bool
	{
		match eventtype {
			event_type::IDLE => true,
			event_type::HOVER => true,
			event_type::CLICKED => true,
			event_type::WINREFRESH => true,
			_ => false
		}
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

impl ShaderDrawerImpl for UiButton {
	fn cache_mustUpdate(&self) -> bool
	{
		self._cacheUpdated || self.checkContentUpdate()
	}
	
	fn cache_submit(&mut self)
	{
		if(self._hide)
		{
			if(ShaderDrawer_Manager::singleton().inspect::<HGE_shader_2Dsimple_holder>(|holder|{
				holder.remove(&mut self._uuidStorage);
			})){
				self._cacheUpdated = false;
			}
			return;
		}
		
		let mut structure = ShaderDrawerImplStruct::default();
		let mut newHitbox = UiHitbox::new();
		for x in self._content.iter_mut()
		{
			if let Some(mut content) = x.cache_get()
			{
				structure.combine(&mut content);
				newHitbox.updateFromHitbox(x.getHitbox());
			}
			else {
				println!("uibutton content invalid");
				return;
			}
		}
		
		if(self._state==UiButtonState::IDLE || self._hitbox.isEmpty())
		{
			if(newHitbox.isEmpty())
			{
				println!("uibutton pas de hitbox");
				return;
			}
			self._hitbox = newHitbox;
		}
		
		self._cacheUpdated = false;
		
		ShaderDrawer_Manager::singleton().inspect::<HGE_shader_2Dsimple_holder>(|holder|{
			self._uuidStorage = Some(holder.insert(self._uuidStorage,structure));
		});
	}
	
	fn cache_remove(&mut self) {
		ShaderDrawer_Manager::singleton().inspect::<HGE_shader_2Dsimple_holder>(|holder|{
			holder.remove(&mut self._uuidStorage);
		});
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
}
