use std::any::Any;
use crate::components::corners::corner2;
use crate::components::event::event_trait;
use crate::components::interfacePosition::interfacePosition;
use crate::Interface::UiHitbox::UiHitbox;
use crate::Interface::UiPage::UiPageContent;
use crate::Shaders::Shs_2DVertex::{HGE_shader_2Dsimple, HGE_shader_2Dsimple_holder};
use crate::Shaders::StructAllCache::StructAllCache;

#[derive(Clone)]
pub struct Line
{
	_pos: [interfacePosition; 2],
	_color: [[f32; 4]; 2],
	_cache: StructAllCache,
	_canUpdate: bool
}

impl Line
{
	pub fn setStart(&mut self, newstart: interfacePosition)
	{
		self._pos[0] = newstart;
		self._canUpdate = true;
	}
	
	pub fn setEnd(&mut self, newend: interfacePosition)
	{
		self._pos[1] = newend;
		self._canUpdate = true;
	}
	
	pub fn setColor(&mut self, colors: corner2<[f32; 4]>)
	{
		self._color[0] = colors.start;
		self._color[1] = colors.end;
	}
	
	fn cacheRefresh(&mut self)
	{
		let mut vecstruct = Vec::new();
		vecstruct.push(HGE_shader_2Dsimple {
			position: self._pos[0].convertToVertex(),
			ispixel: self._pos[0].getTypeInt(),
			texture: 0,
			uvcoord: [0.0, 0.0],
			color: self._color[0],
			color_blend_type: 0,
		});
		vecstruct.push(HGE_shader_2Dsimple {
			position: self._pos[1].convertToVertex(),
			ispixel: self._pos[1].getTypeInt(),
			texture: 0,
			uvcoord: [1.0, 1.0],
			color: self._color[1],
			color_blend_type: 0,
		});
		//println!("vecstruct {:?}", vecstruct);
		
		self._cache = StructAllCache::newFromString("interface_line", HGE_shader_2Dsimple_holder::new(vecstruct,[0, 1].to_vec()).into());
		self._canUpdate = false;
	}
}

impl Default for Line
{
	fn default() -> Self {
		return Line
		{
			_pos: [interfacePosition::new_percent(0.0, 0.0), interfacePosition::new_percent(0.0, 0.0)],
			_color: [[0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0]],
			_cache: StructAllCache::new(),
			_canUpdate: true,
		};
	}
}

impl event_trait for Line {}

impl UiPageContent for Line
{
	fn getHitbox(&self) -> UiHitbox {
		UiHitbox::new()
	}
	
	fn cache_isUpdated(&self) -> bool {
		self._canUpdate
	}
	
	fn cache_update(&mut self) {
		self.cacheRefresh();
	}
	
	fn getCache(&self) -> &StructAllCache
	{
		&self._cache
	}
	
	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}
}
