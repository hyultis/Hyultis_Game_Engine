use std::time::SystemTime;
use dashmap::DashMap;
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use crate::Shaders::ShaderStruct::ShaderStructHolder;

#[derive(Clone)]
pub struct StructAllCache
{
	_datas: DashMap<String,Box<dyn ShaderStructHolder>>,
	_storedPipeline: Vec<String>,
	_pipeline: String,
	_canUpdateMem: bool,
	_lastUpdate: u128
}

impl PartialEq for StructAllCache
{
	fn eq(&self, other: &Self) -> bool {
		self._pipeline == self._pipeline && self._lastUpdate == other._lastUpdate
	}
}

impl StructAllCache
{
	pub fn new() -> Self
	{
		let defaultpipeline = "".to_string();
		return StructAllCache
		{
			_datas: DashMap::new(),
			_storedPipeline: vec![defaultpipeline.clone()],
			_pipeline: defaultpipeline,
			_canUpdateMem: false,
			_lastUpdate: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos(),
		};
	}
	
	pub fn newFrom<T: ShaderStructHolder>(holder: Box<dyn ShaderStructHolder>) -> Self
	{
		let defaultpipeline = T::pipelineName();
		let datas = DashMap::new();
		datas.insert(defaultpipeline.clone(),holder);
		return StructAllCache
		{
			_datas: datas,
			_storedPipeline: vec![defaultpipeline.clone()],
			_pipeline: defaultpipeline,
			_canUpdateMem: false,
			_lastUpdate: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos(),
		};
	}
	
	pub fn newFromString(name: impl Into<String>, holder: Box<dyn ShaderStructHolder>) -> Self
	{
		let defaultpipeline = name.into();
		let datas = DashMap::new();
		datas.insert(defaultpipeline.clone(),holder);
		return StructAllCache
		{
			_datas: datas,
			_storedPipeline: vec![defaultpipeline.clone()],
			_pipeline: defaultpipeline,
			_canUpdateMem: false,
			_lastUpdate: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos(),
		};
	}
	
	pub fn setPipeline(&mut self, newtype: impl Into<String>)
	{
		let newtype = newtype.into();
		self._pipeline = newtype.clone();
		if(!self._storedPipeline.contains(&newtype))
		{
			self._storedPipeline.push(newtype);
		}
		self._canUpdateMem = false;
		self._lastUpdate = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos();
	}
	
	pub fn getPipelineName(&self) -> &String
	{
		return &self._pipeline;
	}
	
	pub fn reset(&mut self)
	{
		self._datas.iter_mut().for_each(|mut x|{x.reset()});
		self._canUpdateMem = false;
		self._lastUpdate = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos();
	}
	
	pub fn replace(&mut self, other: &StructAllCache)
	{
		for x in &other._storedPipeline
		{
			if(!self._storedPipeline.contains(x))
			{
				self._storedPipeline.push(x.clone());
			}
		}
		for x in &self._storedPipeline {
			if let Some(otherdatas) = other._datas.get(x)
			{
				match self._datas.get_mut(x) {
					None => {self._datas.insert(x.clone(),otherdatas.clone());},
					Some(mut datas) => {
						datas.replaceHolder(otherdatas.value());
					}
				}
			}
		}
		
		self._canUpdateMem = true;
		self._lastUpdate = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos();
	}
	
	pub fn append(&mut self, other: &StructAllCache)
	{
		for x in &other._storedPipeline
		{
			if(!self._storedPipeline.contains(x))
			{
				self._storedPipeline.push(x.clone());
			}
		}
		
		for x in &self._storedPipeline {
			if let Some(otherdatas) = other._datas.get(x)
			{
				match self._datas.get_mut(x) {
					None => {
						self._datas.insert(x.clone(),otherdatas.clone());
					},
					Some(mut datas) => {
						datas.appendHolder(otherdatas.value());
					}
				}
			}
		}
		
		self._canUpdateMem = true;
		self._lastUpdate = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos();
	}
	
	pub fn insertPipeline(&mut self, name: impl Into<String>, holder: Box<dyn ShaderStructHolder>)
	{
		self._datas.insert(name.into(),holder);
	}
	
	pub fn extractPipeline<T: ShaderStructHolder + Clone + 'static>(&self) -> Option<T>
	{
		let pipeline = T::pipelineName();
		if let Some(dataforpipeline) = self._datas.get(&pipeline)
		{
			if let Some(downcast) = dataforpipeline.as_any().downcast_ref::<T>()
			{
				return Some(downcast.clone());
			}
		}
		
		return None;
	}
	
	pub fn isOlderThan(&self, other: &StructAllCache) -> bool
	{
		self._lastUpdate < other._lastUpdate
	}
	
	pub fn getLastUpdateValue(&self) -> u128
	{
		self._lastUpdate
	}
	
	pub fn getAllPipeline(&self) -> &Vec<String>
	{
		&self._storedPipeline
	}
	
	pub fn swapPipeline(&mut self, origin: impl Into<String>, dest: impl Into<String>)
	{
		let origin = origin.into();
		let dest = dest.into();
		if let Some((_,found)) = self._datas.remove(&origin)
		{
			self._datas.insert(dest.clone(),found);
			let mut new = Vec::new();
			for x in &self._storedPipeline
			{
				if(*x!=origin)
				{
					new.push(x.clone());
				}
			}
			new.push(dest);
			self._storedPipeline = new;
		}
	}
	
	pub fn holderUpdate(&mut self)
	{
		for thispipeline in &self._storedPipeline
		{
			if let Some(mut thisholder) = self._datas.get_mut(thispipeline)
			{
				thisholder.update();
			}
		}
	}
	
	pub fn holderDraw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>)
	{
		for thispipeline in &self._storedPipeline
		{
			if let Some(thisholder) = self._datas.get(thispipeline)
			{
				thisholder.draw(cmdBuilder,thispipeline.clone());
			}
		}
	}
}
