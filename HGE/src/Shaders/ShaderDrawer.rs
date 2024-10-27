use crate::HGEsubpass::HGEsubpassName;
use crate::Shaders::ShaderStruct::ShaderStructHolder;
use dashmap::DashMap;
use std::sync::{Arc, OnceLock};
use uuid::Uuid;
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use Htrace::namedThread;

pub struct ShaderDrawer_Manager
{
	_datas: DashMap<String, Arc<dyn ShaderStructHolder>>,
	_subpassRegister: DashMap<HGEsubpassName, Vec<String>>
}

static SINGLETON: OnceLock<ShaderDrawer_Manager> = OnceLock::new();

impl ShaderDrawer_Manager
{
	pub fn singleton() -> &'static Self
	{
		return SINGLETON.get_or_init(|| {
			Self {
				_datas: DashMap::new(),
				_subpassRegister: Default::default(),
			}
		});
	}
	
	pub fn register<T>(&self, subpass: HGEsubpassName)
		where
			T: ShaderStructHolder
	{
		let key = T::pipelineName();
		match self._subpassRegister.get_mut(&subpass) {
			None => { self._subpassRegister.insert(subpass, vec![key.clone()]); },
			Some(mut found) => { found.push(key.clone()); }
		};
		
		self._datas.insert(key, Arc::new(T::init()));
	}
	
	pub fn inspect<T>(func: impl FnOnce(&T) + Send + 'static)
		where
			T: ShaderStructHolder
	{
		let _ = namedThread!(||{
			let Some(tmp) = Self::singleton().get::<T>() else {return};
			if let Some(holder) = tmp.downcast_ref::<T>()
			{
				func(holder);
			};
		});
	}
	
	pub fn get<T>(&self) -> Option<Arc<dyn ShaderStructHolder>>
		where
			T: ShaderStructHolder
	{
		return self._datas.get(&T::pipelineName()).map(|x| x.value().clone());
	}
	
	pub fn allholder_Update()
	{
		let _ = namedThread!(||
		{
			for thispipeline in Self::singleton()._datas.iter()
			{
				thispipeline.value().update();
			}
		});
	}
	
	pub fn holder_Draw(&self, subpass: &HGEsubpassName, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>)
	{
		let Some(x) = self._subpassRegister.get(subpass) else { return };
		let listof = x.value();
		
		for key in listof
		{
			if let Some(thisshader) = self._datas.get(key)
			{
				thisshader.draw(cmdBuilder, key.clone());
			}
		}
	}
	
	pub fn uuid_generate() -> Uuid
	{
		return Uuid::new_v4();
	}
}
