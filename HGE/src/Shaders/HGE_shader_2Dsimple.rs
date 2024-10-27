use crate::components::cacheInfos::cacheInfos;
use crate::HGEsubpass::HGEsubpassName;
use crate::ManagerBuilder::ManagerBuilder;
use crate::Pipeline::EnginePipelines;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Shaders::intoVertexed::IntoVertexted;
use crate::Shaders::names;
use crate::Shaders::Manager::ManagerShaders;
use crate::Shaders::ShaderDrawerImpl::ShaderDrawerImplStruct;
use crate::Shaders::ShaderStruct::{ShaderStruct, ShaderStructHolder, ShaderStructHolder_utils};
use crate::Textures::Manager::ManagerTexture;
use anyhow::anyhow;
use arc_swap::ArcSwapOption;
use dashmap::DashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use uuid::Uuid;
use vulkano::buffer::{BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::PipelineBindPoint;
use Htrace::HTraceError;

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
	fn IntoVertexted(&self, _: bool) -> Option<HGE_shader_2Dsimple> {
		let mut textureid = 0;
		
		if let Some(texture) = &self.texture
		{
			let Some(id) = ManagerTexture::singleton().descriptorSet_getIdTexture(["HGE_set0", "HGE_set1", "HGE_set2"], texture.clone()) else { return None; };
			textureid = id.into();
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
		
		ManagerPipeline::singleton().addFunc(HGE_shader_2Dline_holder::pipelineName(), |renderpass, transparency| {
			EnginePipelines::singleton().pipelineCreationLine(names::simple2D,
				transparency,
				renderpass.clone(),
				HGEsubpassName::UI.getSubpassID(),
				HGE_shader_2Dsimple::per_vertex()
			)
		}, PrimitiveTopology::LineList, true);
		
		return Ok(());
	}
}

///////// Holder

pub struct HGE_shader_2Dline_holder
{
	_content: HGE_shader_2Dsimple_holder
}

impl HGE_shader_2Dline_holder
{
	pub fn insert(&self, uuid: cacheInfos, structure: ShaderDrawerImplStruct<impl IntoVertexted<HGE_shader_2Dsimple> + Send + Sync + 'static>)
	{
		self._content.insert(uuid, structure);
	}
	
	pub fn remove(&self, uuid: cacheInfos)
	{
		self._content.remove(uuid);
	}
}

impl ShaderStructHolder for HGE_shader_2Dline_holder
{
	fn init() -> Self {
		Self {
			_content: HGE_shader_2Dsimple_holder::init(),
		}
	}
	
	fn pipelineName() -> String {
		format!("{}_line", HGE_shader_2Dsimple_holder::pipelineName())
	}
	
	fn pipelineNameResolve(&self) -> String {
		Self::pipelineName()
	}
	
	fn reset(&self)
	{
		self._content.reset();
	}
	
	fn update(&self)
	{
		self._content.update();
	}
	
	fn draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, pipelinename: String)
	{
		self._content.draw(cmdBuilder, pipelinename);
	}
}

pub struct HGE_shader_2Dsimple_holder
{
	_datas: DashMap<Uuid, ShaderDrawerImplStruct<Box<dyn IntoVertexted<HGE_shader_2Dsimple> + Send + Sync>>>,
	_haveUpdate: AtomicBool,
	_cacheDatasMem: ArcSwapOption<Subbuffer<[HGE_shader_2Dsimple]>>,
	_cacheIndicesMem: ArcSwapOption<Subbuffer<[u32]>>,
	_cacheIndicesLen: AtomicU32,
}

impl HGE_shader_2Dsimple_holder
{
	pub fn insert(&self, uuid: cacheInfos, structure: ShaderDrawerImplStruct<impl IntoVertexted<HGE_shader_2Dsimple> + Send + Sync + 'static>)
	{
		ShaderStructHolder_utils::insert(uuid.into(), structure, &self._datas);
		self._haveUpdate.store(true, Ordering::Relaxed);
	}
	
	pub fn remove(&self, uuid: cacheInfos)
	{
		self._datas.remove(&uuid.into());
		self._haveUpdate.store(true, Ordering::Relaxed);
	}
	
	fn compileData(&self) -> (Vec<HGE_shader_2Dsimple>, Vec<u32>, bool)
	{
		let mut vertex = Vec::new();
		let mut indices = Vec::new();
		let mut atleastone = false;
		
		for content in self._datas.iter() // (_,one)
		{
			let mut stop = false;
			let mut tmpvertex = Vec::new();
			let oldindices = vertex.len() as u32;
			for x in &content.vertex {
				let Some(unwraped) = x.IntoVertexted(false) else {
					stop = true;
					break;
				};
				tmpvertex.push(unwraped);
			}
			
			if (!stop)
			{
				vertex.append(&mut tmpvertex);
				for x in &content.indices {
					indices.push(*x + oldindices);
				}
				atleastone = true;
			}
		};
		
		return (vertex, indices, atleastone);
	}
}

impl ShaderStructHolder for HGE_shader_2Dsimple_holder
{
	fn init() -> Self {
		Self {
			_datas: DashMap::new(),
			_haveUpdate: AtomicBool::new(false),
			_cacheDatasMem: ArcSwapOption::empty(),
			_cacheIndicesMem: ArcSwapOption::empty(),
			_cacheIndicesLen: AtomicU32::new(0),
		}
	}
	
	fn pipelineName() -> String {
		names::simple2D.to_string()
	}
	
	fn pipelineNameResolve(&self) -> String {
		Self::pipelineName()
	}
	
	fn reset(&self)
	{
		self._datas.clear();
		self._haveUpdate.store(false, Ordering::Relaxed);
		self._cacheIndicesMem.store(None);
		self._cacheDatasMem.store(None);
		self._cacheIndicesLen.store(0, Ordering::Relaxed);
	}
	
	fn update(&self)
	{
		if (!self._haveUpdate.load(Ordering::Relaxed))
		{
			return;
		}
		
		let (vertex, indices, atleastone) = self.compileData();
		if (!atleastone)
		{
			self._cacheDatasMem.store(None);
			self._cacheIndicesMem.store(None);
			self._cacheIndicesLen.store(0, Ordering::Relaxed);
			return;
		}
		
		ShaderStructHolder_utils::updateBuffer(vertex, &self._cacheDatasMem, BufferCreateInfo {
			usage: BufferUsage::VERTEX_BUFFER,
			..Default::default()
		}, AllocationCreateInfo {
			memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
				| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
			..Default::default()
		});
		
		self._cacheIndicesLen.store(ShaderStructHolder_utils::updateBuffer(indices, &self._cacheIndicesMem, BufferCreateInfo {
			usage: BufferUsage::INDEX_BUFFER,
			..Default::default()
		}, AllocationCreateInfo {
			memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
				| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
			..Default::default()
		}), Ordering::Relaxed);
		
		self._haveUpdate.store(false, Ordering::Relaxed);
	}
	
	fn draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, pipelinename: String)
	{
		let Some(datamem) = &*self._cacheDatasMem.load() else { return };
		let Some(indicemem) = &*self._cacheIndicesMem.load() else { return };
		
		let Some(pipelineLayout) = ManagerPipeline::singleton().layoutGet(&pipelinename) else { return; };
		if (ManagerShaders::singleton().push_constants(names::simple2D, cmdBuilder, pipelineLayout.clone(), 0) == false)
		{
			return;
		}
		
		for setid in 0..3
		{
			let Some(descriptorCache) = ManagerTexture::singleton().descriptorSet_getVulkanCache(format!("HGE_set{}", setid)) else { return; };
			HTraceError!(cmdBuilder.bind_descriptor_sets(
				PipelineBindPoint::Graphics,
				pipelineLayout.clone(),
				setid,
				descriptorCache,
			));
		}
		
		let lenIndice = self._cacheIndicesLen.load(Ordering::Relaxed);
		
		ManagerBuilder::builderAddPipeline(cmdBuilder, &pipelinename);
		
		cmdBuilder
			.bind_vertex_buffers(0, ((&**datamem).clone())).unwrap()
			.bind_index_buffer((&**indicemem).clone()).unwrap()
			.draw_indexed(lenIndice, 1, 0, 0, 0).unwrap();
		
		if ManagerBuilder::builderAddPipelineTransparency(cmdBuilder, &pipelinename)
		{
			cmdBuilder
				.bind_vertex_buffers(0, ((&**datamem).clone())).unwrap()
				.bind_index_buffer((&**indicemem).clone()).unwrap()
				.draw_indexed(lenIndice, 1, 0, 0, 0).unwrap();
		}
	}
}
