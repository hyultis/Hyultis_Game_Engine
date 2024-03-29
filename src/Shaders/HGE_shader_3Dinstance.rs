use vulkano::buffer::{BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::pipeline::graphics::vertex_input::Vertex;
use std::convert::TryInto;
use ahash::HashMap;
use anyhow::anyhow;
use Htrace::HTraceError;
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::PipelineBindPoint;
use crate::HGEsubpass::HGEsubpassName;
use crate::ManagerBuilder::ManagerBuilder;
use crate::Pipeline::EnginePipelines;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Shaders::names;
use crate::Shaders::Manager::ManagerShaders;
use crate::Shaders::ShaderStruct::{ShaderStruct, ShaderStructHolder, ShaderStructHolder_utils};
use crate::Shaders::HGE_shader_3Dsimple::{HGE_shader_3Dsimple_def};
use crate::Shaders::intoVertexed::IntoVertexted;
use crate::Shaders::ShaderDrawerImpl::ShaderDrawerImplStruct;
use crate::Textures::Manager::ManagerTexture;

impl IntoVertexted<HGE_shader_3Dinstance> for HGE_shader_3Dsimple_def
{
	fn IntoVertexted(&self, _: bool) -> Option<HGE_shader_3Dinstance> {
		return Some(HGE_shader_3Dinstance {
			position: self.position,
			color: self.color,
			texcoord: self.uvcoord,
		});
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Vertex, BufferContents)]
pub struct HGE_shader_3Dinstance
{
	#[format(R32G32B32_SFLOAT)]
	pub position: [f32; 3],
	#[format(R32G32_SFLOAT)]
	pub texcoord: [f32; 2],
	#[format(R32G32B32A32_SFLOAT)]
	pub color: [f32; 4],
}

impl Default for HGE_shader_3Dinstance
{
	fn default() -> Self {
		HGE_shader_3Dinstance {
			position: [0.0, 0.0, 0.0],
			texcoord: [0.0, 0.0],
			color: [1.0, 1.0, 1.0, 1.0],
		}
	}
}

impl ShaderStruct for HGE_shader_3Dinstance
{
	fn createPipeline() -> anyhow::Result<()>
	{
		if ManagerShaders::singleton().get(names::instance3D).is_none()
		{
			return Err(anyhow!("missing shader \"{}\"",names::instance3D));
		}
		
		ManagerPipeline::singleton().addFunc(HGE_shader_3Dinstance_holder::pipelineName(), |renderpass, transparency| {
			EnginePipelines::singleton().pipelineCreation(names::instance3D,
				transparency,
				renderpass.clone(),
				HGEsubpassName::WORLDSOLID.getSubpassID(),
				[HGE_shader_3Dinstance::per_vertex(), HGE_shader_3Dinstance_data::per_instance()]
			)
		}, PrimitiveTopology::TriangleList, true);
		
		return Ok(());
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Vertex, BufferContents)]
pub struct HGE_shader_3Dinstance_data {
	#[format(R32G32B32_SFLOAT)]
	pub instance_offset: [f32; 3],
	#[format(R32G32B32_SFLOAT)]
	pub instance_scale: [f32; 3],
	#[format(R32G32B32_SFLOAT)]
	pub instance_rotation: [f32; 3],
	#[format(R32G32B32A32_SFLOAT)]
	pub instance_color: [f32; 4],
	#[format(R32_UINT)]
	pub instance_texture: u32,
	#[format(R32G32_SFLOAT)]
	pub instance_texcoord_offset: [f32; 2],
}

impl Default for HGE_shader_3Dinstance_data {
	fn default() -> Self {
		HGE_shader_3Dinstance_data {
			instance_offset: [0.0, 0.0, 0.0],
			instance_scale: [1.0, 1.0, 1.0],
			instance_rotation: [0.0, 0.0, 0.0],
			instance_color: [1.0, 1.0, 1.0, 1.0],
			instance_texture: 0,
			instance_texcoord_offset: [0.0, 0.0],
		}
	}
}

struct HGE_shader_3Dinstance_subholder
{
	_model: ShaderDrawerImplStruct<Box<dyn IntoVertexted<HGE_shader_3Dinstance> + Send + Sync>>,
	_haveUpdate: bool,
	_instance: HashMap<String, HGE_shader_3Dinstance_data>,
	_cacheDatasMem: Option<Subbuffer<[HGE_shader_3Dinstance]>>,
	_cacheIndicesMem: Option<Subbuffer<[u32]>>,
	_cacheIndicesLen: u32,
	_cacheInstanceMem: Option<Subbuffer<[HGE_shader_3Dinstance_data]>>,
	_cacheInstanceLen: u32,
}

impl HGE_shader_3Dinstance_subholder
{
	pub fn compileData(&self) -> (Vec<HGE_shader_3Dinstance>, Vec<u32>, bool)
	{
		let mut vertex = Vec::new();
		let mut indices = Vec::new();
		let mut atleastone = false;
		
		let mut stop = false;
		let mut tmpvertex = Vec::new();
		for x in &self._model.vertex {
			let Some(unwraped) = x.IntoVertexted(false) else {
				stop = true;
				break;
			};
			tmpvertex.push(unwraped);
		}
		
		if (!stop)
		{
			vertex.append(&mut tmpvertex);
			for x in &self._model.indices {
				indices.push(*x);
			}
			atleastone = true;
		}
		
		return (vertex, indices, atleastone);
	}
}

///// HOLDER
pub struct HGE_shader_3Dinstance_holder
{
	_haveUpdate: bool,
	_datas: HashMap<String, HGE_shader_3Dinstance_subholder>,
}

impl HGE_shader_3Dinstance_holder
{
	pub fn addInstance(&mut self, modelname: impl Into<String>, instancename: impl Into<String>, instance: HGE_shader_3Dinstance_data)
	{
		let modelname = modelname.into();
		if let Some(this) = self._datas.get_mut(&modelname)
		{
			this._instance.insert(instancename.into(), instance);
			self._haveUpdate = true;
		}
	}
	
	pub fn importModel(&mut self, modelname: impl Into<String>, mut model: ShaderDrawerImplStruct<impl IntoVertexted<HGE_shader_3Dinstance> + Send + Sync + 'static>)
	{
		let modelname = modelname.into();
		if (!self._datas.contains_key(&modelname))
		{
			self._datas.insert(modelname.clone(), HGE_shader_3Dinstance_subholder {
				_model: Default::default(),
				_haveUpdate: false,
				_instance: Default::default(),
				_cacheDatasMem: None,
				_cacheIndicesMem: None,
				_cacheInstanceMem: None,
				_cacheIndicesLen: 0,
				_cacheInstanceLen: 0,
			});
		}
		
		if let Some(this) = self._datas.get_mut(&modelname)
		{
			let mut newvertex = Vec::new();
			
			for x in model.vertex.drain(0..) {
				let tmp: Box<dyn IntoVertexted<HGE_shader_3Dinstance> + Send + Sync> = Box::new(x);
				newvertex.push(tmp);
			}
			
			this._model = ShaderDrawerImplStruct {
				vertex: newvertex,
				indices: model.indices.clone(),
			};
		}
	}
}

impl ShaderStructHolder for HGE_shader_3Dinstance_holder
{
	fn init() -> Self {
		Self {
			_haveUpdate: false,
			_datas: Default::default(),
		}
	}
	
	fn pipelineName() -> String {
		names::instance3D.to_string()
	}
	
	fn pipelineNameResolve(&self) -> String {
		Self::pipelineName()
	}
	
	fn reset(&mut self) {
		self._datas = Default::default();
	}
	
	fn update(&mut self)
	{
		if (!self._haveUpdate)
		{
			return;
		}
		
		let mut haveatleastone = false;
		
		self._datas.iter_mut().filter(|(_, selfdata)| {
			selfdata._model.vertex.len() != 0 && selfdata._model.indices.len() != 0
		}).for_each(|(_, selfdata)|
			{
				let (vertex, indices, atleastone) = selfdata.compileData();
				if(atleastone)
				{
					ShaderStructHolder_utils::updateBuffer(vertex,&mut selfdata._cacheDatasMem,BufferCreateInfo {
						usage: BufferUsage::VERTEX_BUFFER,
						..Default::default()
					},AllocationCreateInfo {
						memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
							| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
						..Default::default()
					});
					
					selfdata._cacheIndicesLen = ShaderStructHolder_utils::updateBuffer(indices,&mut selfdata._cacheIndicesMem,BufferCreateInfo {
						usage: BufferUsage::INDEX_BUFFER,
						..Default::default()
					},AllocationCreateInfo {
						memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
							| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
						..Default::default()
					});
					
					selfdata._cacheInstanceLen = ShaderStructHolder_utils::updateBuffer(selfdata._instance.clone().into_values().collect::<Vec<HGE_shader_3Dinstance_data>>(),&mut selfdata._cacheInstanceMem,BufferCreateInfo {
						usage: BufferUsage::VERTEX_BUFFER,
						..Default::default()
					},AllocationCreateInfo {
						memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
							| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
						..Default::default()
					});
					
					haveatleastone = true;
				}
			});
		
		if(haveatleastone)
		{
			self._haveUpdate = false;
		}
	}
	
	fn draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, pipelinename: String)
	{
		let Some(pipelineLayout) = ManagerPipeline::singleton().layoutGet(&pipelinename) else { return; };
		if (ManagerShaders::singleton().push_constants(names::instance3D, cmdBuilder, pipelineLayout.clone(), 0) == false)
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
				descriptor.value().load_full(),
			));
		});
		
		self._datas.iter().filter(|(_, selfdata)| {
			selfdata._cacheDatasMem.is_some() && selfdata._cacheIndicesMem.is_some() && selfdata._cacheInstanceMem.is_some()
		}).for_each(|(_, selfdata)|
			{
				let datamem = selfdata._cacheDatasMem.clone().unwrap();
				let indicemem = selfdata._cacheIndicesMem.clone().unwrap();
				let instancemem = selfdata._cacheInstanceMem.clone().unwrap();
				
				ManagerBuilder::builderAddPipeline(cmdBuilder, &pipelinename);
				
				cmdBuilder
					.bind_vertex_buffers(0, (datamem.clone(), instancemem.clone())).unwrap()
					.bind_index_buffer(indicemem.clone()).unwrap()
					.draw_indexed(selfdata._cacheIndicesLen, selfdata._cacheInstanceLen, 0, 0, 0).unwrap();
				
				ManagerBuilder::builderAddPipelineTransparency(cmdBuilder, &pipelinename);
				
				cmdBuilder
					.bind_vertex_buffers(0, (datamem, instancemem)).unwrap()
					.bind_index_buffer(indicemem).unwrap()
					.draw_indexed(selfdata._cacheIndicesLen, selfdata._cacheInstanceLen, 0, 0, 0).unwrap();
			});
	}
}
