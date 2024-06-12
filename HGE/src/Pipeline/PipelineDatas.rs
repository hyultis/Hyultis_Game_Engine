use std::sync::Arc;
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::GraphicsPipeline;

pub struct PipelineDatas
{
	pub primitive_type: PrimitiveTopology,
	pub pipeline: Arc<GraphicsPipeline>,
	pub haveInstance: bool,
}
