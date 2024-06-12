use crate::components::corners::{corner2, corner4};
use crate::components::event::event_trait;
use crate::components::{Components, HGEC_origin};
use crate::components::cacheInfos::cacheInfos;
use crate::components::color::color;
use crate::components::offset::offset;
use crate::components::rotations::rotation;
use crate::components::scale::scale;
use crate::components::worldPosition::worldPosition;
use crate::Models3D::chunk_content::chunk_content;
use crate::Models3D::ModelUtils;
use crate::Shaders::HGE_shader_3Dsimple::{HGE_shader_3Dsimple_def, HGE_shader_3Dsimple_holder};
use crate::Shaders::ShaderDrawer::ShaderDrawer_Manager;
use crate::Shaders::ShaderDrawerImpl::{ShaderDrawerImpl, ShaderDrawerImplReturn, ShaderDrawerImplStruct};

#[derive(Clone)]
struct Cube
{
	_corner: [worldPosition; 2],
	_components: Components,
	_canUpdate: bool,
	_cacheinfos: cacheInfos
}

impl Cube
{
	fn new(corners: corner2<worldPosition>) -> Self
	{
		Self
		{
			_corner: corners.intoArray(),
			_components: Default::default(),
			_canUpdate: false,
			_cacheinfos: cacheInfos::default(),
		}
	}
}


impl Cube // Struct3D_vertex
{
	pub fn components(&self) -> &Components<worldPosition, rotation, scale, offset<worldPosition, rotation, scale>>
	{
		&self._components
	}
	pub fn components_mut(&mut self) -> &mut Components<worldPosition, rotation, scale, offset<worldPosition, rotation, scale>>
	{
		self._canUpdate = true;
		&mut self._components
	}
	
	fn getFace(&self, pos: corner4<worldPosition>) -> ShaderDrawerImplStruct<HGE_shader_3Dsimple_def>
	{
		let mut vecstruct = Vec::new();
		vecstruct.push(HGE_shader_3Dsimple_def {
			position: pos.LeftTop.get(),
			uvcoord: [0.0,0.0],
			..Default::default()
		});
		vecstruct.push(HGE_shader_3Dsimple_def {
			position: pos.RightTop.get(),
			uvcoord: [1.0,0.0],
			..Default::default()
		});
		vecstruct.push(HGE_shader_3Dsimple_def {
			position: pos.LeftBottom.get(),
			uvcoord: [0.0,1.0],
			..Default::default()
		});
		vecstruct.push(HGE_shader_3Dsimple_def {
			position: pos.RightBottom.get(),
			uvcoord: [1.0,1.0],
			..Default::default()
		});
		
		ShaderDrawerImplStruct{ vertex: vecstruct, indices: [0, 1, 2, 1, 3, 2].to_vec() }
	}
	
