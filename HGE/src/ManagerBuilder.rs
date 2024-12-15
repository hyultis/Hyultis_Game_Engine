use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use anyhow::Error;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
	AutoCommandBufferBuilder, CommandBufferInheritanceInfo, CommandBufferUsage,
	RenderPassBeginInfo, SecondaryAutoCommandBuffer, SubpassBeginInfo,
};
use vulkano::device::Queue;
use vulkano::render_pass::Framebuffer;
use Htrace::{HTrace, HTraceError};

struct ManagerBuilderStorage
{
	_commandBuffer: Arc<StandardCommandBufferAllocator>,
	_queue: Arc<Queue>,
	_usage: CommandBufferUsage,
}

pub struct ManagerBuilder
{
	_builders: RwLock<HashMap<String, ManagerBuilderStorage>>,
}

impl ManagerBuilder
{
	pub fn new() -> ManagerBuilder
	{
		ManagerBuilder {
			_builders: RwLock::new(HashMap::new()),
		}
	}

	pub fn add(
		&self,
		name: &str,
		commandBuffer: Arc<StandardCommandBufferAllocator>,
		queue: Arc<Queue>,
		usage: CommandBufferUsage,
	)
	{
		self._builders.write().insert(
			name.to_string(),
			ManagerBuilderStorage {
				_commandBuffer: commandBuffer,
				_queue: queue,
				_usage: usage,
			},
		);
	}

	pub fn generate(
		&self,
		name: &str,
	) -> Option<AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>>
	{
		let tmp = self._builders.read();

		if (tmp.get(&name.to_string()).is_none())
		{
			return None;
		}

		let tmp = tmp.get(&name.to_string()).unwrap();
		let builder = AutoCommandBufferBuilder::secondary(
			tmp._commandBuffer.clone(),
			tmp._queue.queue_family_index(),
			tmp._usage.clone(),
			CommandBufferInheritanceInfo {
				render_pass: None,
				..Default::default()
			},
		)
		.unwrap();

		return Some(builder);
	}

	pub fn builderBegin(
		&self,
		builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
		framebuffers: Arc<Framebuffer>,
	) -> anyhow::Result<()>
	{
		let tmp = builder.begin_render_pass(
			RenderPassBeginInfo {
				clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into()), Some(1f32.into())],
				..RenderPassBeginInfo::framebuffer(framebuffers)
			},
			SubpassBeginInfo::default(),
		);

		if let Err(error) = tmp
		{
			return Err(Error::new(error));
		}

		tmp.unwrap();
		return Ok(());
	}

	pub fn builderAddPipeline(
		builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
		pipelineName: impl Into<String>,
	)
	{
		let pipelineName = pipelineName.into();
		match ManagerPipeline::singleton().get(&pipelineName)
		{
			None =>
			{
				HTrace!(
					"pipeline \"{}\" doesn't exist in ManagerPipeline",
					pipelineName
				);
			}
			Some(pipelineDatas) =>
			{
				HTraceError!(builder.bind_pipeline_graphics(pipelineDatas.pipeline.clone()));
			}
		}
	}

	pub fn builderAddPipelineTransparency(
		builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
		pipelineName: impl Into<String>,
	) -> bool
	{
		let pipelineName = pipelineName.into();
		match ManagerPipeline::singleton().getTransparency(&pipelineName)
		{
			None => false,
			Some(pipelineDatas) =>
			{
				HTraceError!(builder.bind_pipeline_graphics(pipelineDatas.pipeline.clone()));
				true
			}
		}
	}

	pub fn builderEnd(
		builder: AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
	) -> Arc<SecondaryAutoCommandBuffer>
	{
		//builder.end_render_pass().unwrap();

		// Finish building the command buffer by calling `build`.
		return builder.build().unwrap();
	}
}
