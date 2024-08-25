use std::hash::{Hash, Hasher};
use std::sync::Arc;
use glyph_brush::{OwnedSection, OwnedText};
use glyph_brush_layout::{BuiltInLineBreaker, Layout};
use parking_lot::RwLock;
use crate::components::{Components, HGEC_offset, HGEC_origin};
use crate::components::cacheInfos::cacheInfos;
use crate::HGEMain::HGEMain;
use crate::components::interfacePosition::interfacePosition;
use crate::Interface::ManagerFont::ManagerFont;
use crate::components::event::{event, event_trait, event_trait_add, event_type};
use crate::components::offset::offset;
use crate::components::rotations::rotation;
use crate::components::scale::scale;
use crate::entities::utils::entities_utils;
use crate::Interface::UiButton::UiButton_content;
use crate::Interface::UiHidable::UiHidable_content;
use crate::Interface::UiHitbox::{UiHitbox, UiHitbox_raw};
use crate::Interface::UiPage::{UiPageContent, UiPageContent_type};
use crate::Shaders::HGE_shader_2Dsimple::{HGE_shader_2Dsimple_def, HGE_shader_2Dsimple_holder};
use crate::Shaders::ShaderDrawer::ShaderDrawer_Manager;
use crate::Shaders::ShaderDrawerImpl::{ShaderDrawerImpl, ShaderDrawerImplReturn, ShaderDrawerImplStruct};

#[derive(Clone)]
pub struct Extra
{
	pub color: [f32; 4],
	pub z: f32,
	pub textId: u128,
}

impl Hash for Extra
{
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.textId.hash(state);
		((self.color[0] * 1000000.0) as i32).hash(state);
		((self.color[1] * 1000000.0) as i32).hash(state);
		((self.color[2] * 1000000.0) as i32).hash(state);
		((self.color[3] * 1000000.0) as i32).hash(state);
		((self.z * 1000000.0) as i32).hash(state);
	}
}

impl PartialEq for Extra {
	fn eq(&self, other: &Extra) -> bool {
		self.color[0] - other.color[0] < 1.0e-6 &&
			self.color[1] - other.color[1] < 1.0e-6 &&
			self.color[2] - other.color[2] < 1.0e-6 &&
			self.color[3] - other.color[3] < 1.0e-6 &&
			self.z - other.z < 1.0e-6 &&
			self.textId == other.textId
	}
}

impl Eq for Extra {}

impl Default for Extra
{
	fn default() -> Self {
		Extra
		{
			color: [1.0, 1.0, 1.0, 1.0],
			z: 0.0,
			textId: 0,
		}
	}
}

#[derive(Clone)]
pub enum TextSize
{
	// regular size, depending on size of screen / is mobile
	NORMAL,
	// smaller size than "NORMAL"
	SMALL,
	// smaller size than "SMALL"
	SMALLER,
	// bigger size than "NORMAL"
	BIG,
	// bigger size than "BIG"
	BIGGER,
	// fixed size, usefull if you need to regulary change text, but not its size
	FIX(f32)
}

impl TextSize
{
	pub fn getInt(&self) -> f32
	{
		let dim = HGEMain::singleton().getWindowInfos();
		let size;
		if(dim.isWide)
		{
			size = match self {
				TextSize::SMALLER => dim.heightF / 36.0,
				TextSize::SMALL => dim.heightF / 30.0,
				TextSize::NORMAL => dim.heightF / 24.0,
				TextSize::BIG => dim.heightF / 18.0,
				TextSize::BIGGER => dim.heightF / 12.0,
				TextSize::FIX(u) => u.abs()
			};
		}
		else
		{
			size = match self {
				TextSize::SMALLER => dim.widthF / 18.0,
				TextSize::SMALL => dim.widthF / 16.0,
				TextSize::NORMAL => dim.widthF / 14.0,
				TextSize::BIG => dim.widthF / 12.0,
				TextSize::BIGGER => dim.widthF / 10.0,
				TextSize::FIX(u) => u.abs()
			};
		}
		return size.round();
	}
}

#[derive(Clone)]
pub struct TextCacheUpdater
{
	pub(crate) vertex: Vec<HGE_shader_2Dsimple_def>,
	pub(crate) indices: Vec<u32>,
	pub(crate) isUpdated: bool
}

pub struct Text
{
	_components: Components<interfacePosition>,
	_layout: Layout<BuiltInLineBreaker>,
	_texts: Vec<OwnedText>,
	_textSize: Option<TextSize>,
	_managerfont_textId: u128,
	_cacheShared: Arc<RwLock<TextCacheUpdater>>,
	_isVisible: bool,
	_events: event<Text>,
	_hitbox: UiHitbox,
	_cacheinfos: cacheInfos
}

