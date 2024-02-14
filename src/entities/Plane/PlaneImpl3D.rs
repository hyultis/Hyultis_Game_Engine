use crate::components::HGEC_origin;
use crate::components::worldPosition::worldPosition;
use crate::entities::Plane::Plane;
use crate::Models3D::chunk_content::chunk_content;
use crate::Models3D::ModelUtils;
use crate::Shaders::HGE_shader_3Dsimple::{HGE_shader_3Dsimple, HGE_shader_3Dsimple_holder};
use crate::Shaders::StructAllCache::StructAllCache;

impl Plane<worldPosition>
{
	
	/// define a plane as flat 2D square aligned on X
	/// X depend of leftTop
	pub fn setSquareX(&mut self, leftTop: worldPosition, mut bottomRight: worldPosition)
	{
		bottomRight.x = leftTop.x;
		self._pos[0] = leftTop;
		self._pos[1] = worldPosition {
			x: leftTop.x,
			y: bottomRight.y,
			z: leftTop.z,
		};
		self._pos[2] = worldPosition {
			x: leftTop.x,
			y: leftTop.y,
			z: bottomRight.z,
		};
		self._pos[3] = bottomRight;
		self._canUpdate = true;
	}
	
	/// define a plane as flat 2D square aligned on Y
	/// Y depend of leftTop
	pub fn setSquareY(&mut self, leftTop: worldPosition, mut bottomRight: worldPosition)
	{
		bottomRight.y = leftTop.y;
		self._pos[0] = leftTop;
		self._pos[1] = worldPosition {
			x: bottomRight.x,
			y: leftTop.y,
			z: leftTop.z,
		};
		self._pos[2] = worldPosition {
			x: leftTop.x,
			y: leftTop.y,
			z: bottomRight.z,
		};
		self._pos[3] = bottomRight;
		self._canUpdate = true;
	}
	
	/// define a plane as flat 2D square aligned on Z
	/// Z depend of leftTop
	pub fn setSquareZ(&mut self, leftTop: worldPosition, mut bottomRight: worldPosition)
	{
		bottomRight.z = leftTop.z;
		self._pos[0] = leftTop;
		self._pos[1] = worldPosition {
			x: bottomRight.x,
			y: leftTop.y,
			z: leftTop.z,
		};
		self._pos[2] = worldPosition {
			x: leftTop.x,
			y: bottomRight.y,
			z: leftTop.z,
		};
		self._pos[3] = bottomRight;
		self._canUpdate = true;
	}
	
	pub fn cacheRefreshWorld(&mut self)
	{
		let Some(texture) = self._components.computeTexture() else {
			return
		};
		
		let mut vecstruct = Vec::new();
		for i in 0..4
		{
			let mut vertex = self._pos[i];
			self._components.computeVertex(&mut vertex);
			
			vecstruct.push(HGE_shader_3Dsimple {
				position: vertex.get(),
				normal: [0.0, 0.0, 0.0],
				nbtexture: texture.id,
				texcoord: self._uvcoord.map(|x|{x[i]}).unwrap_or(texture.uvcoord.toArray4()[i]),
				color: self._color.map(|x|{x[i].blend(texture.color)}).unwrap_or(texture.color).toArray(),
				color_blend_type: 0,
			});
		}
		
		let indice = [0, 1, 2, 1, 3, 2].to_vec();
		ModelUtils::generateNormal(&mut vecstruct, indice.clone());
		
		self._cache = StructAllCache::newFrom::<HGE_shader_3Dsimple_holder>(HGE_shader_3Dsimple_holder::new(vecstruct,indice).into());
		//self._hitbox = UiHitbox::newFromCache(&self._cache);
		self._canUpdate = false;
	}
}

impl chunk_content for Plane<worldPosition>
{
	fn cache_isUpdated(&self) -> bool {
		self._canUpdate
	}
	
	fn cache_update(&mut self) {
		self.cacheRefreshWorld();
	}
	
	fn cache_get(&self) -> &StructAllCache {
		&self._cache
	}
}
