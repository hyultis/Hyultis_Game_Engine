use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::pipeline::graphics::vertex_input::Vertex;
use std::convert::TryInto;
use std::sync::Arc;
use ahash::HashMap;
use anyhow::anyhow;
use Htrace::HTraceError;
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::PipelineBindPoint;
use crate::HGEsubpass::HGEsubpassName;
use crate::ManagerBuilder::ManagerBuilder;
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;
use crate::Models3D::chunk_content::chunk_content;
use crate::Pipeline::EnginePipelines;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Shaders::{names};
use crate::Shaders::Manager::ManagerShaders;
use crate::Shaders::ShaderStruct::{ShaderStruct, ShaderStructHolder};
use crate::Shaders::Shs_3DVertex::{HGE_shader_3Dsimple, HGE_shader_3Dsimple_holder};
use crate::Textures::Manager::ManagerTexture;

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
		
		ManagerPipeline::singleton().addFunc(HGE_shader_3Dinstance_holder::pipelineName(), |renderpass,transparency| {
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

#[derive(Clone)]
struct HGE_shader_3Dinstance_subholder
{
	_datas: Vec<HGE_shader_3Dinstance>,
	_indices: Vec<u32>,
	_instance: HashMap<String, HGE_shader_3Dinstance_data>,
	_cacheDatasMem: Option<Subbuffer<[HGE_shader_3Dinstance]>>,
	_cacheIndicesMem: Option<Subbuffer<[u32]>>,
	_cacheInstanceMem: Option<Subbuffer<[HGE_shader_3Dinstance_data]>>
}

#[derive(Clone)]
pub struct HGE_shader_3Dinstance_holder
{
	_activeModel: String,
	_datas: HashMap<String, HGE_shader_3Dinstance_subholder>,
}

impl HGE_shader_3Dinstance_holder
{
	pub fn new(name: impl Into<String>) -> Self
	{
		let name = name.into();
		let mut tmp = Self
		{
			_activeModel: name.clone(),
			_datas: Default::default(),
		};
		
		tmp._datas.insert(name, HGE_shader_3Dinstance_subholder {
			_datas: vec![],
			_indices: vec![],
			_instance: Default::default(),
			_cacheDatasMem: None,
			_cacheIndicesMem: None,
			_cacheInstanceMem: None,
		});
		
		return tmp;
	}
	
	pub fn getName(&self) -> String
	{
		return self._activeModel.clone();
	}
	
	pub fn addInstance(&mut self, name: String, instance: HGE_shader_3Dinstance_data)
	{
		if let Some(this) = self._datas.get_mut(&self._activeModel)
		{
			this._instance.insert(name, instance);
		}
	}
	
	pub fn importModel(&mut self, mut model: impl chunk_content)
	{
		model.cache_update();
		
		if let Some(thispart) = model.cache_get().extractPipeline::<HGE_shader_3Dsimple_holder>()
		{
			self.append(thispart.getDatas(), thispart.getIndices());
		}
	}
	
	fn append(&mut self, vertex: &Vec<HGE_shader_3Dsimple>, indices: &Vec<u32>)
	{
		if let Some(this) = self._datas.get_mut(&self._activeModel)
		{
			let oldmaxindice = this._datas.len() as u32;
			vertex.iter().for_each(|x| {
				this._datas.push(HGE_shader_3Dinstance {
					position: x.position,
					texcoord: x.texcoord,
					color: x.color,
				});
			});
			
			indices.iter().for_each(|x| {
				this._indices.push(*x + oldmaxindice);
			});
		}
	}
}

impl Into<Box<dyn ShaderStructHolder>> for HGE_shader_3Dinstance_holder
{
	fn into(self) -> Box<dyn ShaderStructHolder> {
		Box::new(self)
	}
}

impl ShaderStructHolder for HGE_shader_3Dinstance_holder
{
	fn pipelineName() -> String {
		names::instance3D.to_string()
	}
	
	fn appendHolder(&mut self, unkownholder: &Box<dyn ShaderStructHolder>)
	{
		if let Some(getbackholder) = unkownholder.downcast_ref::<HGE_shader_3Dinstance_holder>()
		{
			getbackholder._datas.iter().for_each(|(name, data)| {
				if (self._datas.contains_key(name))
				{
					if let Some(thisname) = self._datas.get_mut(name)
					{
						data._instance.iter().for_each(|(key, instance)| {
							thisname._instance.insert(key.clone(), instance.clone());
						});
					}
				} else {
					self._datas.insert(name.clone(), data.clone());
				}
			});
		}
	}
	
	fn replaceHolder(&mut self, unkownholder: &Box<dyn ShaderStructHolder>)
	{
		if let Some(getbackholder) = unkownholder.downcast_ref::<HGE_shader_3Dinstance_holder>()
		{
			getbackholder._datas.iter().for_each(|(name, data)| {
				self._datas.insert(name.clone(), data.clone());
			});
		}
	}
	
	fn reset(&mut self) {
		self._datas = Default::default();
	}
	
	fn update(&mut self)
	{
		self._datas.iter_mut().filter(|(_, selfdata)| {
			selfdata._datas.len() != 0 && selfdata._indices.len() != 0
		}).for_each(|(_, selfdata)|
			{
				let lendatas = selfdata._datas.len();
				if (lendatas > 0)
				{
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
						selfdata._datas.clone(),
					).unwrap();
					selfdata._cacheDatasMem = Some(buffer);
				}
				
				let lenindices = selfdata._indices.len();
				if (lenindices > 0)
				{
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
						selfdata._indices.clone(),
					).unwrap();
					selfdata._cacheIndicesMem = Some(buffer);
				}
				
				let leninstance = selfdata._instance.len();
				if (leninstance > 0)
				{
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
						selfdata._instance.clone().into_values().collect::<Vec<HGE_shader_3Dinstance_data>>(),
					).unwrap();
					
					selfdata._cacheInstanceMem = Some(buffer);
				}
			});
	}
	
	fn draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>, pipelinename: String)
	{
		let Some(pipelineLayout) = ManagerPipeline::singleton().layoutGet(&pipelinename) else { return; };
		if(ManagerShaders::singleton().push_constants(names::instance3D, cmdBuilder, pipelineLayout.clone(), 0)==false)
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
		
		self._datas.iter().filter(|(_, selfdata)| {
				selfdata._cacheDatasMem.is_some() && selfdata._cacheIndicesMem.is_some() && selfdata._cacheInstanceMem.is_some()
			}).for_each(|(_, selfdata)|
				{
					
					let datamem = selfdata._cacheDatasMem.clone().unwrap();
					let indicemem = selfdata._cacheIndicesMem.clone().unwrap();
					let instancemem = selfdata._cacheInstanceMem.clone().unwrap();
					let lenIndice = selfdata._indices.len() as u32;
					let lenInstance = selfdata._instance.len() as u32;
					
					ManagerBuilder::builderAddPipeline(cmdBuilder, &pipelinename);
					
					cmdBuilder
						.bind_vertex_buffers(0, (datamem.clone(), instancemem.clone())).unwrap()
						.bind_index_buffer(indicemem.clone()).unwrap()
						.draw_indexed(lenIndice, lenInstance, 0, 0, 0).unwrap();
					
					ManagerBuilder::builderAddPipelineTransparency(cmdBuilder, &pipelinename);
					
					cmdBuilder
						.bind_vertex_buffers(0, (datamem, instancemem)).unwrap()
						.bind_index_buffer(indicemem).unwrap()
						.draw_indexed(lenIndice, lenInstance, 0, 0, 0).unwrap();
				});
	}
}
