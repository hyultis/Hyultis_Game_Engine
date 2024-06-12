use crate::Textures::Textures::Texture_part;

#[derive(Copy, Clone, Debug)]
pub struct uvcoord
{
	pub left: f32,
	pub top: f32,
	pub right: f32,
	pub bottom: f32,
}

impl uvcoord
{
	pub fn toArray2(&self) -> [[f32; 2]; 2]
	{
		[[self.left,self.top],[self.right,self.bottom]]
	}
	
	pub fn toArray4(&self) -> [[f32; 2]; 4]
	{
		[
			[self.left,self.top],
			[self.right,self.top],
			[self.left,self.bottom],
			[self.right,self.bottom]
		]
	}
	
	/// recalculate 0.0/0.0 - 1.0/1.0 uvcoord, to be inside the local -- TODO
	pub fn recalculateInside(&self, uvcoord : [f32;2]) -> [f32; 2]
	{
		uvcoord
	}
}

impl Default for uvcoord
{
	fn default() -> Self {
		uvcoord{
			left: 0.0,
			top: 0.0,
			right: 1.0,
			bottom: 1.0,
		}
	}
}

impl From<Texture_part> for uvcoord
{
	fn from(value: Texture_part) -> Self {
		uvcoord{
			left: value.uvcoord[0][0],
			top: value.uvcoord[0][1],
			right: value.uvcoord[1][0],
			bottom: value.uvcoord[1][1],
		}
	}
}
