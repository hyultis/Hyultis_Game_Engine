use std::sync::OnceLock;
use dashmap::DashMap;
use dashmap::mapref::one::RefMut;
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use crate::HGEsubpass::HGEsubpassName;
use crate::Shaders::ShaderStruct::{ShaderStructHolder};

pub struct ShaderDrawer_Manager
{
	_datas: DashMap<String, Box<dyn ShaderStructHolder>>,
	_subpassRegister: DashMap<HGEsubpassName, Vec<String>>
}

static SINGLETON: OnceLock<ShaderDrawer_Manager> = OnceLock::new();

impl ShaderDrawer_Manager
{
	pub fn singleton() -> &'static Self
	{
		return SINGLETON.get_or_init(||{
			Self{
				_datas: DashMap::new(),
				_subpassRegister: Default::default(),
			}
		});
	}
	
	pub fn register<T>(&self, subpass: HGEsubpassName)
		where T: ShaderStructHolder
	{
		let key = T::pipelineName();
		match self._subpassRegister.get_mut(&subpass) {
			None => {self._subpassRegister.insert(subpass,vec![key.clone()]);},
			Some(mut found) => {found.push(key.clone());}
		};
		
		self._datas.insert(key,Box::new(T::init()));
	}
	
	pub fn inspect<T>(&self,func: impl FnOnce(&mut T)) -> bool
		where T: ShaderStructHolder
	{
		let Some(mut tmp) = self.get::<T>() else {return false;};
		if let Some(holder) = tmp.value_mut().downcast_mut::<T>()
		{
			func(holder);
			return true;
		}
		return false;
	}
	
	pub fn get<T>(&self) -> Option<RefMut<String, Box<dyn ShaderStructHolder>>>
		where T: ShaderStructHolder
	{
		return self._datas.get_mut(&T::pipelineName());
	}
	
	pub fn allholder_Update(&self)
	{
		for mut thispipeline in self._datas.iter_mut()
		{
			thispipeline.update();
		}
	}
	
	pub fn holder_Draw(&self, subpass: HGEsubpassName, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>)
	{
		let listof = {
			let Some(listof) = self._subpassRegister.get(&subpass) else {return};
			listof.clone()
		};
		
		for key in listof
		{
			if let Some(thisshader) = self._datas.get(&key)
			{
				thisshader.draw(cmdBuilder,key);
			}
		}
	}
}
