use std::hash::{Hash, Hasher};
use vulkano::pipeline::graphics::vertex_input::Vertex;
use std::convert::TryInto;
use anyhow::anyhow;
use Htrace::HTraceError;
use uuid::Uuid;
use vulkano::buffer::{BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::PipelineBindPoint;
use crate::HGEsubpass::HGEsubpassName;
use crate::ManagerBuilder::ManagerBuilder;
use crate::Pipeline::EnginePipelines;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Shaders::intoVertexed::IntoVertexted;
use crate::Shaders::Manager::ManagerShaders;
use crate::Shaders::names;
use crate::Shaders::ShaderDrawerImpl::ShaderDrawerImplStruct;
use crate::Shaders::ShaderStruct::{ShaderStruct, ShaderStructHolder, ShaderStructHolder_utils};
use crate::Textures::Manager::ManagerTexture;
use dashmap::DashMap;
use crate::components::cacheInfos::cacheInfos;


// struct externe, a changer en HGE_shader_2Dsimple
#[derive(Clone, Debug)]
pub struct HGE_shader_3Dsimple_def {
	pub position: [f32; 3],
	pub normal: [f32; 3],
	pub texture: Option<String>,
	pub uvcoord: [f32; 2],
	pub color: [f32; 4],
	pub color_blend_type: u32 // 0 = mul, 1 = add
}

impl Default for HGE_shader_3Dsimple_def
{
	fn default() -> Self {
		Self {
			position: [0.0, 0.0, 0.0],
			normal: [0.0, 0.0, 0.0],
			texture: None,
			uvcoord: [0.0, 0.0],
			color: [1.0, 1.0, 1.0, 1.0],
			color_blend_type: 0,
		}
	}
}

impl IntoVertexted<HGE_shader_3Dsimple> for HGE_shader_3Dsimple_def
{
	fn IntoVertexted(&self, _: bool) -> Option<HGE_shader_3Dsimple> {
		let mut textureid = 0;
		
		if let Some(texture) = &self.texture
		{
			let Some(id) = ManagerTexture::singleton().descriptorSet_getIdTexture(["HGE_set0","HGE_set1","HGE_set2"],texture.clone()) else { return None; };
			textureid = id.into();
		}
		
		return Some(HGE_shader_3Dsimple {
			position: self.position,
			normal: self.normal,
			nbtexture: textureid,
			color: self.color,
			color_blend_type: self.color_blend_type,
			texcoord: self.uvcoord,
		});
	}
}



impl PartialEq for HGE_shader_3Dsimple_def {
	fn eq(&self, other: &Self) -> bool {
		self.position == other.position &&
			self.normal == other.normal &&
			self.uvcoord == other.uvcoord &&
			self.color == other.color &&
			self.texture == other.texture &&
			self.color_blend_type == other.color_blend_type
	}
}

impl Eq for HGE_shader_3Dsimple_def {}

impl Hash for HGE_shader_3Dsimple_def {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.position[0].to_bits().hash(state);
		self.position[1].to_bits().hash(state);
		self.position[2].to_bits().hash(state);
		self.normal[0].to_bits().hash(state);
		self.normal[1].to_bits().hash(state);
		self.normal[2].to_bits().hash(state);
		self.uvcoord[0].to_bits().hash(state);
		self.uvcoord[1].to_bits().hash(state);
		self.color[0].to_bits().hash(state);
		self.color[1].to_bits().hash(state);
		self.color[2].to_bits().hash(state);
		self.color[3].to_bits().hash(state);
		self.texture.hash(state);
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Vertex, BufferContents)]
pub struct HGE_shader_3Dsimple
{
	#[format(R32G32B32_SFLOAT)]
	pub position: [f32; 3],
	#[format(R32G32B32_SFLOAT)]
	pub normal: [f32; 3],
	#[format(R32_UINT)]
	pub nbtexture: u32,
	#[format(R32G32_SFLOAT)]
	pub texcoord: [f32; 2],
	#[format(R32G32B32A32_SFLOAT)]
	pub color: [f32; 4],
	#[format(R32_UINT)]
	pub color_blend_type: u32 // 0 = mul, 1 = add
}

impl Default for HGE_shader_3Dsimple
{
	fn default() -> Self {
		HGE_shader_3Dsimple {
			position: [0.0, 0.0, 0.0],
			normal: [0.0, 0.0, 0.0],
			nbtexture: 0,
			texcoord: [0.0, 0.0],
			color: [1.0, 1.0, 1.0, 1.0],
			color_blend_type: 0,
		}
	}
}

impl ShaderStruct for HGE_shader_3Dsimple {
	fn createPipeline() -> anyhow::Result<()>
	{
		if ManagerShaders::singleton().get(names::simple3D).is_none()
		{
			return Err(anyhow!("missing shader \"{}\"",names::simple3D));
		}
		
		ManagerPipeline::singleton().addFunc(HGE_shader_3Dsimple_holder::pipelineName(), |renderpass,transparency| {
			EnginePipelines::singleton().pipelineCreation(names::simple3D,
				transparency,
				renderpass.clone(),
				HGEsubpassName::WORLDSOLID.getSubpassID(),
				HGE_shader_3Dsimple::per_vertex()
			)
		}, PrimitiveTopology::TriangleList, true);
		return Ok(());
	}
}

