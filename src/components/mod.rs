use std::fmt::Debug;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use cgmath::{Point3, Vector3};

pub mod interfacePosition;
pub mod worldPosition;
pub mod event;
pub mod corners;
pub mod rotations;
pub mod scale;
pub mod offset;
pub mod texture;
pub mod enums;
pub mod uvcoord;
pub mod color;
pub mod hideable;
pub mod window;

pub trait HGEC_base<T>: Clone + Debug + Send + Sync + Default
{
	fn compute(&self, vertex: &mut T);
}

pub trait HGEC_origin: HGEC_base<Self> + Add + AddAssign + Sub + SubAssign
{
    fn get(&self) -> [f32;3];
	fn set(&mut self, new: [f32;3]);
	
	fn toPoint3(&self) -> Point3<f32>;
	fn toVec3(&self) -> Vector3<f32>;
}

pub trait HGEC_rotation<A>: HGEC_base<A> + Add + AddAssign + Sub + SubAssign
{
	fn get(&self) -> [f32;3];
}

pub trait HGEC_scale<A>: HGEC_base<A> + Add + AddAssign + Sub + SubAssign
{
	fn get(&self) -> [f32;3];
}

pub trait HGEC_texture: HGEC_base<Option<texture::texture>>
{
	fn check(&mut self);
}

pub struct componentInstance
{
	pub origin: [f32;3],
	pub scale: [f32;3],
	pub rotation: [f32;3]
}

pub trait HGEC_offset<A,B,C>: HGEC_base<A>
	where A: HGEC_origin,
	      B: HGEC_rotation<A>,
	      C: HGEC_scale<A>
{
	fn origin(&self) -> &A;
	fn origin_mut(&mut self) -> &mut A;
	
	fn rotation(&self) -> &B;
	fn rotation_mut(&mut self) -> &mut B;
	
	fn scale(&self) -> &C;
	fn scale_mut(&mut self) -> &mut C;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Components<A = worldPosition::worldPosition,B = rotations::rotation,C = scale::scale,D = offset::offset<A,B,C>, F = texture::textureAsset>
	where A: HGEC_origin,
	      B: HGEC_rotation<A>,
	      C: HGEC_scale<A>,
	      D: HGEC_offset<A,B,C>,
	      F: HGEC_texture
{
	origin: A,
	rotation: B,
	scale: C,
	offset: D,
	texture: F
}

impl<A, B, C, D, F> Components<A, B, C, D, F>
	where A: HGEC_origin,
	      B: HGEC_rotation<A>,
	      C: HGEC_scale<A>,
	      D: HGEC_offset<A,B,C>,
	      F: HGEC_texture
{
	pub fn computeVertex(&self, vertex: &mut A)
	{
		self.rotation.compute(vertex);
		self.offset.rotation().compute(vertex);
		self.scale.compute(vertex);
		self.offset.scale().compute(vertex);
		self.origin.compute(vertex);
		self.offset.origin().compute(vertex);
	}
	
	pub fn computeTexture(&mut self) -> Option<texture::texture>
	{
		let mut tmp = Some(texture::texture::default());
		self.texture.check();
		self.texture.compute(&mut tmp);
		return tmp;
	}
	
	pub fn computeInstance(&mut self) -> componentInstance
	{
		componentInstance{
			origin: self.origin.get(),
			scale: self.scale.get(),
			rotation: self.rotation.get(),
		}
	}
	
	pub fn origin(&self) -> &A
	{
		return &self.origin;
	}
	pub fn origin_mut(&mut self) -> &mut A
	{
		return &mut self.origin;
	}
	
	
	pub fn rotation(&self) -> &B
	{
		return &self.rotation;
	}
	pub fn rotation_mut(&mut self) -> &mut B
	{
		return &mut self.rotation;
	}
	
	pub fn scale(&self) -> &C
	{
		return &self.scale;
	}
	pub fn scale_mut(&mut self) -> &mut C
	{
		return &mut self.scale;
	}
	
	pub fn offset(&self) -> &D
	{
		return &self.offset;
	}
	pub fn offset_mut(&mut self) -> &mut D
	{
		return &mut self.offset;
	}
	
	pub fn texture(&self) -> &F
	{
		return &self.texture;
	}
	pub fn texture_mut(&mut self) -> &mut F
	{
		return &mut self.texture;
	}
}
