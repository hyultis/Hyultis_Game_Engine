use dashmap::DashMap;
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::DynClone;
use uuid::Uuid;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, Subbuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use vulkano::memory::allocator::AllocationCreateInfo;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;
use crate::Shaders::intoVertexed::IntoVertexted;
use crate::Shaders::ShaderDrawerImpl::ShaderDrawerImplStruct;

pub trait ShaderStruct: DynClone + Send + Sync + Downcast {
	fn createPipeline() -> anyhow::Result<()>
		where
			Self: Sized;
}

impl_downcast!(ShaderStruct);
dyn_clone::clone_trait_object!(ShaderStruct);

pub trait ShaderStructHolder: Send + Sync + Downcast
{
	fn init() -> Self
		where
			Self: Sized;
	
	fn pipelineName() -> String
	where
		Self: Sized;
	
	fn pipelineNameResolve(&self) -> String;
	
	fn reset(&mut self);
	
	fn update(&mut self);
	fn draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, pipelinename: String);
}

impl_downcast!(ShaderStructHolder);

pub trait ShaderStructInstance: ShaderStruct
{}

pub struct ShaderStructHolder_utils{}
impl ShaderStructHolder_utils
{
	pub fn updateBuffer<T>(vertex: Vec<T>, output: &mut Option<Subbuffer<[T]>>, bufferInfos: BufferCreateInfo, allocInfos: AllocationCreateInfo) -> u32
		where T: BufferContents
	{
		let len = vertex.len() as u32;
		if(len==0)
		{
			*output = None;
			return len;
		}
		
		let Ok(buffer) = Buffer::from_iter(
			ManagerMemoryAllocator::singleton().get(),
			bufferInfos,
			allocInfos,
			vertex,
		) else {
			*output = None;
			return 0;
		};
		*output = Some(buffer);
		
		return len;
	}
	
	pub fn insert<T: ShaderStruct + Vertex>(uuid: Uuid, mut structure: ShaderDrawerImplStruct<impl IntoVertexted<T> + Send + Sync + 'static>, array: &DashMap<Uuid, ShaderDrawerImplStruct<Box<dyn IntoVertexted<T> + Send + Sync>>>)
	{
		let mut vertexconvert = Vec::new();
		for x in structure.vertex.drain(0..) {
			let tmp: Box<dyn IntoVertexted<T> + Send + Sync> = Box::new(x);
			vertexconvert.push(tmp);
		};
		
		let newstruct = ShaderDrawerImplStruct{
			vertex: vertexconvert,
			indices: structure.indices.clone(),
		};
		
		array.insert(uuid, newstruct);
	}
}
