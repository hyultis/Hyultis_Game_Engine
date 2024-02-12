use std::collections::HashMap;
use std::ops::Index;
use std::path::Path;
use Htrace::HTrace;
use tobj::{load_obj_buf, LoadError, LoadOptions};
use crate::assetStreamReader::assetManager;
use crate::components::{Components, HGEC_origin};
use crate::components::event::event_trait;
use crate::components::offset::offset;
use crate::components::rotations::rotation;
use crate::components::scale::scale;
use crate::components::worldPosition::worldPosition;
use crate::Models3D::chunk_content::chunk_content;
use crate::Shaders::Shs_3DVertex::{HGE_shader_3Dsimple, HGE_shader_3Dsimple_holder};
use crate::Shaders::StructAllCache::StructAllCache;

#[derive(Clone)]
pub struct loadOBJ
{
	_components: Components,
	_model: Option<tobj::Model>,
	_canUpdate: bool,
	_cache: StructAllCache
}

impl loadOBJ
{
	pub fn new(path: impl Into<String>) -> loadOBJ
	{
		loadOBJ::newFromIndex(path.into(), 0)
	}
	
	pub fn newFromIndex(path: impl Into<String>, index: usize) -> loadOBJ
	{
		let path = path.into();
		let pathbinding = path.clone();
		let parentpath = Path::new(&pathbinding);
		let mut model: Option<tobj::Model> = None;
		let mut tmp = Err(LoadError::InvalidLoadOptionConfig);
		
		if let Some(mut file) = assetManager::singleton().readFile(path.clone())
		{
			tmp = load_obj_buf(&mut file, &LoadOptions {
				single_index: true,
				triangulate: true,
				ignore_points: true,
				ignore_lines: true,
			}, |mat_path| {
				let mtlfile = parentpath.parent().unwrap().join(mat_path).to_str().unwrap().to_string();
				let mut ptr = assetManager::singleton().readFile(mtlfile).unwrap();
				tobj::load_mtl_buf(&mut ptr)
			})
		}
		
		if (tmp.is_ok())
		{
			let (tmpmodels, _) = tmp.unwrap();
			model = Some(tmpmodels.index(index).clone());
			
			HTrace!("Number of models          = {}", tmpmodels.len());
			//println!("Number of materials       = {}", tmpmaterials.unwrap().len());
		}
		else
		{
			HTrace!("not load : {} on {}", path, tmp.unwrap_err());
		}
		
		return loadOBJ
		{
			_components: Default::default(),
			_model: model,
			_canUpdate: true,
			_cache: StructAllCache::new()
		};
	}
	
	pub fn components(&self) -> &Components<worldPosition, rotation, scale, offset<worldPosition, rotation, scale>>
	{
		&self._components
	}
	pub fn components_mut(&mut self) -> &mut Components<worldPosition, rotation, scale, offset<worldPosition, rotation, scale>>
	{
		self._canUpdate = true;
		&mut self._components
	}
	
	fn convert_ObjToData(&mut self)
	{
		if (self._model.is_none())
		{
			return;
		}
		
		let mut new_vextex = Vec::new();
		let mut new_indice = Vec::new();
		let mut unique_vertices = HashMap::new();
		let textureId = match self._components.computeTexture() {
			None => 0,
			Some(texture) => texture.id
		};
		
		let thismesh = self._model.clone().unwrap().mesh;
		for index in 0..thismesh.indices.len()
		{
			let posindex = thismesh.indices[index] as usize * 3;
			let texindex = thismesh.indices[index] as usize * 2;
			
			
			let mut vertex = worldPosition::new(thismesh.positions[posindex],thismesh.positions[posindex+1],thismesh.positions[posindex+2]);
			self._components.computeVertex(&mut vertex);
			
			let newvertex = HGE_shader_3Dsimple {
				position: vertex.get(),
				normal: [thismesh.normals[posindex],
					thismesh.normals[posindex + 1],
					thismesh.normals[posindex + 2]],
				nbtexture: textureId,
				texcoord: [thismesh.texcoords[texindex], 1.0 - thismesh.texcoords[texindex + 1]],
				color: self._components.texture().color().getArray(),
				color_blend_type: self._components.texture().colorBlend().toU32()
			};
			
			if let Some(index) = unique_vertices.get(&newvertex)
			{
				new_indice.push(*index as u32);
			} else {
				let newindex = new_vextex.len() as u32;
				new_indice.push(newindex);
				unique_vertices.insert(newvertex, newindex);
				new_vextex.push(newvertex);
			}
		}
		
		self._cache = StructAllCache::newFrom::<HGE_shader_3Dsimple_holder>(HGE_shader_3Dsimple_holder::new(new_vextex,new_indice).into());
		self._canUpdate = false;
	}
}

impl event_trait for loadOBJ {}

impl chunk_content for loadOBJ
{
	fn cache_isUpdated(&self) -> bool {
		self._canUpdate
	}
	
	fn cache_update(&mut self) {
		
		self.convert_ObjToData();
	}
	
	fn cache_get(&self) -> &StructAllCache {
		&self._cache
	}
}
