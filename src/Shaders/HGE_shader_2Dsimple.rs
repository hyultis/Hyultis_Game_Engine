use std::collections::BTreeMap;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use std::convert::TryInto;
use std::fmt::Debug;
use std::sync::Arc;
use anyhow::anyhow;
use Htrace::HTraceError;
use uuid::Uuid;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::PipelineBindPoint;
use crate::HGEsubpass::HGEsubpassName;
use crate::ManagerBuilder::ManagerBuilder;
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;
use crate::Pipeline::EnginePipelines;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Shaders::intoVertexed::IntoVertexted;
use crate::Shaders::Manager::ManagerShaders;
use crate::Shaders::names;
use crate::Shaders::ShaderDrawerImpl::ShaderDrawerImplStruct;
use crate::Shaders::ShaderStruct::{ShaderStruct, ShaderStructHolder};
use crate::Textures::Manager::ManagerTexture;

// struct externe, a changer en HGE_shader_2Dsimple
#[derive(Clone, Debug)]
pub struct HGE_shader_2Dsimple_def {
	pub position: [f32; 3],
	pub ispixel: u32,
	pub texture: Option<String>,
	pub uvcoord: [f32; 2],
	pub color: [f32; 4],
	pub color_blend_type: u32 // 0 = mul, 1 = add
}

impl Default for HGE_shader_2Dsimple_def
{
	fn default() -> Self {
		Self {
			position: [0.0, 0.0, 0.0],
			ispixel: 0,
			texture: None,
			uvcoord: [0.0, 0.0],
			color: [1.0, 1.0, 1.0, 1.0],
			color_blend_type: 0,
		}
	}
}

impl IntoVertexted<HGE_shader_2Dsimple> for HGE_shader_2Dsimple_def
{
	fn IntoVertexted(&self, descriptorContext: bool) -> Option<HGE_shader_2Dsimple> {
		let mut textureid = 0;
		
		if let Some(texture) = self.texture.clone()
		{
			let Some(id) = ManagerTexture::singleton().getTextureToId(texture) else { return None; };
			textureid = id;
		}
		
		return Some(HGE_shader_2Dsimple {
			position: self.position,
			ispixel: self.ispixel,
			texture: textureid,
			uvcoord: self.uvcoord,
			color: self.color,
			color_blend_type: self.color_blend_type,
		});
	}
}


// struct internal, a changer en HGE_shader_2Dsimple_raw, remove pub ?
#[repr(C)]
#[derive(Clone, Copy, Debug, Vertex, BufferContents)]
pub struct HGE_shader_2Dsimple {
	#[format(R32G32B32_SFLOAT)]
	pub position: [f32; 3],
	#[format(R32_UINT)]
	pub ispixel: u32,
	#[format(R32_UINT)]
	pub texture: u32,
	#[format(R32G32_SFLOAT)]
	pub uvcoord: [f32; 2],
	#[format(R32G32B32A32_SFLOAT)]
	pub color: [f32; 4],
	#[format(R32_UINT)]
	pub color_blend_type: u32 // 0 = mul, 1 = add
}

impl Default for HGE_shader_2Dsimple
{
	fn default() -> Self {
		Self {
			position: [0.0, 0.0, 0.0],
			ispixel: 0,
			texture: 0,
			uvcoord: [0.0, 0.0],
			color: [1.0, 1.0, 1.0, 1.0],
			color_blend_type: 0,
		}
	}
}

impl ShaderStruct for HGE_shader_2Dsimple {
	fn createPipeline() -> anyhow::Result<()>
	{
		if ManagerShaders::singleton().get(names::instance3D).is_none()
		{
			return Err(anyhow!("missing shader \"{}\"",names::simple2D));
		}
		
		ManagerPipeline::singleton().addFunc(HGE_shader_2Dsimple_holder::pipelineName(), |renderpass, transparency| {
			EnginePipelines::singleton().pipelineCreation(names::simple2D,
				transparency,
				renderpass.clone(),
				HGEsubpassName::UI.getSubpassID(),
				HGE_shader_2Dsimple::per_vertex()
			)
		}, PrimitiveTopology::TriangleList, true);
		
		ManagerPipeline::singleton().addFunc(format!("{}_line", HGE_shader_2Dsimple_holder::pipelineName()), |renderpass, transparency| {
			EnginePipelines::singleton().pipelineCreationLine(names::simple2D,
				transparency,
				renderpass.clone(),
				HGEsubpassName::UI.getSubpassID(),
				HGE_shader_2Dsimple::per_vertex()
			)
		}, PrimitiveTopology::TriangleList, true);
		return Ok(());
	}
}

