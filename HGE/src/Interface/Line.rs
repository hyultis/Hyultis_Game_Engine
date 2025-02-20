use crate::components::cacheInfos::cacheInfos;
use crate::components::corners::corner2;
use crate::components::event::{event, event_trait, event_trait_add, event_type};
use crate::components::interfacePosition::interfacePosition;
use crate::Interface::UiHitbox::UiHitbox;
use crate::Interface::UiPage::UiPageContent;
use crate::Shaders::HGE_shader_2Dsimple::{HGE_shader_2Dline_holder, HGE_shader_2Dsimple_def};
use crate::Shaders::ShaderDrawer::ShaderDrawer_Manager;
use crate::Shaders::ShaderDrawerImpl::{ShaderDrawerImpl, ShaderDrawerImplReturn, ShaderDrawerImplStruct};

#[derive(Clone)]
pub struct Line
{
	_pos: [interfacePosition; 2],
	_color: [[f32; 4]; 2],
	_events: event<Line>,
	_cacheinfos: cacheInfos
}

impl Line
{
	pub fn setStart(&mut self, newstart: interfacePosition)
	{
		self._pos[0] = newstart;
		self._cacheinfos.setNeedUpdate(true);
	}
	
	pub fn setEnd(&mut self, newend: interfacePosition)
	{
		self._pos[1] = newend;
		self._cacheinfos.setNeedUpdate(true);
	}
	
	pub fn setColor(&mut self, colors: corner2<[f32; 4]>)
	{
		self._color[0] = colors.start;
		self._color[1] = colors.end;
	}
}

impl Default for Line
{
	fn default() -> Self {
		let mut event= event::new();
		event.add(event_type::WINREFRESH, event_type::emptyRefresh());
		return Line
		{
			_pos: [interfacePosition::new_percent(0.0, 0.0), interfacePosition::new_percent(0.0, 0.0)],
			_color: [[0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0]],
			_events: event,
			_cacheinfos: cacheInfos::default(),
		};
	}
}

impl event_trait for Line
{
	fn event_trigger(&mut self, eventtype: event_type) -> bool
	{
		let update = self._events.clone().trigger(eventtype, self);
		if(self._cacheinfos.isPresent() && update)
		{
			self.cache_submit();
		}
		return update;
	}
	
	fn event_have(&self, eventtype: event_type) -> bool
	{
		self._events.have(eventtype)
	}
}

impl event_trait_add<Line> for Line
{
	fn event_add(&mut self, eventtype: event_type, func: impl Fn(&mut Line) -> bool + Send + Sync + 'static) {
		self._events.add(eventtype, func);
	}
}

impl ShaderDrawerImpl for Line {
	fn cache_mustUpdate(&self) -> bool
	{
		self._cacheinfos.isNotShow()
	}
	
	fn cache_infos(&self) -> &cacheInfos {
		&self._cacheinfos
	}
	
	fn cache_infos_mut(&mut self) -> &mut cacheInfos {
		&mut self._cacheinfos
	}
	
	fn cache_submit(&mut self)
	{
		let Some(structure) = self.cache_get() else {self.cache_remove();return};
		let tmp = self._cacheinfos;
		ShaderDrawer_Manager::inspect::<HGE_shader_2Dline_holder>(move |holder|{
			holder.insert(tmp,structure);
		});
		self._cacheinfos.setNeedUpdate(false);
		self._cacheinfos.setPresent();
	}
	
	fn cache_remove(&mut self) {
		let tmp = self._cacheinfos;
		ShaderDrawer_Manager::inspect::<HGE_shader_2Dline_holder>(move |holder|{
			holder.remove(tmp);
		});
		self._cacheinfos.setAbsent();
	}
}

impl ShaderDrawerImplReturn<HGE_shader_2Dsimple_def> for Line
{
	fn cache_get(&mut self) -> Option<ShaderDrawerImplStruct<HGE_shader_2Dsimple_def>> {
		let mut vecstruct = Vec::new();
		vecstruct.push(HGE_shader_2Dsimple_def {
			position: self._pos[0].convertToVertex(),
			ispixel: self._pos[0].getTypeInt(),
			color: self._color[0],
			..HGE_shader_2Dsimple_def::default()
		});
		vecstruct.push(HGE_shader_2Dsimple_def {
			position: self._pos[1].convertToVertex(),
			ispixel: self._pos[1].getTypeInt(),
			color: self._color[1],
			..HGE_shader_2Dsimple_def::default()
		});
		
		Some(ShaderDrawerImplStruct{ vertex: vecstruct, indices: vec![0,1] })
	}
}

impl UiPageContent for Line
{
	fn getHitbox(&self) -> UiHitbox {
		UiHitbox::new()
	}
}
