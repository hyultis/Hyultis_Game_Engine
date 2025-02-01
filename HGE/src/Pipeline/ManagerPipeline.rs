use crate::Pipeline::PipelineDatas::PipelineDatas;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use std::sync::{Arc, OnceLock};
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineLayout};
use vulkano::render_pass::RenderPass;

struct updateStruct
{
	pub normal: Box<dyn Fn(Arc<RenderPass>, bool) -> Arc<GraphicsPipeline> + Send + Sync>,
	pub transparency: bool,
	pub topology: PrimitiveTopology,
	pub haveInstance: bool,
}

pub struct ManagerPipeline
{
	_pipelines: DashMap<String, PipelineDatas>,
	_pipelinesTransparency: DashMap<String, PipelineDatas>,
	_updateFunc: DashMap<String, updateStruct>,
}

static SINGLETON: OnceLock<ManagerPipeline> = OnceLock::new();

impl ManagerPipeline
{
	fn new() -> ManagerPipeline
	{
		return ManagerPipeline {
			_pipelines: DashMap::new(),
			_pipelinesTransparency: DashMap::new(),
			_updateFunc: DashMap::new(),
		};
	}

	pub fn singleton() -> &'static ManagerPipeline
	{
		return SINGLETON.get_or_init(|| ManagerPipeline::new());
	}

	pub fn addFunc(
		&self,
		name: impl Into<String>,
		pipeline: impl Fn(Arc<RenderPass>, bool) -> Arc<GraphicsPipeline> + Send + Sync + 'static,
		topology: PrimitiveTopology,
		haveTransparency: bool,
	)
	{
		self._updateFunc.insert(
			name.into(),
			updateStruct {
				normal: Box::new(pipeline),
				transparency: haveTransparency,
				topology,
				haveInstance: false,
			},
		);
	}

	pub fn get(&self, name: impl Into<String>) -> Option<Ref<'_, String, PipelineDatas>>
	{
		return self._pipelines.get(&name.into());
	}

	pub fn getTransparency(&self, name: impl Into<String>) -> Option<Ref<'_, String, PipelineDatas>>
	{
		return self._pipelinesTransparency.get(&name.into());
	}

	pub fn layoutGet(&self, name: impl Into<String>) -> Option<Arc<PipelineLayout>>
	{
		return self.get(name.into()).map(|x| x.pipeline.layout().clone());
	}

	pub fn layoutGetDescriptor(&self, name: impl Into<String>, index: usize) -> Option<Arc<DescriptorSetLayout>>
	{
		let name = name.into();
		if let Some(pipelineDatas) = self.get(name)
		{
			return pipelineDatas.pipeline.layout().set_layouts().get(index).map(|x| x.clone());
		}

		return None;
	}

	pub fn pipelineRefresh(&self, renderpass: Arc<RenderPass>)
	{
		self._updateFunc.iter().for_each(|x| {
			self._pipelines.insert(
				x.key().clone(),
				PipelineDatas {
					primitive_type: x.topology,
					pipeline: (x.normal)(renderpass.clone(), false),
					haveInstance: x.haveInstance,
				},
			);

			if (x.transparency)
			{
				self._pipelinesTransparency.insert(
					x.key().clone(),
					PipelineDatas {
						primitive_type: x.topology,
						pipeline: (x.normal)(renderpass.clone(), true),
						haveInstance: x.haveInstance,
					},
				);
			}
			else
			{
				self._pipelinesTransparency.remove(&x.key().to_string());
			}
		});
	}

	pub fn pipelineGetDebug(&self, name: &str)
	{
		if let Some(pipelineDatas) = self.get(name)
		{
			let desclayout = pipelineDatas.pipeline.layout().set_layouts();
			println!("debug pipeline info : {} - nb set (layout) : {}", name, desclayout.len());
			pipelineDatas.pipeline.layout().push_constant_ranges().iter().for_each(|x| {
				println!("debug pipeline push constant : {} / {}", x.offset, x.size);
			});
			let mut nblayout = 0;
			for x in desclayout.into_iter()
			{
				println!("layout '{}' descriptor_counts : {:?}", nblayout, x.descriptor_counts());
				println!("layout '{}' variable_descriptor_count : {}", nblayout, x.variable_descriptor_count());
				println!("layout '{}' nb binding : {}", nblayout, x.bindings().len());

				for (i, x) in x.bindings().clone()
				{
					println!("layout binding {}.{} is {:?}", nblayout, i, x.descriptor_type);
				}
				nblayout += 1;
			}
		}
	}
}
