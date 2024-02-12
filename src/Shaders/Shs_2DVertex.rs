use vulkano::pipeline::graphics::vertex_input::Vertex;
use std::sync::Arc;
use std::convert::TryInto;
use std::fmt::Debug;
use anyhow::anyhow;
use Htrace::HTraceError;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::PipelineBindPoint;
use crate::HGEsubpass::HGEsubpassName;
use crate::ManagerBuilder::ManagerBuilder;
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;
use crate::Pipeline::EnginePipelines;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Shaders::Manager::ManagerShaders;
use crate::Shaders::names;
use crate::Shaders::ShaderStruct::{ShaderStruct, ShaderStructHolder};
use crate::Textures::Manager::ManagerTexture;


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
		
		ManagerPipeline::singleton().addFunc(HGE_shader_2Dsimple_holder::pipelineName(), |renderpass,transparency| {
			EnginePipelines::singleton().pipelineCreation(names::simple2D,
				transparency,
				renderpass.clone(),
				HGEsubpassName::UI.getSubpassID(),
				HGE_shader_2Dsimple::per_vertex()
			)
		}, PrimitiveTopology::TriangleList, true);
		
		ManagerPipeline::singleton().addFunc(format!("{}_line", HGE_shader_2Dsimple_holder::pipelineName()), |renderpass,transparency| {
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

#[derive(Clone, Default)]
pub struct HGE_shader_2Dsimple_holder
{
	_datas: Vec<HGE_shader_2Dsimple>,
	_indices: Vec<u32>,
	_cacheDatasMem: Option<Subbuffer<[HGE_shader_2Dsimple]>>,
	_cacheIndicesMem: Option<Subbuffer<[u32]>>
}

impl HGE_shader_2Dsimple_holder
{
	pub fn new(vertex: Vec<HGE_shader_2Dsimple>, indices: Vec<u32>) -> Self
	{
		Self {
			_datas: vertex,
			_indices: indices,
			_cacheDatasMem: None,
			_cacheIndicesMem: None,
		}
	}
	
	pub fn getVertex(&self) -> Vec<HGE_shader_2Dsimple>
	{
		self._datas.clone()
	}
	
	pub fn getIndices(&self) -> Vec<u32>
	{
		self._indices.clone()
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
	fn pipelineName() -> String {
		names::simple2D.to_string()
	}
	
	fn appendHolder(&mut self, unkownholder: &Box<dyn ShaderStructHolder>)
	{
		if let Some(getbackholder) = unkownholder.downcast_ref::<HGE_shader_2Dsimple_holder>()
		{
			let oldmaxindice = self._datas.len() as u32;
			getbackholder._datas.iter().for_each(|x| {
				self._datas.push(*x);
			});
			getbackholder._indices.iter().for_each(|x| {
				self._indices.push(*x + oldmaxindice);
			});
		}
	}
	
	fn replaceHolder(&mut self, unkownholder: &Box<dyn ShaderStructHolder>)
	{
		if let Some(getbackholder) = unkownholder.downcast_ref::<HGE_shader_2Dsimple_holder>()
		{
			self._datas = getbackholder._datas.clone();
			self._indices = getbackholder._indices.clone();
		}
	}
	
	fn reset(&mut self)
	{
		self._indices = Vec::new();
		self._datas = Vec::new();
		self._cacheIndicesMem = None;
		self._cacheDatasMem = None;
	}
	
	fn update(&mut self)
	{
		let lendatas = self._datas.len();
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
				self._datas.clone(),
			).unwrap();
			
			self._cacheDatasMem = Some(buffer);
		}
		
		let lenindices = self._indices.len();
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
				self._indices.clone(),
			).unwrap();
			
			self._cacheIndicesMem = Some(buffer);
		}
	}
	
	fn draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>, pipelinename: String)
	{
		if (self._cacheDatasMem.is_none() || self._cacheIndicesMem.is_none())
		{
			return;
		}
		
		let Some(pipelineLayout) = ManagerPipeline::singleton().layoutGet(&pipelinename) else { return; };
		if(ManagerShaders::singleton().push_constants(names::simple2D, cmdBuilder, pipelineLayout.clone(), 0)==false)
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
		
		let datamem = self._cacheDatasMem.clone().unwrap();
		let indicemem = self._cacheIndicesMem.clone().unwrap();
		let lenIndice = self._indices.len() as u32;
		
		ManagerBuilder::builderAddPipeline(cmdBuilder, &pipelinename);
		
		cmdBuilder
			.bind_vertex_buffers(0, (datamem.clone())).unwrap()
			.bind_index_buffer(indicemem.clone()).unwrap()
			.draw_indexed(lenIndice, 1, 0, 0, 0).unwrap();
		
		if ManagerBuilder::builderAddPipelineTransparency(cmdBuilder, &pipelinename)
		{
			let lenIndice = self._indices.len() as u32;
			cmdBuilder
				.bind_vertex_buffers(0, (datamem)).unwrap()
				.bind_index_buffer(indicemem).unwrap()
				.draw_indexed(lenIndice, 1, 0, 0, 0).unwrap();
		}
	}
}