impl Text
{
	pub fn new() -> Text
	{
		Text
		{
			_components: Components::default(),
			_layout: Layout::default(),
			_texts: vec![],
			_textSize: None,
			_managerfont_textId: ManagerFont::singleton().getUniqId(),
			_cacheShared: Arc::new(RwLock::new(TextCacheUpdater{ vertex: vec![], indices: vec![], isUpdated: false })),
			_isVisible: false,
			_events: Self::newWithWinRefreshEvent(),
			_hitbox: UiHitbox::new(),
			_cacheinfos: cacheInfos::default(),
		}
	}
	
	// add text to section and put visibility on
	pub fn addText(&mut self, newtext: OwnedText)
	{
		self._isVisible = true;
		self._texts.push(newtext);
	}
	
	pub fn getMutText(&mut self) -> &mut Vec<OwnedText>
	{
		&mut self._texts
	}
	
	pub fn setTextDynamicSize(&mut self, size: TextSize)
	{
		self._textSize = Some(size);
	}
	
	// remove all text
	pub fn emptyText(&mut self)
	{
		self._texts = vec![];
		self._isVisible = false;
		ManagerFont::singleton().Text_remove(self._managerfont_textId);
		self._cacheinfos.setNeedUpdate(true);
	}
	
	pub fn setPos(&mut self, pos: interfacePosition)
	{
		*self._components.origin_mut() = pos;
		self._cacheinfos.setNeedUpdate(true);
	}
	
	pub fn setLayout(&mut self, newlayout: Layout<BuiltInLineBreaker>)
	{
		self._layout = newlayout;
	}
	
	pub fn setOffset(&mut self, x: f32, y: f32)
	{
		self._components.offset_mut().origin_mut().set([x,y,0.0]);
		self._cacheinfos.setNeedUpdate(true);
	}
	
	pub fn components(&self) -> &Components<interfacePosition, rotation, scale, offset<interfacePosition, rotation, scale>>
	{
		&self._components
	}
	pub fn components_mut(&mut self) -> &mut Components<interfacePosition, rotation, scale, offset<interfacePosition, rotation, scale>>
	{
		self._cacheinfos.setNeedUpdate(true);
		&mut self._components
	}
	
	// need to be called after any changement, to send update to ManagerFont
	pub fn commit(&mut self)
	{
		if (!self._isVisible)
		{
			return;
		}
		
		let mut tmp = OwnedSection::default();
		for x in self._texts.iter()
		{
			let mut newtext = x.clone().with_extra(Extra {
				color: x.extra.color,
				z: x.extra.z,
				textId: self._managerfont_textId,
			});
			
			if let Some(textsize) = &self._textSize
			{
				newtext = newtext.with_scale(textsize.getInt());
			}
			
			tmp = tmp.add_text(newtext);
		}
		tmp = tmp.with_layout(self._layout);
		
		let tmparccache = self._cacheShared.clone();
		ManagerFont::singleton().Text_add(tmp.to_owned(), move | mut x| {
			let mut lockcache = tmparccache.write();
			x.isUpdated = true;
			*lockcache = x;
		}, self._managerfont_textId);
		
		self._cacheinfos.setNeedUpdate(true);
	}
	
	pub fn setVisible(&mut self)
	{
		self._isVisible = true;
	}
	
	pub fn setHidden(&mut self)
	{
		self._isVisible = false;
	}

	pub fn forceRefresh(&mut self)
	{
		self._isVisible = false;
		self._cacheShared.write().isUpdated = true;
		self._cacheinfos.setNeedUpdate(true);
	}
	
	pub fn getVisible(&self) -> bool
	{
		return self._isVisible;
	}

	fn newWithWinRefreshEvent() -> event<Text>
	{
		let mut tmp = event::new();
		tmp.add(event_type::WINREFRESH, event_type::emptyRefresh());
		return tmp;
	}
}

