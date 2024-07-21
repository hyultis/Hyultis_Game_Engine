use std::collections::HashMap;
use std::ops::Index;
use std::path::Path;
use Htrace::HTrace;
use tobj::{load_obj_buf, LoadError, LoadOptions};
use crate::assetStreamReader::assetManager;
use crate::components::{Components, HGEC_origin};
use crate::components::cacheInfos::cacheInfos;
use crate::components::event::event_trait;
use crate::components::offset::offset;
use crate::components::rotations::rotation;
use crate::components::scale::scale;
use crate::components::worldPosition::worldPosition;
use crate::Models3D::chunk_content::chunk_content;
use crate::Shaders::HGE_shader_3Dsimple::{HGE_shader_3Dsimple_def, HGE_shader_3Dsimple_holder};
use crate::Shaders::ShaderDrawer::ShaderDrawer_Manager;
use crate::Shaders::ShaderDrawerImpl::{ShaderDrawerImpl, ShaderDrawerImplReturn, ShaderDrawerImplStruct};

#[derive(Clone)]
pub struct loadOBJ
{
	_components: Components,
	_model: Option<tobj::Model>,
	_cacheinfos: cacheInfos
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
			_cacheinfos: cacheInfos::default(),
		};
	}
	
	pub fn components(&self) -> &Components<worldPosition, rotation, scale, offset<worldPosition, rotation, scale>>
	{
		&self._components
	}
	pub fn components_mut(&mut self) -> &mut Components<worldPosition, rotation, scale, offset<worldPosition, rotation, scale>>
	{
		self._cacheinfos.setNeedUpdate(true);
		&mut self._components
	}
}

impl event_trait for loadOBJ {}


impl chunk_content for loadOBJ {}

impl ShaderDrawerImpl for loadOBJ {
	fn cache_mustUpdate(&self) -> bool {
		self._cacheinfos.isNotShow()
	}
	
	fn cache_infos(&self) -> &cacheInfos {
		&self._cacheinfos
	}
	
	fn cache_submit(&mut self) {
		let Some(structure) = self.cache_get() else {self.cache_remove();return};
		
		let tmp = self._cacheinfos;
		ShaderDrawer_Manager::inspect::<HGE_shader_3Dsimple_holder>(move |holder|{
			holder.insert(tmp,structure);
		});
		self._cacheinfos.setNeedUpdate(false);
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

impl ShaderDrawerImplReturn<HGE_shader_3Dsimple_def> for loadOBJ
{
	fn cache_get(&mut self) -> Option<ShaderDrawerImplStruct<HGE_shader_3Dsimple_def>> {
		
		if (self._model.is_none())
		{
			return None;
		}
		
		let mut new_vextex = Vec::new();
		let mut new_indice = Vec::new();
		let mut unique_vertices = HashMap::new();
		
		let thismesh = self._model.clone().unwrap().mesh;
		for index in 0..thismesh.indices.len()
		{
			let posindex = thismesh.indices[index] as usize * 3;
			let texindex = thismesh.indices[index] as usize * 2;
			
			
			let mut vertex = worldPosition::new(thismesh.positions[posindex],thismesh.positions[posindex+1],thismesh.positions[posindex+2]);
			self._components.computeVertex(&mut vertex);
			
			let newvertex = HGE_shader_3Dsimple_def {
				position: vertex.get(),
				normal: [thismesh.normals[posindex],
					thismesh.normals[posindex + 1],
					thismesh.normals[posindex + 2]],
				texture: self._components.texture().getName().clone(),
				color: self._components.texture().color().getArray(),
				color_blend_type: self._components.texture().colorBlend().toU32(),
				uvcoord: [thismesh.texcoords[texindex], 1.0 - thismesh.texcoords[texindex + 1]],
			};
			
			if let Some(index) = unique_vertices.get(&newvertex)
			{
				new_indice.push(*index as u32);
			} else {
				let newindex = new_vextex.len() as u32;
				new_indice.push(newindex);
				unique_vertices.insert(newvertex.clone(), newindex);
				new_vextex.push(newvertex);
			}
		}
		
		Some(
			ShaderDrawerImplStruct{
				vertex: new_vextex,
				indices: new_indice,
			})
	}
}