	fn getVertex(&self) -> ShaderDrawerImplStruct<HGE_shader_3Dsimple_def>
	{
		//front
		let mut allfaces = self.getFace(corner4{
			LeftTop: worldPosition::new(self._corner[0].x,self._corner[0].y,self._corner[0].z),
			RightTop: worldPosition::new(self._corner[1].x,self._corner[0].y,self._corner[0].z),
			LeftBottom: worldPosition::new(self._corner[1].x,self._corner[1].y,self._corner[0].z),
			RightBottom: worldPosition::new(self._corner[0].x,self._corner[1].y,self._corner[0].z),
		});
		//back
		allfaces.combine(&mut self.getFace(corner4{
			LeftTop: worldPosition::new(self._corner[0].x,self._corner[0].y,self._corner[1].z),
			RightTop: worldPosition::new(self._corner[1].x,self._corner[0].y,self._corner[1].z),
			LeftBottom: worldPosition::new(self._corner[1].x,self._corner[1].y,self._corner[1].z),
			RightBottom: worldPosition::new(self._corner[0].x,self._corner[1].y,self._corner[1].z),
		}));
		//top
		allfaces.combine(&mut self.getFace(corner4{
			LeftTop: worldPosition::new(self._corner[0].x,self._corner[1].y,self._corner[0].z),
			RightTop: worldPosition::new(self._corner[1].x,self._corner[1].y,self._corner[0].z),
			LeftBottom: worldPosition::new(self._corner[1].x,self._corner[1].y,self._corner[1].z),
			RightBottom: worldPosition::new(self._corner[0].x,self._corner[1].y,self._corner[1].z),
		}));
		//bottom
		allfaces.combine(&mut self.getFace(corner4{
			LeftTop: worldPosition::new(self._corner[0].x,self._corner[0].y,self._corner[0].z),
			RightTop: worldPosition::new(self._corner[1].x,self._corner[0].y,self._corner[0].z),
			LeftBottom: worldPosition::new(self._corner[1].x,self._corner[0].y,self._corner[1].z),
			RightBottom: worldPosition::new(self._corner[0].x,self._corner[0].y,self._corner[1].z),
		}));
		//Left
		allfaces.combine(&mut self.getFace(corner4{
			LeftTop: worldPosition::new(self._corner[0].x,self._corner[0].y,self._corner[0].z),
			RightTop: worldPosition::new(self._corner[0].x,self._corner[1].y,self._corner[0].z),
			LeftBottom: worldPosition::new(self._corner[0].x,self._corner[1].y,self._corner[1].z),
			RightBottom: worldPosition::new(self._corner[0].x,self._corner[0].y,self._corner[1].z),
		}));
		//Right
		allfaces.combine(&mut self.getFace(corner4{
			LeftTop: worldPosition::new(self._corner[1].x,self._corner[0].y,self._corner[0].z),
			RightTop: worldPosition::new(self._corner[1].x,self._corner[1].y,self._corner[0].z),
			LeftBottom: worldPosition::new(self._corner[1].x,self._corner[1].y,self._corner[1].z),
			RightBottom: worldPosition::new(self._corner[1].x,self._corner[0].y,self._corner[1].z),
		}));
		
		return allfaces;
	}
}

impl event_trait for Cube {}

impl chunk_content for Cube {}

impl ShaderDrawerImpl for Cube {
	fn cache_mustUpdate(&self) -> bool {
		self._canUpdate || self._cacheinfos.isAbsent()
	}
	
	fn cache_submit(&mut self) {
		let Some(structure) = self.cache_get() else {self.cache_remove();return};
		
		let tmp = self._cacheinfos;
		ShaderDrawer_Manager::inspect::<HGE_shader_3Dsimple_holder>(move |holder|{
			holder.insert(tmp,structure);
		});
		self._cacheinfos.setPresent();
	}
	
	fn cache_remove(&mut self) {
		let tmp = self._cacheinfos;
		ShaderDrawer_Manager::inspect::<HGE_shader_3Dsimple_holder>(move |holder|{
			holder.remove(tmp);
		});
		self._cacheinfos.setAbsent();
	}
}

impl ShaderDrawerImplReturn<HGE_shader_3Dsimple_def> for Cube
{
	fn cache_get(&mut self) -> Option<ShaderDrawerImplStruct<HGE_shader_3Dsimple_def>>
	{
		let texturename = self._components.texture().getName().clone();
		let color_blend_type = self._components.texture().colorBlend().toU32();
		let mut texturecolor = color::default();
		if let Some(texture) = self._components.computeTexture()
		{
			texturecolor = texture.color;
		}
		
		let mut vertex = self.getVertex();
		for x in vertex.vertex.iter_mut()
		{
			let mut tmp = worldPosition::new(x.position[0],x.position[1],x.position[2]);
			self._components.computeVertex(&mut tmp);
			x.position = tmp.get();
			x.texture = texturename.clone();
			x.color_blend_type = color_blend_type;
			x.color = texturecolor.toArray();
		}
		
		ModelUtils::generateNormal(&mut vertex.vertex, &vertex.indices);
		
		self._canUpdate = false;
		
		return Some(vertex);
	}
}
