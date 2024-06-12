use cgmath::{Matrix4, Point3, Transform, Vector3};
use derive_more::{Add, AddAssign, Sub, SubAssign};
use crate::components::{HGEC_base, HGEC_origin};

/// simple storage of 3D position
#[derive(Copy, Clone, Debug, Add, AddAssign, Sub, SubAssign)]
pub struct worldPosition
{
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl worldPosition
{
	pub fn new(x: f32,y: f32,z: f32) -> Self
	{
		return worldPosition{
			x,
			y,
			z,
		};
	}
}

impl HGEC_base<worldPosition> for worldPosition
{
	fn compute(&self, vertex: &mut worldPosition) {
		let offset = Matrix4::from_translation(Vector3::new(self.x,self.y,self.z));
		let tmp = offset.transform_point(vertex.toPoint3());
		*vertex = worldPosition::new(tmp.x,tmp.y,tmp.z);
	}
}

impl HGEC_origin for worldPosition
{
    fn get(&self) -> [f32; 3]
    {
        [self.x,self.y,self.z]
    }
	
	fn set(&mut self, new: [f32; 3]) {
		self.x = new[0];
		self.y = new[1];
		self.z = new[2];
	}
	
	fn toPoint3(&self) -> Point3<f32>
	{
		return Point3::new(self.x,self.y,self.z);
	}
	fn toVec3(&self) -> Vector3<f32>
	{
		return Vector3::new(self.x,self.y,self.z);
	}
	
}

impl Default for worldPosition
{
    fn default() -> Self {
        worldPosition
        {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}
