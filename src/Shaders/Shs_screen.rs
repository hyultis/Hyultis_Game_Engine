use vulkano::pipeline::graphics::vertex_input::Vertex;
use std::convert::TryInto;
use anyhow::anyhow;
use bytemuck::{Pod, Zeroable};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use crate::Pipeline::EnginePipelines;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Shaders::Manager::ManagerShaders;
use crate::Shaders::names;
use crate::Shaders::ShaderStruct::ShaderStruct;

#[repr(C)]
#[derive(Clone, Copy, Debug, Vertex, Pod, Zeroable)]
pub struct HGE_shader_screen {
	#[format(R32G32_SFLOAT)]
	pub position: [f32; 2]
}

impl Default for HGE_shader_screen
{
	fn default() -> Self {
		HGE_shader_screen::new()
	}
}

impl HGE_shader_screen
{
	pub fn new() -> Self
	{
		return HGE_shader_screen
		{
			position: [0.0,0.0]
		};
	}
}

impl ShaderStruct for HGE_shader_screen {
	fn createPipeline() -> anyhow::Result<()>
	{
		if ManagerShaders::singleton().get(names::screen).is_none()
		{
			return Err(anyhow!("missing shader \"{}\"",names::screen));
		}
		
		ManagerPipeline::singleton().addFunc(names::screen, |renderpass,_| {
			EnginePipelines::singleton().pipelineCreationScreen(names::screen, renderpass, HGE_shader_screen::per_vertex())
		}, PrimitiveTopology::TriangleList,false);
		
		return Ok(());
	}
}

impl HGE_shader_screen
{
	pub fn getDefaultTriangle() -> Vec<HGE_shader_screen>
	{
		vec![
			HGE_shader_screen {
				position: [-1.0, -1.0],
			},
			HGE_shader_screen {
				position: [-1.0, 1.0],
			},
			HGE_shader_screen {
				position: [1.0, -1.0],
			},
			HGE_shader_screen {
				position: [1.0, -1.0],
			},
			HGE_shader_screen {
				position: [1.0, 1.0],
			},
			HGE_shader_screen {
				position: [-1.0, 1.0],
			},
		]
	}
}
