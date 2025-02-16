use crate::components::cacheInfos::cacheInfos;
use crate::components::event::{event_trait, event_type};
use crate::components::hideable::hideable;
use crate::Interface::UiHitbox::UiHitbox;
use crate::Interface::UiPage::{UiPageContent, UiPageContent_type};
use crate::Shaders::HGE_shader_2Dsimple::HGE_shader_2Dsimple_def;
use crate::Shaders::ShaderDrawerImpl::{ShaderDrawerImpl, ShaderDrawerImplReturn};
use parking_lot::RwLock;
use std::sync::Arc;

pub trait UiButton_content: UiPageContent + ShaderDrawerImplReturn<HGE_shader_2Dsimple_def> {}
dyn_clone::clone_trait_object!(UiButton_content);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiButtonState
{
	IDLE,
	HOVER,
	PRESSED,
}

#[derive(Clone)]
pub struct UiButton
{
	_hitbox: UiHitbox,
	_content: Vec<Box<dyn UiButton_content + Send + Sync>>,
	_pressedFn: Arc<RwLock<Option<Box<dyn FnMut(&mut UiButton) + Send + Sync>>>>,
	_state: UiButtonState,
	_hide: bool,
	_cacheinfos: cacheInfos,
}

impl UiButton
{
	pub fn new() -> Self
	{
		return UiButton {
			_hitbox: UiHitbox::new(),
			_content: Vec::new(),
			_pressedFn: Arc::new(RwLock::new(None)),
			_state: UiButtonState::IDLE,
			_hide: false,
			_cacheinfos: cacheInfos::default(),
		};
	}

	// add a ui content to drawing
	pub fn add(&mut self, content: impl UiButton_content + Send + Sync + 'static)
	{
		self._content.push(Box::new(content));
		self._cacheinfos.setNeedUpdate(true);
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
		self._cacheinfos.setNeedUpdate(true);
	}

	fn setCacheToHover(&mut self)
	{
		//println!("set to hover");
		self._state = UiButtonState::HOVER;
		self._cacheinfos.setNeedUpdate(true);
	}

	fn setCacheToPressed(&mut self)
	{
		//println!("set to pressed");
		self._state = UiButtonState::PRESSED;
		self._cacheinfos.setNeedUpdate(true);
	}

	fn checkContentUpdate(&self) -> bool
	{
		self._content.iter().any(|x| x.cache_infos().isNotShow())
	}
}

impl event_trait for UiButton
{
	fn event_trigger(&mut self, eventtype: event_type) -> bool
	{
		let mut returning = false;
		match eventtype
		{
			event_type::IDLE =>
			{
				if (self._state != UiButtonState::IDLE)
				{
					self._content.iter_mut().filter(|x| x.event_have(eventtype)).for_each(|item| {
						item.event_trigger(eventtype);
					});
					self.setCacheToIdle();
					returning = true;
				}
			}
			event_type::HOVER =>
			{
				if (self._state != UiButtonState::HOVER)
				{
					self._content.iter_mut().filter(|x| x.event_have(eventtype)).for_each(|item| {
						item.event_trigger(eventtype);
					});
					self.setCacheToHover();
					returning = true;
				}
			}
			event_type::CLICKED =>
			{
				if (self._state != UiButtonState::PRESSED)
				{
					self._content.iter_mut().filter(|x| x.event_have(eventtype)).for_each(|item| {
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
			event_type::WINREFRESH =>
			{
				let mut update = false;
				for x in self._content.iter_mut().filter(|x| x.event_have(eventtype))
				{
					if (x.event_trigger(eventtype))
					{
						update = true;
					}
				}

				if (update)
				{
					returning = true;
				}
			}
			_ => (),
		};

		if (self._cacheinfos.isPresent() && returning)
		{
			self.cache_submit();
		}

		return returning;
	}

	fn event_have(&self, eventtype: event_type) -> bool
	{
		match eventtype
		{
			event_type::IDLE => true,
			event_type::HOVER => true,
			event_type::CLICKED => true,
			event_type::WINREFRESH => true,
			_ => false,
		}
	}
}

impl hideable for UiButton
{
	fn hide(&mut self)
	{
		self._hide = true;
		self._cacheinfos.setNeedUpdate(true);
	}

	fn show(&mut self)
	{
		self._hide = false;
		self._cacheinfos.setNeedUpdate(true);
	}

	fn isShow(&self) -> bool
	{
		!self._hide
	}
}

impl ShaderDrawerImpl for UiButton
{
	fn cache_mustUpdate(&self) -> bool
	{
		self.checkContentUpdate() || self._cacheinfos.isNotShow()
	}

	fn cache_infos(&self) -> &cacheInfos
	{
		&self._cacheinfos
	}

	fn cache_infos_mut(&mut self) -> &mut cacheInfos
	{
		&mut self._cacheinfos
	}

	fn cache_submit(&mut self)
	{
		if (self._hide)
		{
			self._cacheinfos.setNeedUpdate(false);
			self.cache_remove();
			return;
		}

		let mut newHitbox = UiHitbox::new();
		let mut haveOneNotCommit = false;
		let mut atleastOneDrawed = false;
		for x in self._content.iter_mut()
		{
			newHitbox.updateFromHitbox(x.getHitbox());
			x.cache_submit();
			if (x.cache_infos().isNotShow())
			{
				atleastOneDrawed = true;
			}
			else
			{
				haveOneNotCommit = true;
			}
		}

		if (self._state == UiButtonState::IDLE || self._hitbox.isEmpty())
		{
			if (newHitbox.isEmpty())
			{
				if (atleastOneDrawed)
				{
					self._cacheinfos.setNeedUpdate(false);
				}
				return;
			}
			self._hitbox = newHitbox;
		}

		if (!haveOneNotCommit)
		{
			self._cacheinfos.setNeedUpdate(false);
		}
		if (atleastOneDrawed)
		{
			self._cacheinfos.setPresent();
		}
		else
		{
			self._cacheinfos.setAbsent();
		}
	}

	fn cache_remove(&mut self)
	{
		for x in self._content.iter_mut()
		{
			x.cache_remove();
		}
		self._cacheinfos.setAbsent();
	}
}

impl UiPageContent for UiButton
{
	fn getType(&self) -> UiPageContent_type
	{
		return UiPageContent_type::INTERACTIVE;
	}

	fn getHitbox(&self) -> UiHitbox
	{
		self._hitbox.clone()
	}
}
