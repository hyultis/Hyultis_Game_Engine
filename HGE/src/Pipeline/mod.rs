use std::sync::{Arc, OnceLock};
use ahash::HashMapExt;
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::vertex_input::VertexDefinition;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::graphics::color_blend::{AttachmentBlend, BlendFactor, BlendOp, ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::depth_stencil::{CompareOp, DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::{InputAssemblyState, PrimitiveTopology};
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::shader::SpecializationConstant;
use crate::HGEMain::HGEMain;
use crate::HGEsubpass::HGEsubpassName;
use crate::Shaders;
use crate::Shaders::Manager::Shader_type;

pub mod ManagerPipeline;
pub mod PipelineDatas;


pub struct EnginePipelines
{
}

static SINGLETON: OnceLock<EnginePipelines> = OnceLock::new();

impl EnginePipelines
{
	fn new() -> Self
	{
		Self{
		}
	}
	
	pub fn singleton() -> &'static EnginePipelines
	{
		return SINGLETON.get_or_init(|| {
			EnginePipelines::new()
		});
	}
	
	pub fn pipelineCreation(&self,
	                        name :impl Into<String>,
	                        transparency: bool,
	                        renderpass: Arc<RenderPass>,
	                        subpassID: u32,
	                        vertexDef: impl VertexDefinition) -> Arc<GraphicsPipeline>
	{
		let name = name.into();
		
		let dimensions = HGEMain::singleton().getWindowInfos();
		let device = HGEMain::singleton().getDevice().device.clone();
		let subpass = Subpass::from(renderpass, subpassID).unwrap();
		
		let mut specialzitiondata = ahash::HashMap::new();
		specialzitiondata.insert(0,match transparency {
			true => SpecializationConstant::U32(1),
			false => SpecializationConstant::U32(0)
		});
		
		let shadercontent = Shaders::Manager::ManagerShaders::singleton().get(name).unwrap();
		let vertexbinding = shadercontent.shader.get(&Shader_type::VERTEX).unwrap().entry_point("main").unwrap();
		let fragbinding = shadercontent.shader.get(&Shader_type::FRAGMENT).unwrap()
			.specialize(specialzitiondata).unwrap()
			.entry_point("main").unwrap();
		let vertexinputstate= vertexDef.definition(&vertexbinding.info().input_interface).unwrap();
		
		let stages = [
			PipelineShaderStageCreateInfo::new(vertexbinding),
			PipelineShaderStageCreateInfo::new(fragbinding),
		];
		
		let layout = PipelineLayout::new(
			device.clone(),
			PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
				.into_pipeline_layout_create_info(device.clone())
				.unwrap(),
		).unwrap();
		
		
		let mut pipelineCreationInfos = GraphicsPipelineCreateInfo{
			stages: stages.to_vec().into(),
			vertex_input_state: Some(vertexinputstate),
			input_assembly_state: Some(InputAssemblyState::default()),
			viewport_state: Some(ViewportState {
				viewports: [dimensions.ViewPort()].into(),
				..Default::default()
			}),
			rasterization_state: Some(RasterizationState::default()),
			multisample_state: Some(MultisampleState::default()),
			depth_stencil_state: Some(DepthStencilState{
				depth: Some(DepthState::simple()),
				..Default::default()
			}),
			color_blend_state: Some(ColorBlendState::with_attachment_states(
				subpass.num_color_attachments(),
				ColorBlendAttachmentState::default(),
			)),
			subpass: Some(subpass.clone().into()),
			..GraphicsPipelineCreateInfo::layout(layout)
		};
		
		if (transparency)
		{
			pipelineCreationInfos = GraphicsPipelineCreateInfo{
				depth_stencil_state: Some(DepthStencilState{
					depth: Some(DepthState {
						compare_op: CompareOp::Less,
						write_enable: false,
					}),
					..Default::default()
				}),
				color_blend_state: Some(ColorBlendState::with_attachment_states(
					subpass.num_color_attachments(),
					ColorBlendAttachmentState {
						blend: Some(AttachmentBlend{
							color_blend_op: BlendOp::Add,
							src_color_blend_factor: BlendFactor::SrcAlpha, // BlendFactor::SrcAlpha
							dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha, // BlendFactor::OneMinusSrcAlpha
							alpha_blend_op: BlendOp::Max,
							src_alpha_blend_factor: BlendFactor::SrcAlpha,
							dst_alpha_blend_factor: BlendFactor::DstAlpha,
						}),
						..Default::default()
					},
				)),
				..pipelineCreationInfos
			};
		}
		
		return GraphicsPipeline::new(device,None, pipelineCreationInfos).unwrap();
	}
	
	pub fn pipelineCreationLine(&self,
	                            name :impl Into<String>,
	                            transparency: bool,
	                            renderpass: Arc<RenderPass>,
	                            subpassID: u32,
	                            vertexDef: impl VertexDefinition) -> Arc<GraphicsPipeline>
	{
		let name = name.into();
		
		let dimensions = HGEMain::singleton().getWindowInfos();
		let device = HGEMain::singleton().getDevice().device.clone();
		let subpass = Subpass::from(renderpass, subpassID).unwrap();
		
		let mut specialzitiondata = ahash::HashMap::new();
		specialzitiondata.insert(0,match transparency {
			true => SpecializationConstant::U32(1),
			false => SpecializationConstant::U32(0)
		});
		
		let shadercontent = Shaders::Manager::ManagerShaders::singleton().get(name).unwrap();
		let vertexbinding = shadercontent.shader.get(&Shader_type::VERTEX).unwrap().entry_point("main").unwrap();
		let fragbinding = shadercontent.shader.get(&Shader_type::FRAGMENT).unwrap()
			.specialize(specialzitiondata).unwrap()
			.entry_point("main").unwrap();
		let vertexinputstate= vertexDef.definition(&vertexbinding.info().input_interface).unwrap();
		
		let stages = [
			PipelineShaderStageCreateInfo::new(vertexbinding),
			PipelineShaderStageCreateInfo::new(fragbinding)
		];
		
		let layout = PipelineLayout::new(
			device.clone(),
			PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
				.into_pipeline_layout_create_info(device.clone())
				.unwrap(),
		).unwrap();
		
		let mut pipelineCreationInfos = GraphicsPipelineCreateInfo{
			stages: stages.to_vec().into(),
			vertex_input_state: Some(vertexinputstate),
			input_assembly_state: Some(InputAssemblyState{
				topology: PrimitiveTopology::LineList,
				primitive_restart_enable: false,
				..InputAssemblyState::default()
			}),
			viewport_state: Some(ViewportState {
				viewports: [dimensions.ViewPort()].into(),
				..Default::default()
			}),
			rasterization_state: Some(RasterizationState::default()),
			multisample_state: Some(MultisampleState::default()),
			depth_stencil_state: Some(DepthStencilState{
				depth: Some(DepthState::simple()),
				..Default::default()
			}),
			color_blend_state: Some(ColorBlendState::with_attachment_states(
				subpass.num_color_attachments(),
				ColorBlendAttachmentState::default(),
			)),
			subpass: Some(subpass.clone().into()),
			..GraphicsPipelineCreateInfo::layout(layout)
		};
		
		if (transparency)
		{
			pipelineCreationInfos = GraphicsPipelineCreateInfo{
				depth_stencil_state: Some(DepthStencilState{
					depth: Some(DepthState {
						compare_op: CompareOp::Less,
						write_enable: false,
					}),
					..Default::default()
				}),
				color_blend_state: Some(ColorBlendState::with_attachment_states(
					subpass.num_color_attachments(),
					ColorBlendAttachmentState {
						blend: Some(AttachmentBlend{
							color_blend_op: BlendOp::Add,
							src_color_blend_factor: BlendFactor::SrcAlpha, // BlendFactor::SrcAlpha
							dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha, // BlendFactor::OneMinusSrcAlpha
							alpha_blend_op: BlendOp::Max,
							src_alpha_blend_factor: BlendFactor::SrcAlpha,
							dst_alpha_blend_factor: BlendFactor::DstAlpha,
						}),
						..Default::default()
					},
				)),
				..pipelineCreationInfos
			};
		}
		
		return GraphicsPipeline::new(device,None, pipelineCreationInfos).unwrap();
	}
	
	pub fn pipelineCreationScreen(&self,
	                              name :impl Into<String>,
	                              renderpass: Arc<RenderPass>,
	                              vertexDef: impl VertexDefinition) -> Arc<GraphicsPipeline>
	{
		let name = name.into();
		
		let dimensions = HGEMain::singleton().getWindowInfos();
		let device = HGEMain::singleton().getDevice().device.clone();
		let subpass = Subpass::from(renderpass, HGEsubpassName::FINAL.getSubpassID()).unwrap();
		
		let shadercontent = Shaders::Manager::ManagerShaders::singleton().get(name).unwrap();
		let vertexbinding = shadercontent.shader.get(&Shader_type::VERTEX).unwrap().entry_point("main").unwrap();
		let fragbinding = shadercontent.shader.get(&Shader_type::FRAGMENT).unwrap()
			.entry_point("main").unwrap();
		let vertexinputstate= vertexDef.definition(&vertexbinding.info().input_interface).unwrap();
		
		let stages = [
			PipelineShaderStageCreateInfo::new(vertexbinding),
			PipelineShaderStageCreateInfo::new(fragbinding),
		];
		
		let layout = PipelineLayout::new(
			device.clone(),
			PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
				.into_pipeline_layout_create_info(device.clone())
				.unwrap(),
		).unwrap();
		
		let pipelineCreationInfos = GraphicsPipelineCreateInfo{
			stages: stages.to_vec().into(),
			vertex_input_state: Some(vertexinputstate),
			input_assembly_state: Some(InputAssemblyState::default()),
			viewport_state: Some(ViewportState {
				viewports: [dimensions.ViewPort()].into(),
				..Default::default()
			}),
			rasterization_state: Some(RasterizationState::default()),
			multisample_state: Some(MultisampleState::default()),
			depth_stencil_state: None,
			color_blend_state: Some(ColorBlendState::with_attachment_states(
				subpass.num_color_attachments(),
				ColorBlendAttachmentState::default(),
			)),
			subpass: Some(subpass.into()),
			..GraphicsPipelineCreateInfo::layout(layout)
		};
		
		return GraphicsPipeline::new(device,None, pipelineCreationInfos).unwrap();
	}
	
}
