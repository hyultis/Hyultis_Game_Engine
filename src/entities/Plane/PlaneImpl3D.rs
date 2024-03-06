use crate::components::color::color;
use crate::components::HGEC_origin;
use crate::components::worldPosition::worldPosition;
use crate::entities::Plane::Plane;
use crate::Models3D::ModelUtils;
use crate::Shaders::HGE_shader_3Dsimple::{HGE_shader_3Dsimple_def, HGE_shader_3Dsimple_holder};
use crate::Shaders::ShaderDrawer::ShaderDrawer_Manager;
use crate::Shaders::ShaderDrawerImpl::{ShaderDrawerImpl, ShaderDrawerImplReturn, ShaderDrawerImplStruct};

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
}

impl ShaderDrawerImpl for Plane<worldPosition> {
	fn cache_mustUpdate(&self) -> bool {
		self._canUpdate
	}
	
	fn cache_submit(&mut self) {
		let Some(structure) = self.cache_get() else {return};
		
		if(ShaderDrawer_Manager::singleton().inspect::<HGE_shader_3Dsimple_holder>(|holder|{
			self._uuidStorage = Some(holder.insert(self._uuidStorage,structure));
		}))
		{
			self._canUpdate = false;
		}
	}
}

impl ShaderDrawerImplReturn<HGE_shader_3Dsimple_def> for Plane<worldPosition>
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
		let mut textureuvcoord = [[0.0,0.0],[1.0,0.0],[0.0,1.0],[1.0,1.0]];
		if let Some(texture) = self._components.computeTexture()
		{
			textureuvcoord = texture.uvcoord.toArray4();
		}
		
		let mut vecstruct = Vec::new();
		for i in 0..4
		{
			let mut vertex = self._pos[i];
			self._components.computeVertex(&mut vertex);
			
			vecstruct.push(HGE_shader_3Dsimple_def {
				position: vertex.get(),
				normal: [0.0, 0.0, 0.0],
				texture: texturename.clone(),
				color_blend_type,
				uvcoord: self._uvcoord.map(|x|{x[i]}).unwrap_or(textureuvcoord[i]),
				color: self._color.map(|x|{x[i].blend(texturecolor)}).unwrap_or(texturecolor).toArray(),
			});
		}
		
		let indice = [0, 1, 2, 1, 3, 2].to_vec();
		ModelUtils::generateNormal(&mut vecstruct, &indice);
		
		return Some(
			ShaderDrawerImplStruct{
				vertex: vecstruct,
				indices: indice,
			});
	}
}