impl event_trait for Text
{
	fn event_trigger(&mut self, eventtype: event_type) -> bool
	{
		let mut update = self._events.clone().trigger(eventtype, self);
		if (eventtype == event_type::WINREFRESH)
		{
			update = true;
			self.commit();
			self._cacheShared.write().isUpdated = true;
		}
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

impl event_trait_add<Text> for Text
{
	fn event_add(&mut self, eventtype: event_type, func: impl Fn(&mut Text) -> bool + Send + Sync + 'static) {
		self._events.add(eventtype, func);
	}
}

impl Clone for Text
{
	fn clone(&self) -> Self {
		let tmpvec: Vec<_> = self._texts.iter().cloned().collect();
		
		let tmpfinal = Text {
			_components: self._components.clone(),
			_layout: self._layout.clone(),
			_texts: tmpvec,
			_textSize: self._textSize.clone(),
			_managerfont_textId: self._managerfont_textId,
			_cacheShared: self._cacheShared.clone(),
			_isVisible: self._isVisible,
			_events: self._events.clone(),
			_hitbox: self._hitbox.clone(),
			_cacheinfos: self._cacheinfos.clone(),
		};
		
		return tmpfinal;
	}
}

impl ShaderDrawerImpl for Text {
	fn cache_mustUpdate(&self) -> bool {
		self._cacheShared.read().isUpdated || self._cacheinfos.isNotShow()
	}
	
	fn cache_infos(&self) -> &cacheInfos {
		&self._cacheinfos
	}
	
	fn cache_infos_mut(&mut self) -> &mut cacheInfos {
		&mut self._cacheinfos
	}
	
	fn cache_submit(&mut self) {
		if(!self._isVisible)
		{
			let tmp = self._cacheinfos;
			ShaderDrawer_Manager::inspect::<HGE_shader_2Dsimple_holder>(move |holder|{
				holder.remove(tmp);
			});
			self._cacheShared.write().isUpdated = false;
			self._cacheinfos.setNeedUpdate(false);
			self._cacheinfos.setAbsent();
			return;
		}
		
		let Some(structure) = self.cache_get() else {self.cache_remove();return};
		let tmp = self._cacheinfos;
		ShaderDrawer_Manager::inspect::<HGE_shader_2Dsimple_holder>(move |holder|{
			holder.insert(tmp,ShaderDrawerImplStruct{
				vertex: structure.vertex.clone(),
				indices: structure.indices.clone(),
			});
		});
		self._cacheinfos.setPresent();
	}
	
	fn cache_remove(&mut self) {
		let tmp = self._cacheinfos;
		ShaderDrawer_Manager::inspect::<HGE_shader_2Dsimple_holder>(move |holder|{
			holder.remove(tmp);
		});
		self._cacheinfos.setAbsent();
	}
}

impl ShaderDrawerImplReturn<HGE_shader_2Dsimple_def> for Text
{
	fn cache_get(&mut self) -> Option<ShaderDrawerImplStruct<HGE_shader_2Dsimple_def>> {
		let mut structure = {
			let mut tmp = self._cacheShared.write();
			tmp.isUpdated = false;
			tmp.clone()
		};
		let mut hitboxvec = Vec::new();
		
		for vertex in structure.vertex.iter_mut() {
			let mut vertexCorrected = interfacePosition::new_pixel(vertex.position[0] as i32,vertex.position[1] as i32);
			self._components.computeVertex(&mut vertexCorrected);
			vertex.position = vertexCorrected.convertToVertex();
			vertex.ispixel = vertexCorrected.getTypeInt();
			vertex.color[0] = vertex.color[0]*self._components.texture().color().r;
			vertex.color[1] = vertex.color[1]*self._components.texture().color().g;
			vertex.color[2] = vertex.color[2]*self._components.texture().color().b;
			vertex.color[3] = vertex.color[3]*self._components.texture().color().a;
			
			hitboxvec.push(UiHitbox_raw {
				position: vertex.position,
				ispixel: vertex.ispixel==1,
			});
			
		};
		self._hitbox = UiHitbox::newFrom2D(&hitboxvec);
		self._cacheinfos.setNeedUpdate(false);
		
		return Some(ShaderDrawerImplStruct{
			vertex: structure.vertex.drain(0..).collect(),
			indices: structure.indices.drain(0..).collect(),
		});
	}
}

impl UiPageContent for Text
{
	fn getType(&self) -> UiPageContent_type
	{
		return UiPageContent_type::INTERACTIVE;
	}

	fn getHitbox(&self) -> UiHitbox {
		self._hitbox.clone()
	}
}

impl UiButton_content for Text {}
impl UiHidable_content for Text {}

impl entities_utils for Text
{
	fn cloneAsNew(&self) -> Self
	{
		let tmpvec: Vec<_> = self._texts.iter().cloned().collect();
		
		let mut tmpfinal = Text {
			_components: self._components.clone(),
			_layout: self._layout.clone(),
			_texts: tmpvec,
			_textSize: self._textSize.clone(),
			_managerfont_textId: ManagerFont::singleton().getUniqId(),
			_cacheShared: Arc::new(RwLock::new(TextCacheUpdater{ vertex: vec![], indices: vec![], isUpdated: false })),
			_isVisible: self._isVisible,
			_events: Self::newWithWinRefreshEvent(),
			_hitbox: UiHitbox::new(),
			_cacheinfos: cacheInfos::default(),
		};
		tmpfinal.commit();
		
		return tmpfinal;
	}
}
