use crate::components::cacheInfos::cacheInfos;
use crate::components::color::color;
use crate::components::event::{event_trait, event_trait_add, event_type};
use crate::components::interfacePosition::interfacePosition;
use crate::entities::Plane::Plane;
use crate::Interface::UiButton::UiButton_content;
use crate::Interface::UiHidable::UiHidable_content;
use crate::Interface::UiHitbox::{UiHitbox, UiHitbox_raw};
use crate::Interface::UiPage::{UiPageContent, UiPageContent_type};
use crate::Shaders::HGE_shader_2Dsimple::{HGE_shader_2Dsimple_def, HGE_shader_2Dsimple_holder};
use crate::Shaders::ShaderDrawer::ShaderDrawer_Manager;
use crate::Shaders::ShaderDrawerImpl::{ShaderDrawerImpl, ShaderDrawerImplReturn, ShaderDrawerImplStruct};

impl Plane<interfacePosition>
{
	/// define a plane as 2D square
	/// z depend of leftTop
	pub fn setSquare(&mut self, leftTop: interfacePosition, mut bottomRight: interfacePosition)
	{
		let z = leftTop.getZ();
		bottomRight.setZ(z);
		self._pos[1] = interfacePosition::fromSame(&bottomRight, &leftTop);
		self._pos[2] = interfacePosition::fromSame(&leftTop, &bottomRight);
		self._pos[0] = leftTop;
		self._pos[3] = bottomRight;
		self._cacheinfos.setNeedUpdate(true);
	}
}

impl UiPageContent for Plane<interfacePosition>
{
	fn getType(&self) -> UiPageContent_type
	{
		if (self._events.haveOneEvent())
		{
			return UiPageContent_type::INTERACTIVE;
		}
		return UiPageContent_type::IDLE;
	}

	fn getHitbox(&self) -> UiHitbox
	{
		self._hitbox.clone()
	}
}

impl ShaderDrawerImpl for Plane<interfacePosition>
{
	fn cache_mustUpdate(&self) -> bool
	{
		self._cacheinfos.isNotShow()
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
		let Some(structure) = self.cache_get()
		else
		{
			self.cache_remove();
			return;
		};

		let tmp = self._cacheinfos;
		if (!ShaderDrawer_Manager::inspect::<HGE_shader_2Dsimple_holder>(move |holder| {
			holder.insert(tmp, structure);
		}))
		{
			return;
		}
		self._cacheinfos.setPresent();
	}

	fn cache_remove(&mut self)
	{
		let tmp = self._cacheinfos;
		ShaderDrawer_Manager::inspect::<HGE_shader_2Dsimple_holder>(move |holder| {
			holder.remove(tmp);
		});
		self._cacheinfos.setAbsent();
	}
}

impl ShaderDrawerImplReturn<HGE_shader_2Dsimple_def> for Plane<interfacePosition>
{
	fn cache_get(&mut self) -> Option<ShaderDrawerImplStruct<HGE_shader_2Dsimple_def>>
	{
		if (!self._events.have(event_type::WINREFRESH))
		{
			self._events.add(event_type::WINREFRESH, event_type::emptyRefresh());
		}

		let texturename = self._components.texture().getName().clone();

		let color_blend_type = self._components.texture().colorBlend().toU32();
		let mut texturecolor = color::default();
		let mut textureuvcoord = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
		if let Some(texture) = self._components.computeTexture()
		{
			texturecolor = texture.color;
			textureuvcoord = texture.uvcoord.toArray4();
		}

		let mut vecstruct = Vec::new();
		for i in 0..4
		{
			let mut tmp = self._pos[i].clone();
			self._components.computeVertex(&mut tmp);

			vecstruct.push(HGE_shader_2Dsimple_def {
				position: tmp.convertToVertex(),
				ispixel: tmp.getTypeInt(),
				texture: texturename.clone(),
				uvcoord: self._uvcoord.map(|x| x[i]).unwrap_or(textureuvcoord[i]),
				color: self._color.map(|x| x[i].blend(texturecolor)).unwrap_or(texturecolor).toArray(),
				color_blend_type: color_blend_type,
			});
		}

		if let Some(vec) = &self._posHitbox
		{
			let mut hitboxvec = Vec::new();
			for i in 0..4
			{
				let mut tmp = vec[i].clone();
				self._components.computeVertex(&mut tmp);

				hitboxvec.push(UiHitbox_raw {
					position: tmp.convertToVertex(),
					ispixel: tmp.getTypeInt() == 1,
				});
			}

			self._hitbox = UiHitbox::newFrom2D(&hitboxvec);
		}
		else
		{
			let mut hitboxvec = Vec::new();
			for x in vecstruct.iter()
			{
				hitboxvec.push(UiHitbox_raw {
					position: x.position,
					ispixel: x.ispixel == 1,
				});
			}
			self._hitbox = UiHitbox::newFrom2D(&hitboxvec);
		}

		self._cacheinfos.setNeedUpdate(false);

		return Some(ShaderDrawerImplStruct {
			vertex: vecstruct,
			indices: [0, 1, 2, 1, 3, 2].to_vec(),
		});
	}
}

impl UiHidable_content for Plane<interfacePosition> {}
impl UiButton_content for Plane<interfacePosition> {}

impl event_trait for Plane<interfacePosition>
{
	fn event_trigger(&mut self, eventtype: event_type) -> bool
	{
		let update = self._events.clone().trigger(eventtype, self);
		if (eventtype == event_type::WINREFRESH)
		{
			self._cacheinfos.setNeedUpdate(true);
		}
		if (self._cacheinfos.isPresent() && update)
		{
			self.cache_submit();
		}
		return update;
	}

	fn event_have(&self, eventtype: event_type) -> bool
	{
		self._events.have(eventtype)
	}
}

impl event_trait_add<Plane<interfacePosition>> for Plane<interfacePosition>
{
	fn event_add(&mut self, eventtype: event_type, func: impl Fn(&mut Plane<interfacePosition>) -> bool + Send + Sync + 'static)
	{
		self._events.add(eventtype, func);
	}
}
