use cgmath::{Matrix4, Transform};
use derive_more::{Add, AddAssign, Sub, SubAssign};
use crate::components::{HGEC_base, HGEC_origin, HGEC_scale};
use crate::components::interfacePosition::interfacePosition;
use crate::components::worldPosition::worldPosition;

#[derive(Copy, Clone, Debug, Add, AddAssign, Sub, SubAssign)]
pub struct scale
{
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl Default for scale
{
	fn default() -> Self {
		return scale {
			x: 1.0,
			y: 1.0,
			z: 1.0,
		};
	}
}

impl HGEC_base<worldPosition> for scale {
	fn compute(&self, vertex: &mut worldPosition)
	{
		let scale = Matrix4::from_nonuniform_scale(self.x,self.y,self.z);
		let tmp = scale.transform_point(vertex.toPoint3());
		*vertex = worldPosition::new(tmp.x,tmp.y,tmp.z);
	}
}

impl HGEC_scale<worldPosition> for scale
{
	fn get(&self) -> [f32; 3] {
		[self.x,self.y,self.z]
	}
}

impl HGEC_base<interfacePosition> for scale {
	fn compute(&self, vertex: &mut interfacePosition)
	{
		let scale = Matrix4::from_nonuniform_scale(self.x,self.y,self.z);
		let tmp = scale.transform_point(vertex.toPoint3());
		vertex.set([tmp.x,tmp.y,tmp.z]);
	}
}

impl HGEC_scale<interfacePosition> for scale
{
	fn get(&self) -> [f32; 3] {
		[self.x,self.y,self.z]
	}
}