///////// Holder

pub struct HGE_shader_3Dsimple_holder
{
	_datas: DashMap<Uuid, ShaderDrawerImplStruct<Box<dyn IntoVertexted<HGE_shader_3Dsimple> + Send + Sync>>>,
	_haveUpdate: bool,
	_cacheDatasMem: Option<Subbuffer<[HGE_shader_3Dsimple]>>,
	_cacheIndicesMem: Option<Subbuffer<[u32]>>,
	_cacheIndicesLen: u32,
}

impl HGE_shader_3Dsimple_holder
{
	pub fn insert(&mut self, uuid: cacheInfos, structure: ShaderDrawerImplStruct<impl IntoVertexted<HGE_shader_3Dsimple> + Send + Sync + 'static>)
	{
		ShaderStructHolder_utils::insert(uuid.into(),structure,&self._datas);
		self._haveUpdate = true;
	}
	
	pub fn remove(&mut self, uuid: cacheInfos)
	{
		self._datas.remove(&uuid.into());
		self._haveUpdate = true;
	}
	
	fn compileData(&self) -> (Vec<HGE_shader_3Dsimple>, Vec<u32>, bool)
	{
		let mut vertex = Vec::new();
		let mut indices = Vec::new();
		let mut atleastone = false;
		
		for one in self._datas.iter()
		{
			let mut stop = false;
			let mut tmpvertex = Vec::new();
			let oldindices = vertex.len() as u32;
			for x in &one.vertex {
				let Some(unwraped) = x.IntoVertexted(false) else {
					stop = true;
					break;
				};
				tmpvertex.push(unwraped);
			}
			
			if (!stop)
			{
				vertex.append(&mut tmpvertex);
				for x in &one.indices {
					indices.push(*x + oldindices);
				}
				atleastone = true;
			}
		};
		
		return (vertex, indices, atleastone);
	}
}

impl ShaderStructHolder for HGE_shader_3Dsimple_holder
{
	fn init() -> Self {
		Self{
			_datas: DashMap::new(),
			_cacheDatasMem: None,
			_cacheIndicesMem: None,
			_haveUpdate: false,
			_cacheIndicesLen: 0,
		}
	}
	
	fn pipelineName() -> String {
		names::simple3D.to_string()
	}
	
	fn pipelineNameResolve(&self) -> String {
		Self::pipelineName()
	}
	
	fn reset(&mut self)
	{
		self._datas = DashMap::new();
		self._haveUpdate = false;
		self._cacheIndicesMem = None;
		self._cacheDatasMem = None;
		self._cacheIndicesLen = 0;
	}
	
	fn update(&mut self)
	{
		if (!self._haveUpdate)
		{
			return;
		}
		
		let (vertex, indices, atleastone) = self.compileData();
		if(!atleastone)
		{
			return;
		}
		
		ShaderStructHolder_utils::updateBuffer(vertex,&mut self._cacheDatasMem,BufferCreateInfo {
			usage: BufferUsage::VERTEX_BUFFER,
			..Default::default()
		},AllocationCreateInfo {
			memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
				| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
			..Default::default()
		});
		
		self._cacheIndicesLen = ShaderStructHolder_utils::updateBuffer(indices,&mut self._cacheIndicesMem,BufferCreateInfo {
			usage: BufferUsage::INDEX_BUFFER,
			..Default::default()
		},AllocationCreateInfo {
			memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
				| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
			..Default::default()
		});
		
		self._haveUpdate = false;
	}
	
	fn draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, pipelinename: String)
	{
		let Some(datamem) = &self._cacheDatasMem else {return};
		let Some(indicemem) = &self._cacheIndicesMem else {return};
		
		let Some(pipelineLayout) = ManagerPipeline::singleton().layoutGet(&pipelinename) else { return; };
		if (ManagerShaders::singleton().push_constants(names::simple3D, cmdBuilder, pipelineLayout.clone(), 0) == false)
		{
			return;
		}
		
		for setid in 0..3
		{
			let Some(descriptorCache) = ManagerTexture::singleton().descriptorSet_getVulkanCache(format!("HGE_set{}",setid)) else { return; };
			HTraceError!(cmdBuilder.bind_descriptor_sets(
				PipelineBindPoint::Graphics,
				pipelineLayout.clone(),
				setid,
				descriptorCache,
			));
		}
		
		let lenIndice = self._cacheIndicesLen;
		
		ManagerBuilder::builderAddPipeline(cmdBuilder, &pipelinename);
		
		cmdBuilder
			.bind_vertex_buffers(0, (datamem.clone())).unwrap()
			.bind_index_buffer(indicemem.clone()).unwrap()
			.draw_indexed(lenIndice, 1, 0, 0, 0).unwrap();
		
		if ManagerBuilder::builderAddPipelineTransparency(cmdBuilder, &pipelinename)
		{
			cmdBuilder
				.bind_vertex_buffers(0, (datamem.clone())).unwrap()
				.bind_index_buffer(indicemem.clone()).unwrap()
				.draw_indexed(lenIndice, 1, 0, 0, 0).unwrap();
		}
	}
}
