use crate::HGEFrame::HGEFrame;
use crate::HGEMain::HGEMain;
use crate::ManagerBuilder::ManagerBuilder;
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Shaders::names;
use crate::Shaders::HGE_shader_screen::HGE_shader_screen;
use crate::Shaders::Manager::ManagerShaders;
use crate::Shaders::ShaderDrawer::ShaderDrawer_Manager;
use parking_lot::RwLock;
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferInheritanceInfo, CommandBufferInheritanceRenderPassInfo, CommandBufferInheritanceRenderPassType, CommandBufferUsage, PrimaryAutoCommandBuffer, SecondaryAutoCommandBuffer, SubpassBeginInfo, SubpassContents, SubpassEndInfo};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::PipelineBindPoint;
use vulkano::render_pass::{RenderPass, Subpass};
use Htrace::HTraceError;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum HGEsubpassName
{
	UI,
	WORLDSOLID,
	FINAL
}

impl HGEsubpassName
{
	pub fn getSubpassID(&self) -> u32
	{
		match self
		{
			HGEsubpassName::UI => 0,
			HGEsubpassName::WORLDSOLID => 1,
			HGEsubpassName::FINAL => 2
		}
	}
	
	pub fn getByOrder() -> [HGEsubpassName; 3]
	{
		[
			HGEsubpassName::UI,
			HGEsubpassName::WORLDSOLID,
			HGEsubpassName::FINAL
		]
	}
}

pub(crate) struct HGEsubpass
{
	_cacheMemMonoVertex: RwLock<Option<Subbuffer<[HGE_shader_screen]>>>,
	_startApp: Instant
}


static SINGLETON: OnceLock<HGEsubpass> = OnceLock::new();

impl HGEsubpass
{
	fn new() -> Self
	{
		return HGEsubpass {
			_cacheMemMonoVertex: RwLock::new(None),
			_startApp: Instant::now(),
		}
	}
	
	pub fn singleton() -> &'static HGEsubpass
	{
		return SINGLETON.get_or_init(|| {
			HGEsubpass::new()
		});
	}
	
	pub fn ExecAllPass(&self, render_pass: Arc<RenderPass>, mut primaryCommandBuffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, HGEFrameC: &HGEFrame, stdAllocCommand: &StandardCommandBufferAllocator)
	{
		let AllSubpass = HGEsubpassName::getByOrder();
		let length = AllSubpass.len();
		for nbpass in 0..length
		{
			//let lastinstant = Instant::now();
			let thispass = &AllSubpass[nbpass];
			primaryCommandBuffer = primaryCommandBuffer.execute_commands(self.passExec(thispass, render_pass.clone(), HGEFrameC, stdAllocCommand)).unwrap();
			if (nbpass < length - 1)
			{
				primaryCommandBuffer.next_subpass(SubpassEndInfo::default(), SubpassBeginInfo {
					contents: SubpassContents::SecondaryCommandBuffers,
					..SubpassBeginInfo::default()
				}).unwrap();
			}
			//println!("thisubpass {:?} : {}",thispass,lastinstant.elapsed().as_nanos());
		}
	}
	
	fn passExec(&self, thispass: &HGEsubpassName, render_pass: Arc<RenderPass>, HGEFrameC: &HGEFrame, stdAllocCommand: &StandardCommandBufferAllocator) -> Arc<SecondaryAutoCommandBuffer>
	{
		let subpass = Subpass::from(render_pass, thispass.getSubpassID()).unwrap();
		let mut cmdBuilder = AutoCommandBufferBuilder::secondary(
			stdAllocCommand,
			HGEMain::singleton().getDevice().getQueueGraphic().queue_family_index(),
			CommandBufferUsage::OneTimeSubmit,
			CommandBufferInheritanceInfo {
				render_pass: Some(CommandBufferInheritanceRenderPassType::BeginRenderPass(CommandBufferInheritanceRenderPassInfo {
					subpass: {
						subpass
					},
					framebuffer: None,
				})),
				..Default::default()
			}
		).unwrap();
		
		ShaderDrawer_Manager::singleton().holder_Draw(thispass, &mut cmdBuilder);
		
		if (*thispass == HGEsubpassName::FINAL)
		{
			self.pass_Final(&mut cmdBuilder, HGEFrameC)
		};
		
		return ManagerBuilder::builderEnd(cmdBuilder);
	}
	
	fn pass_Final(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, HGEFrameC: &HGEFrame)
	{
		let Ok(descriptor_set) = PersistentDescriptorSet::new(
			&HGEMain::singleton().getAllocatorSet(),
			ManagerPipeline::singleton().layoutGetDescriptor(names::screen, 1).unwrap(),
			[WriteDescriptorSet::image_view(0, HGEFrameC.getImgUI()),
				WriteDescriptorSet::image_view(1, HGEFrameC.getImgWS())],
			[]
		) else { return };
		
		HTraceError!(cmdBuilder.bind_descriptor_sets(
			PipelineBindPoint::Graphics,
			ManagerPipeline::singleton().layoutGet(names::screen).unwrap(),
			1,
			descriptor_set,
		));
		
		if (ManagerShaders::singleton().push_constants(names::screen, cmdBuilder, ManagerPipeline::singleton().layoutGet(names::screen).unwrap(), 0) == false)
		{
			return;
		}
		
		/*
		HGE_rawshader_screen_vert::PushConstants {
				time: self._startApp.elapsed().as_secs_f32().into(),
				rush: HGEMain::singleton().getRushEffect().into(),
				freeze: HGEMain::singleton().getFreezeEffect().into(),
				window: HGEMain::singleton().getWindowInfos().into()
			}
		 */
		
		ManagerBuilder::builderAddPipeline(cmdBuilder, names::screen);
		
		let vertexTPR = self.getMonoVertex();
		let vertexlen = vertexTPR.len();
		cmdBuilder
			.bind_vertex_buffers(0, vertexTPR).unwrap()
			.draw(vertexlen as u32, 1, 0, 0).unwrap();
	}
	
	fn getMonoVertex(&self) -> Subbuffer<[HGE_shader_screen]>
	{
		if ({ self._cacheMemMonoVertex.read().is_none() })
		{
			let mut vertexTPRBinding = self._cacheMemMonoVertex.write();
			
			let vertices = HGE_shader_screen::getDefaultTriangle();
			
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
				vertices,
			).unwrap();
			
			*vertexTPRBinding = Some(buffer);
		}
		
		return self._cacheMemMonoVertex.read().clone().unwrap();
	}
}
