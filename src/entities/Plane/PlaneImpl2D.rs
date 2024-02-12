use std::any::Any;
use crate::components::interfacePosition::interfacePosition;
use crate::entities::Plane::Plane;
use crate::Interface::UiHitbox::UiHitbox;
use crate::Interface::UiPage::{UiPageContent, UiPageContent_type};
use crate::Shaders::Shs_2DVertex::{HGE_shader_2Dsimple, HGE_shader_2Dsimple_holder};
use crate::Shaders::StructAllCache::StructAllCache;

impl Plane<interfacePosition>
{
	/// define a plane as 2D square
	/// z depend of leftTop
	pub fn setSquare(&mut self, leftTop: interfacePosition, mut bottomRight: interfacePosition)
	{
		let z = leftTop.getZ();
		bottomRight.setZ(z);
		self._pos[1] = interfacePosition::fromSame(&bottomRight,&leftTop);
		self._pos[2] = interfacePosition::fromSame(&leftTop, &bottomRight);
		self._pos[0] = leftTop;
		self._pos[3] = bottomRight;
		self._canUpdate = true;
	}
	
	pub fn cacheRefreshUI(&mut self)
	{
		let Some(texture) = self._components.computeTexture() else {
			return
		};
		
		let mut vecstruct = Vec::new();
		for i in 0..4
		{
			let mut tmp = self._pos[i].clone();
			self._components.computeVertex(&mut tmp);
			
			vecstruct.push(HGE_shader_2Dsimple {
				position: tmp.convertToVertex(),
				ispixel: tmp.getTypeInt(),
				texture: texture.id,
				uvcoord: self._uvcoord.map(|x|{x[i]}).unwrap_or(texture.uvcoord.toArray4()[i]),
				color: self._color.map(|x|{x[i].blend(texture.color)}).unwrap_or(texture.color).toArray(),
				color_blend_type: self._components.texture().colorBlend().toU32(),
			});
		}
		
		if let Some(vec) = &self._posHitbox
		{
			let mut hitboxvec = Vec::new();
			for i in 0..4
			{
				let mut tmp = vec[i].clone();
				self._components.computeVertex(&mut tmp);
				
				hitboxvec.push(HGE_shader_2Dsimple {
					position: tmp.convertToVertex(),
					ispixel: tmp.getTypeInt(),
					texture: texture.id,
					uvcoord: self._uvcoord.map(|x|{x[i]}).unwrap_or(texture.uvcoord.toArray4()[i]),
					color: self._color.map(|x|{x[i].blend(texture.color)}).unwrap_or(texture.color).toArray(),
					color_blend_type: self._components.texture().colorBlend().toU32(),
				});
			}
			
			self._hitbox = UiHitbox::newFrom2D(&hitboxvec);
		}
		else
		{
			self._hitbox = UiHitbox::newFrom2D(&vecstruct);
		}
		
		self._cache = StructAllCache::newFrom::<HGE_shader_2Dsimple_holder>(HGE_shader_2Dsimple_holder::new(vecstruct,[0, 1, 2, 1, 3, 2].to_vec()).into());
		self._canUpdate = false;
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
	
	fn getHitbox(&self) -> UiHitbox {
		self._hitbox.clone()
	}
	
	fn cache_isUpdated(&self) -> bool {
		self._canUpdate
	}
	
	fn cache_update(&mut self) {
		self.cacheRefreshUI();
	}
	
	fn getCache(&self) -> &StructAllCache {
		&self._cache
	}
	
	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}
}
