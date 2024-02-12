use cgmath::{Deg, Euler, Quaternion, Rotation};
use derive_more::{Add, AddAssign, Sub, SubAssign};
use crate::components::{HGEC_base, HGEC_origin, HGEC_rotation};
use crate::components::interfacePosition::interfacePosition;
use crate::components::worldPosition::worldPosition;

#[derive(Copy, Clone, Debug, Add, AddAssign, Sub, SubAssign)]
pub struct rotation
{
	/// pitch is X axis
	pub pitch: Deg<f32>,
	/// yaw is Y axis
	pub yaw: Deg<f32>,
	/// roll is Z axis
	pub roll: Deg<f32>,
}

impl Default for rotation
{
	fn default() -> Self {
		return rotation {
			pitch: Deg(0.0),
			yaw: Deg(0.0),
			roll: Deg(0.0),
		};
	}
}

impl HGEC_base<worldPosition> for rotation {
	fn compute(&self, vertex: &mut worldPosition)
	{
		let rotation = Quaternion::from(Euler {
			x: self.pitch,
			y: self.yaw,
			z: self.roll,
		});
		
		let tmp = rotation.rotate_point(vertex.toPoint3());
		*vertex = worldPosition::new(tmp.x,tmp.y,tmp.z);
	}
}

impl HGEC_rotation<worldPosition> for rotation
{
	fn get(&self) -> [f32; 3] {
		[self.pitch.0,self.yaw.0,self.roll.0]
	}
}

impl HGEC_base<interfacePosition> for rotation {
	fn compute(&self, vertex: &mut interfacePosition) {
		let rotation = Quaternion::from(Euler {
			x: self.pitch,
			y: self.yaw,
			z: self.roll,
		});
		
		let tmp = rotation.rotate_point(vertex.toPoint3());
		vertex.set([tmp.x,tmp.y,tmp.z]);
	}
}

impl HGEC_rotation<interfacePosition> for rotation
{
	fn get(&self) -> [f32; 3] {
		[self.pitch.0,self.yaw.0,self.roll.0]
	}
}
