use std::fmt::Debug;
use derive_more::{Add, AddAssign, Sub, SubAssign};
use crate::components::{HGEC_base, HGEC_offset, HGEC_origin, HGEC_rotation, HGEC_scale};
use crate::components::rotations::rotation;
use crate::components::scale::scale;
use crate::components::worldPosition::worldPosition;

#[derive(Copy, Clone, Debug, Default, Add, AddAssign, Sub, SubAssign)]
pub struct offset<A = worldPosition, B = rotation, C = scale>
	where A: HGEC_origin,
	      B: HGEC_rotation<A>,
	      C: HGEC_scale<A>
{
	origin: A,
	rotation: B,
	scale: C
}

impl<A, B, C> HGEC_base<A> for offset<A, B, C>
	where A: HGEC_origin,
	      B: HGEC_rotation<A>,
	      C: HGEC_scale<A>
{
	fn compute(&self, _: &mut A)
	{
	}
}

impl<A, B, C> HGEC_offset<A, B, C> for offset<A, B, C>
	where A: HGEC_origin,
	      B: HGEC_rotation<A>,
	      C: HGEC_scale<A>
{
	fn origin(&self) -> &A {
		&self.origin
	}
	
	fn origin_mut(&mut self) -> &mut A {
		&mut self.origin
	}
	
	fn rotation(&self) -> &B {
		&self.rotation
	}
	
	fn rotation_mut(&mut self) -> &mut B {
		&mut self.rotation
	}
	
	fn scale(&self) -> &C {
		&self.scale
	}
	
	fn scale_mut(&mut self) -> &mut C {
		&mut self.scale
	}
}
