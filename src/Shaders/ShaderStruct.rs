use std::sync::Arc;
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::DynClone;
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;

pub trait ShaderStruct: DynClone + Send + Sync + Downcast {
	fn createPipeline() -> anyhow::Result<()>
		where
			Self: Sized;
}

impl_downcast!(ShaderStruct);
dyn_clone::clone_trait_object!(ShaderStruct);

pub trait ShaderStructHolder: DynClone + Send + Sync + Downcast
{
	fn pipelineName() -> String
	where
		Self: Sized;
	
	fn appendHolder(&mut self, unkownholder: &Box<dyn ShaderStructHolder>);
	fn replaceHolder(&mut self,unkownholder: &Box<dyn ShaderStructHolder>);
	fn reset(&mut self);
	
	fn update(&mut self);
	fn draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>, pipelinename: String);
}

impl_downcast!(ShaderStructHolder);
dyn_clone::clone_trait_object!(ShaderStructHolder);

pub trait ShaderStructInstance: ShaderStruct
{}
