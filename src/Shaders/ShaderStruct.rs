use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::DynClone;
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};

pub trait ShaderStruct: DynClone + Send + Sync + Downcast {
	fn createPipeline() -> anyhow::Result<()>
		where
			Self: Sized;
}

impl_downcast!(ShaderStruct);
dyn_clone::clone_trait_object!(ShaderStruct);

pub trait ShaderStructHolder: DynClone + Send + Sync + Downcast
{
	fn init() -> Self
		where
			Self: Sized;
	
	fn pipelineName() -> String
	where
		Self: Sized;
	
	fn reset(&mut self);
	
	fn update(&mut self);
	fn draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, pipelinename: String);
}

impl_downcast!(ShaderStructHolder);
dyn_clone::clone_trait_object!(ShaderStructHolder);

pub trait ShaderStructInstance: ShaderStruct
{}