///////// Holder

#[derive(Clone,Default)]
pub struct HGE_shader_2Dsimple_holder
{
	_datas: BTreeMap<Uuid, ShaderDrawerImplStruct<Arc<dyn IntoVertexted<HGE_shader_2Dsimple> + Send + Sync>>>,
	_haveUpdate: bool,
	_cacheDatasMem: Option<Subbuffer<[HGE_shader_2Dsimple]>>,
	_cacheIndicesMem: Option<Subbuffer<[u32]>>,
	_cacheIndicesLen: u32,
}

impl HGE_shader_2Dsimple_holder
{
	pub fn new() -> Self
	{
		Self {
			_datas: BTreeMap::new(),
			_haveUpdate: false,
			_cacheDatasMem: None,
			_cacheIndicesMem: None,
			_cacheIndicesLen: 0,
		}
	}
	
	pub fn insert(&mut self, uuid: Option<Uuid>, mut structure: ShaderDrawerImplStruct<impl IntoVertexted<HGE_shader_2Dsimple> + Send + Sync + 'static>) -> Uuid
	{
		let mut vertexconvert = Vec::new();
		for x in structure.vertex.drain(0..) {
			let tmp: Arc<dyn IntoVertexted<HGE_shader_2Dsimple> + Send + Sync> = Arc::new(x);
			vertexconvert.push(tmp);
		};
		
		let newstruct = ShaderDrawerImplStruct{
			vertex: vertexconvert,
			indices: structure.indices.clone(),
		};
		
		let uuid = uuid.unwrap_or_else(|| Uuid::new_v4());
		self._datas.insert(uuid, newstruct);
		
		self._haveUpdate = true;
		return uuid;
	}
	
	pub fn remove(&mut self,  uuid: Option<Uuid>)
	{
		let Some(uuid) = uuid else {return};
		self._datas.remove(&uuid);
		self._haveUpdate = true;
	}
	
	fn compileData(&self) -> (Vec<HGE_shader_2Dsimple>, Vec<u32>, bool)
	{
		let mut vertex = Vec::new();
		let mut indices = Vec::new();
		let mut atleastone = false;
		
		for (_, one) in &self._datas
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

impl Into<Box<dyn ShaderStructHolder>> for HGE_shader_2Dsimple_holder
{
	fn into(self) -> Box<dyn ShaderStructHolder> {
		Box::new(self)
	}
}

impl ShaderStructHolder for HGE_shader_2Dsimple_holder
{
	fn init() -> Self {
		Self {
			_datas: BTreeMap::new(),
			_haveUpdate: false,
			_cacheDatasMem: None,
			_cacheIndicesMem: None,
			_cacheIndicesLen: 0,
		}
	}
	
	fn pipelineName() -> String {
		names::simple2D.to_string()
	}
	
	fn reset(&mut self)
	{
		self._datas = BTreeMap::new();
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
		
		let buffer = Buffer::from_iter(
			ManagerMemoryAllocator::singleton().get(),
			BufferCreateInfo {
				usage: BufferUsage::VERTEX_BUFFER,
				..Default::default()
			},
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
					| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
				..Default::default()
			},
			vertex,
		).unwrap();
		self._cacheDatasMem = Some(buffer);
		
		self._cacheIndicesLen = indices.len() as u32;
		let buffer = Buffer::from_iter(
			ManagerMemoryAllocator::singleton().get(),
			BufferCreateInfo {
				usage: BufferUsage::INDEX_BUFFER,
				..Default::default()
			},
			AllocationCreateInfo {
				memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
					| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
				..Default::default()
			},
			indices,
		).unwrap();
		
		self._cacheIndicesMem = Some(buffer);
		self._haveUpdate = false;
	}
	
	fn draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, pipelinename: String)
	{
		let Some(datamem) = &self._cacheDatasMem else {return};
		let Some(indicemem) = &self._cacheIndicesMem else {return};
		
		let Some(pipelineLayout) = ManagerPipeline::singleton().layoutGet(&pipelinename) else { return; };
		if (ManagerShaders::singleton().push_constants(names::simple2D, cmdBuilder, pipelineLayout.clone(), 0) == false)
		{
			return;
		}
		
		let descriptors = ManagerTexture::singleton().getPersistentDescriptorSet();
		descriptors.iter().for_each(|descriptor| {
			let setid = descriptor.key().getSetId() as u32;
			HTraceError!(cmdBuilder.bind_descriptor_sets(
				PipelineBindPoint::Graphics,
				pipelineLayout.clone(),
				setid,
				descriptor.value().clone(),
			));
		});
		
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
