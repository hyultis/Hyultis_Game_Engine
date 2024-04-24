use std::default::Default;
use crate::components::corners::corner4;
use crate::components::event::event;
use crate::components::{Components, HGEC_origin, HGEC_rotation, HGEC_scale};
use crate::components::cacheInfos::cacheInfos;
use crate::Interface::UiHitbox::UiHitbox;
use crate::components::color::color;
use crate::components::offset::offset;
use crate::components::rotations::rotation;
use crate::components::scale::scale;

pub mod PlaneImpl2D;
pub mod PlaneImpl3D;

#[derive(Clone)]
pub struct Plane<A>
	where A: HGEC_origin,
	      rotation: HGEC_rotation<A>, scale: HGEC_scale<A>
{
	_components: Components<A>,
	_pos: [A; 4],
	_posHitbox: Option<[A; 4]>,
	_uvcoord: Option<[[f32; 2]; 4]>,
	_color: Option<[color; 4]>,
	_canUpdate: bool,
	_hitbox: UiHitbox,
	_events: event<Plane<A>>,
	_cacheinfos: cacheInfos
}

impl<A> Plane<A>
	where A: HGEC_origin,
	      rotation: HGEC_rotation<A>, scale: HGEC_scale<A>
{
	pub fn new() -> Plane<A>
	{
		return Plane
		{
			_components: Components::default(),
			_pos: [A::default(), A::default(), A::default(), A::default()],
			_posHitbox: None,
			_uvcoord: None,
			_color: None, //[[1.0, 1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 1.0]],
			_canUpdate: true,
			_hitbox: UiHitbox::new(),
			_events: event::new(),
			_cacheinfos: cacheInfos::default(),
		};
	}
	
	pub fn setTexCoord(&mut self, newtexcoord: corner4<[f32; 2]>)
	{
		self._uvcoord = Some(newtexcoord.intoArray());
		self._canUpdate = true;
	}
	
	pub fn getTexCoord(&self)->corner4<[f32; 2]>
	{
		let uvcoord = self._uvcoord.unwrap_or([[0.0,0.0],[1.0,0.0],[0.0,1.0],[1.0,1.0]]);
		
		corner4{
			LeftTop: uvcoord[0],
			RightTop: uvcoord[1],
			LeftBottom: uvcoord[2],
			RightBottom: uvcoord[3],
		}
	}
	
	pub fn setTexCoordSquare(&mut self, leftTop: [f32; 2], bottomright: [f32; 2])
	{
		let tmp = [leftTop,
			[bottomright[0], leftTop[1]],
			[leftTop[0], bottomright[1]],
			bottomright
		];
		self._uvcoord = Some(tmp);
		self._canUpdate = true;
	}
	
	pub fn components(&self) -> &Components<A, rotation, scale, offset<A, rotation, scale>>
	{
		&self._components
	}
	pub fn components_mut(&mut self) -> &mut Components<A, rotation, scale, offset<A, rotation, scale>>
	{
		self._canUpdate = true;
		&mut self._components
	}
	
	pub fn setColor(&mut self, color: corner4<color>)
	{
		self._color = Some(color.intoArray());
		self._canUpdate = true;
	}
	
	pub fn getColor(&self) -> corner4<color>
	{
		match self._color {
			None => {
				corner4{
					LeftTop: color::default(),
					RightTop: color::default(),
					LeftBottom: color::default(),
					RightBottom: color::default(),
				}
			}
			Some(x) => {
				corner4{
					LeftTop: x[0],
					RightTop: x[1],
					LeftBottom: x[2],
					RightBottom: x[3],
				}
			}
		}
		
	}
	
	pub fn getVertexPos(&mut self) -> [A; 4]
	{
		self._pos.clone()
	}
	
	pub fn setVertexPos(&mut self, newpos: corner4<A>)
	{
		self._pos = newpos.intoArray();
		self._canUpdate = true;
	}
	
	/// set a hitbox independent of Vertex plane.
	pub fn setVertexPosHitbox(&mut self, newpos: corner4<A>)
	{
		self._posHitbox = Some(newpos.intoArray());
		self._canUpdate = true;
	}
}
