use crate::components::cacheInfos::cacheInfos;
use crate::components::color::color;
use crate::components::corners::corner4;
use crate::components::event::{event, event_trait, event_trait_add, event_type};
use crate::components::interfacePosition::interfacePosition;
use crate::components::offset::offset;
use crate::components::rotations::rotation;
use crate::components::scale::scale;
use crate::components::Components;
use crate::entities::utils::entities_utils;
use crate::entities::Plane::Plane;
use crate::Interface::UiHitbox::UiHitbox;
use crate::Interface::UiPage::{UiPageContent, UiPageContent_type};
use crate::Shaders::HGE_shader_2Dsimple::HGE_shader_2Dsimple_holder;
use crate::Shaders::ShaderDrawer::ShaderDrawer_Manager;
use crate::Shaders::ShaderDrawerImpl::{ShaderDrawerImpl, ShaderDrawerImplReturn, ShaderDrawerImplStruct};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Bar_orientation
{
	/// default go right / reverse go left
	HORIZONTAL,
	/// default go top / reverse go bottom
	VERTICAL,
}

#[derive(Clone, Debug)]
pub struct Bar_state
{
	pub color: color,
}

#[derive(Clone)]
pub struct Bar
{
	_components: Components<interfacePosition>,
	_progress: u16,
	_progressState: HashMap<u16, Bar_state>,
	_position: [interfacePosition; 2],
	_textureSize: [f32; 2],
	_orientation: Bar_orientation,
	_events: event<Bar>,
	_hitbox: UiHitbox,
	_cacheinfos: cacheInfos,
}

impl Bar
{
	pub fn new(leftTop: interfacePosition, bottomRight: interfacePosition) -> Bar
	{
		let mut Statesmap = HashMap::new();
		let defaultState = Bar_state { color: color::default() };
		Statesmap.insert(0, defaultState);

		let mut tmp = Bar {
			_components: Components::default(),
			_progress: 0,
			_progressState: Statesmap,
			_position: [interfacePosition::default(), interfacePosition::default()],
			_textureSize: [1.0, 1.0],
			_orientation: Bar_orientation::HORIZONTAL,
			_events: event::new(),
			_hitbox: UiHitbox::new(),
			_cacheinfos: cacheInfos::default(),
		};
		tmp._events.add(event_type::WINREFRESH, event_type::emptyRefresh());
		tmp.setSquare(leftTop, bottomRight);
		return tmp;
	}

	pub fn setTextureSize(&mut self, sizex: f32, sizey: f32)
	{
		self._textureSize = [sizex, sizey];
		self._cacheinfos.setNeedUpdate(true);
	}

	pub fn setOrientation(&mut self, orientation: Bar_orientation)
	{
		self._orientation = orientation;
		self._cacheinfos.setNeedUpdate(true);
	}

	pub fn setSquare(&mut self, leftTop: interfacePosition, bottomRight: interfacePosition)
	{
		self._position = [leftTop, bottomRight];
		self._cacheinfos.setNeedUpdate(true);
	}

