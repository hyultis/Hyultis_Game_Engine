use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock, RwLock};
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use vulkano::pipeline::PipelineLayout;
use vulkano::shader::ShaderModule;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Shader_type
{
	VERTEX,
	FRAGMENT
}

#[derive(Clone)]
pub struct ShaderContent
{
	pub shader: BTreeMap<Shader_type,Arc<ShaderModule>>,
	pub pushConstant_Func: Arc<dyn Fn(&mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, Arc<PipelineLayout>,u32) + Send + Sync>,
	pub constantFunc: String,
}

pub struct ManagerShaders
{
	_shaders: RwLock<BTreeMap<String,ShaderContent>>
}

static SINGLETON: OnceLock<ManagerShaders> = OnceLock::new();

impl ManagerShaders
{
	fn new() -> ManagerShaders {
		return ManagerShaders {
			_shaders: RwLock::new(BTreeMap::new()),
		};
	}
	
	pub fn singleton() -> &'static ManagerShaders
	{
		return SINGLETON.get_or_init(|| {
			ManagerShaders::new()
		});
	}
	
	pub fn add(&self, name: impl Into<String>,shader: ShaderContent)
	{
		self._shaders.write().unwrap().insert(name.into(),shader);
	}
	
	pub fn get(&self, name: impl Into<String>) -> Option<ShaderContent>
	{
		let name = name.into();
		let tmp = self._shaders.read().unwrap();
		
		return tmp.get(&name).map(|x|x.clone());
	}
	
	pub fn push_constants(&self, name: impl Into<String>,
	                      cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
	                      pipeline_layout: Arc<PipelineLayout>,
	                      offset: u32) -> bool
	{
		let name = name.into();
		match self.get(&name) {
			None => false,
			Some(x) => {
				let tmpfunc = x.pushConstant_Func.clone();
				tmpfunc(cmdBuilder, pipeline_layout,offset);
				true
			}
		}
	}
}
