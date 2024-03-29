use std::collections::BTreeMap;
use HArcMut::HArcMut;
use crate::Models3D::chunk_content::chunk_content;

pub struct chunk
{
	_pos: [i32;3],
	_content: BTreeMap<String,HArcMut<Box<dyn chunk_content + Send + Sync>>>,
}

impl chunk
{
	pub fn new(x: i32, y: i32, z:i32) -> Self
	{
		return chunk{
			_pos: [x,y,z],
			_content: Default::default(),
		};
	}
	
	pub fn pos_get(&self) -> [i32;3]
	{
		return self._pos;
	}
	
	pub fn len(&self) -> usize
	{
		self._content.len()
	}
	
	pub fn add(&mut self, name: impl Into<String>,content: impl chunk_content + Clone + Send + Sync + 'static) -> HArcMut<Box<dyn chunk_content + Send + Sync>>
	{
		let tmp: Box<dyn chunk_content + Send + Sync> = Box::new(content);
		let tmp = HArcMut::new(tmp);
		let name: String = name.into();
		self._content.insert(name,tmp.clone());
		return tmp;
	}
	
	pub fn addHAM(&mut self, name: impl Into<String>,content: HArcMut<Box<dyn chunk_content + Send + Sync + 'static>>)
	{
		let name: String = name.into();
		self._content.insert(name,content);
	}
	
	pub fn get(&mut self, name: impl Into<String>) -> Option<HArcMut<Box<dyn chunk_content + Send + Sync>>>
	{
		let name: String = name.into();
		return self._content.get(&name).map(|x|x.clone());
	}
	
	pub fn cache_remove(&mut self)
	{
		self._content.iter_mut()
			.for_each(|(_, elem)| {
				elem.update(|content|content.cache_remove());
			});
	}
	
	pub fn cacheUpdate(&mut self)
	{
		let haveupdate = self._content.iter()
			.any(|(_, elem)| elem.get().cache_mustUpdate() || elem.isWantDrop());
		
		if (!haveupdate)
		{
			return;
		}
		
		let havedrop = self._content.iter()
			.any(|(_, elem)| elem.isWantDrop());
		
		if(havedrop)
		{
			self._content.retain(|_, item| !item.isWantDrop());
		}
			
		self._content.iter().for_each(|(_, elem)| {
			elem.updateIf(|i| {
				let mut haveupdated = false;
				if (i.cache_mustUpdate())
				{
					haveupdated = true;
					i.cache_submit();
				}
				haveupdated
			});
		});
	}
}
