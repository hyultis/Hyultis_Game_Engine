
use crate::components::cacheInfos::cacheInfos;
use crate::components::event::{event, event_trait, event_trait_add, event_type};
use crate::components::hideable::hideable;
use crate::Interface::UiHitbox::UiHitbox;
use crate::Interface::UiPage::{UiPageContent, UiPageContent_type};
use crate::Shaders::HGE_shader_2Dsimple::{HGE_shader_2Dsimple_def};
use crate::Shaders::ShaderDrawerImpl::{ShaderDrawerImpl, ShaderDrawerImplReturn};

pub trait UiHidable_content: UiPageContent + ShaderDrawerImplReturn<HGE_shader_2Dsimple_def>{}
dyn_clone::clone_trait_object!(UiHidable_content);

#[derive(Clone)]
pub struct UiHidable
{
	_hitbox: UiHitbox,
	_content: Vec<Box<dyn UiHidable_content + Send + Sync>>,
	_hide: bool,
	_cacheinfos: cacheInfos,
	_event: event<Self>,
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
			_cacheinfos: Default::default(),
			_event: event::new(),
		}
	}
	
	// add a ui content to drawing
	pub fn add(&mut self, content: impl UiHidable_content + Send + Sync +'static)
	{
		self._content.push(Box::new(content));
		self._cacheinfos.setNeedUpdate(true);
	}
	
	pub fn boxed(self) -> Box<UiHidable>
	{
		return Box::new(self);
	}
	
	pub fn content_mut(&mut self) -> &mut Vec<Box<dyn UiHidable_content + Send + Sync>>
	{
		self._cacheinfos.setNeedUpdate(true);
		&mut self._content
	}
	
	///////////////// PRIVATE ////////////////
	
	fn checkContentUpdate(&self) -> bool
	{
		self._content.iter().any(|x|{
			x.cache_mustUpdate()
		})
	}
	
	fn checkContentIsPresent(&self) -> bool
	{
		self._content.iter().any(|x|{
			x.cache_infos().isPresent()
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
		self._cacheinfos.setNeedUpdate(true);
	}
	
	fn show(&mut self) {
		self._hide = false;
		self._cacheinfos.setNeedUpdate(true);
	}
	
	fn isShow(&self) -> bool {
		!self._hide
	}
}

impl ShaderDrawerImpl for UiHidable {
	fn cache_mustUpdate(&self) -> bool
	{
		self._cacheinfos.isNotShow() || self.checkContentUpdate()
	}
	
	fn cache_infos(&self) -> &cacheInfos {
		&self._cacheinfos
	}
	
	fn cache_infos_mut(&mut self) -> &mut cacheInfos {
		&mut self._cacheinfos
	}
	
	fn cache_submit(&mut self)
	{
		if(self._hide)
		{
			self._content.iter_mut().for_each(|x|x.cache_remove());
			return;
		}
		
		let mut newHitbox = UiHitbox::new();
		self._content.iter_mut().for_each(|x|newHitbox.updateFromHitbox(x.getHitbox()));
		self._hitbox = newHitbox;
		self._cacheinfos.setNeedUpdate(false);
		self._cacheinfos.setPresent();
		self._content.iter_mut().for_each(|x|x.cache_submit());
	}
	
	fn cache_remove(&mut self) {
		self._content.iter_mut().for_each(|x|x.cache_remove());
		self._cacheinfos.setAbsent();
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
}
