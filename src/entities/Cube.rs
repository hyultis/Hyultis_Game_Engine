use cgmath::Deg;
use crate::components::corners::corner2;
use crate::components::event::event_trait;
use crate::components::HGEC_origin;
use crate::components::worldPosition::worldPosition;
use crate::Models3D::chunk_content::chunk_content;
use crate::Models3D::ModelUtils;
use crate::Shaders::HGE_shader_3Dsimple::{HGE_shader_3Dsimple, HGE_shader_3Dsimple_holder};
use crate::Shaders::StructAllCache::StructAllCache;

#[derive(Clone)]
struct Cube
{
	_corner: [worldPosition; 2],
	_rotation: [Deg<f32>;3],
	_cache: StructAllCache
}

impl Cube
{
	fn new(corners: corner2<worldPosition>) -> Cube
	{
		Cube
		{
			_corner: corners.intoArray(),
			_rotation: [Deg(0.0),Deg(0.0),Deg(0.0)],
			_cache: StructAllCache::new(),
		}
	}
}


impl Cube // Struct3D_vertex
{
	pub fn cacheRefreshWorld(&mut self)
	{
		/*let mut fontid = 0;
		if let Some(texturename) = self._texture.as_ref()
		{
			fontid = ManagerTexture::singleton().getTextureToId(texturename);
		}*/
		let mut vecstruct = Vec::new();
		for _ in 0..6
		{
			let tmppos: worldPosition = worldPosition::default();//self._pos[i] + self._offset;
			
			vecstruct.push(HGE_shader_3Dsimple {
				position: tmppos.get(),
				normal: [0.0, 0.0, 0.0],
				nbtexture: 0,
				texcoord: [0.0,0.0],
				color: [1.0,1.0,1.0,1.0],
				color_blend_type: 0
			});
			vecstruct.push(HGE_shader_3Dsimple {
				position: tmppos.get(),
				normal: [0.0, 0.0, 0.0],
				nbtexture: 0,
				texcoord: [1.0,0.0],
				color: [1.0,1.0,1.0,1.0],
				color_blend_type: 0
			});
			vecstruct.push(HGE_shader_3Dsimple {
				position: tmppos.get(),
				normal: [0.0, 0.0, 0.0],
				nbtexture: 0,
				texcoord: [0.0,1.0],
				color: [1.0,1.0,1.0,1.0],
				color_blend_type: 0
			});
			vecstruct.push(HGE_shader_3Dsimple {
				position: tmppos.get(),
				normal: [0.0, 0.0, 0.0],
				nbtexture: 0,
				texcoord: [0.0,0.0],
				color: [1.0,1.0,1.0,1.0],
				color_blend_type: 0
			});
		}
		
		let indice = [0, 1, 2, 1, 3, 2].to_vec();
		ModelUtils::generateNormal(&mut vecstruct, indice.clone());
		
		self._cache = StructAllCache::newFrom::<HGE_shader_3Dsimple_holder>(HGE_shader_3Dsimple_holder::new(vecstruct,indice).into());
		//self._hitbox = UiHitbox::newFromCache(&self._cache);
	}
}

impl event_trait for Cube {}

impl chunk_content for Cube
{
	fn cache_isUpdated(&self) -> bool {
		true
	}
	
	fn cache_update(&mut self)
	{
		
		self.cacheRefreshWorld();
	}
	
	fn cache_get(&self) -> &StructAllCache {
		&self._cache
	}
}