	// add a changing state at specific percent progress.
	// progress is a percent where 10000 = 100,00% ( 5675 = 56,75% )
	pub fn addState(&mut self, progress: u16, state: Bar_state)
	{
		self._progressState.insert(progress, state);
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

	// progress is a percent between 0.0 (0%) and 1.0 (100%)
	pub fn updateProgress(&mut self, mut progress: f32)
	{
		progress = progress.clamp(0.0, 1.0);
		let progress = (progress * 10000.0) as u16;
		self._progress = progress;
		self._cacheinfos.setNeedUpdate(true);
	}

	pub fn getProgress(&self) -> f32
	{
		return self._progress as f32 / 10000.0;
	}
}

impl event_trait for Bar
{
	fn event_trigger(&mut self, eventtype: event_type) -> bool
	{
		let update = self._events.clone().trigger(eventtype, self);
		if (self._cacheinfos.isPresent() && update)
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

impl event_trait_add<Bar> for Bar
{
	fn event_add(&mut self, eventtype: event_type, func: impl Fn(&mut Bar) -> bool + Send + Sync + 'static)
	{
		self._events.add(eventtype, func);
	}
}

impl ShaderDrawerImpl for Bar
{
	fn cache_mustUpdate(&self) -> bool
	{
		self._cacheinfos.isNotShow()
	}

	fn cache_infos(&self) -> &cacheInfos
	{
		&self._cacheinfos
	}

	fn cache_infos_mut(&mut self) -> &mut cacheInfos
	{
		&mut self._cacheinfos
	}

	fn cache_submit(&mut self)
	{
		let mut tmp: Vec<_> = self._progressState.keys().copied().collect();
		tmp.sort_by(|x, y| x.cmp(&y));

		//normalise
		let mut Global_lastpos = self._position[0].clone();
		let Global_pos_diffx = self._position[1].getX() - self._position[0].getX();
		let Global_pos_diffy = self._position[1].getY() - self._position[0].getY();

		let mut iter = tmp.iter();
		let mut nextItem = iter.next(); // first stage is always 0;
		let mut newplanes = Vec::new();
		while (nextItem.is_some())
		{
			let start = nextItem.unwrap().clone();
			if (start > self._progress)
			{
				break;
			}
			nextItem = iter.next();
			let end = *nextItem.unwrap_or(&10000);

			let startPercent = start as f32 / 10000.0;
			let mut endPercent = end as f32 / 10000.0;
			let percentDiff = (endPercent - startPercent);
			let mut baseDiff = 1.0 / percentDiff;
			if (baseDiff.is_nan() || baseDiff.is_infinite())
			{
				baseDiff = 1.0;
			}

			if (self._progress < end)
			{
				endPercent = self._progress as f32 / 10000.0;
			}
			let localprogress = (endPercent - startPercent) * baseDiff;

			let mut startUVCoord = [0.0, 0.0];
			let mut endUVCoord = self._textureSize;
			for i in [0, 1]
			{
				if self._textureSize[i] < 0.0
				{
					startUVCoord[i] = self._textureSize[i].abs();
					endUVCoord[i] = 0.0;
				}
			}
			let tmp = self._progressState.get(&start).unwrap();
			let startColor = tmp.color;
			let endColor = self._progressState.get(&end).unwrap_or(tmp).color;
			let startUv;
			let endUv;

			let mut newplane = Plane::new();
			match self._orientation
			{
				Bar_orientation::HORIZONTAL =>
				{
					let local_pos_start = Global_lastpos.clone();
					let mut local_pos_end = Global_lastpos.clone();
					local_pos_end.setX(Global_lastpos.getX() + (Global_pos_diffx * percentDiff * localprogress));
					Global_lastpos.setX(Global_lastpos.getX() + (Global_pos_diffx * percentDiff * localprogress));
					local_pos_end.setY(self._position[1].getY());
					newplane.setSquare(local_pos_start, local_pos_end);

					newplane.setColor(corner4 {
						LeftTop: startColor,
						RightTop: startColor.interval(endColor, localprogress),
						LeftBottom: startColor,
						RightBottom: startColor.interval(endColor, localprogress),
					});

					startUv = [(endUVCoord[0] - startUVCoord[0]) * startPercent, startUVCoord[1]];
					endUv = [(endUVCoord[0] - startUVCoord[0]) * endPercent, endUVCoord[1]];

					//startUv[0]-=(self._progress as f32/10000.0)*2.0;
					//endUv[0]-=(self._progress as f32/10000.0)*2.0;
				}
				Bar_orientation::VERTICAL =>
				{
					let local_pos_start = Global_lastpos.clone();
					let mut local_pos_end = Global_lastpos.clone();
					local_pos_end.setY(Global_lastpos.getY() + (Global_pos_diffy * percentDiff * localprogress));
					Global_lastpos.setY(Global_lastpos.getY() + (Global_pos_diffy * percentDiff * localprogress));
					local_pos_end.setX(self._position[1].getX());
					newplane.setSquare(local_pos_start, local_pos_end);

					newplane.setColor(corner4 {
						LeftTop: startColor,
						RightTop: startColor,
						LeftBottom: startColor.interval(endColor, localprogress),
						RightBottom: startColor.interval(endColor, localprogress),
					});

					startUv = [startUVCoord[0], (endUVCoord[1] - startUVCoord[1]) * startPercent];
					endUv = [endUVCoord[0], (endUVCoord[1] - startUVCoord[1]) * endPercent];
				}
			}

			newplane.setTexCoordSquare(startUv, endUv);
			newplanes.push(newplane);
		}

		let mut datas = ShaderDrawerImplStruct::default();
		for x in newplanes.iter_mut()
		{
			*x.components_mut() = self._components.clone();
			if let Some(mut cache) = x.cache_get()
			{
				datas.combine(&mut cache);
			}
		}

		let tmp = self._cacheinfos;
		if (!ShaderDrawer_Manager::inspect::<HGE_shader_2Dsimple_holder>(move |holder| {
			holder.insert(tmp, datas);
		}))
		{
			return;
		}

		self._cacheinfos.setNeedUpdate(false);
		self._cacheinfos.setPresent();
	}

	fn cache_remove(&mut self)
	{
		let tmp = self._cacheinfos;
		ShaderDrawer_Manager::inspect::<HGE_shader_2Dsimple_holder>(move |holder| {
			holder.remove(tmp);
		});
		self._cacheinfos.setAbsent();
	}
}

impl UiPageContent for Bar
{
	fn getType(&self) -> UiPageContent_type
	{
		if (self._events.haveOneEvent())
		{
			return UiPageContent_type::INTERACTIVE;
		}
		return UiPageContent_type::IDLE;
	}

	fn getHitbox(&self) -> UiHitbox
	{
		self._hitbox.clone()
	}
}

impl entities_utils for Bar
{
	fn cloneAsNew(&self) -> Self
	{
		Self {
			_components: self._components.clone(),
			_progress: self._progress.clone(),
			_progressState: self._progressState.clone(),
			_position: self._position.clone(),
			_textureSize: self._textureSize.clone(),
			_orientation: self._orientation.clone(),
			_events: self._events.clone(),
			_hitbox: self._hitbox.clone(),
			_cacheinfos: Default::default(),
		}
	}
}
