#[derive(Copy, Clone, Debug)]
pub enum colorBlend
{
	MUL,
	ADD
}

impl colorBlend
{
	pub fn toU32(&self) -> u32
	{
		match self {
			colorBlend::MUL => 0,
			colorBlend::ADD => 1
		}
	}
}

impl Default for colorBlend
{
	fn default() -> Self {
		colorBlend::MUL
	}
}


#[derive(Copy, Clone, Debug)]
pub struct color
{
	pub r: f32,
	pub g: f32,
	pub b: f32,
	pub a: f32,
}

impl Default for color
{
	fn default() -> Self {
		color{
			r: 1.0,
			g: 1.0,
			b: 1.0,
			a: 1.0,
		}
	}
}

impl color
{
	pub fn setRGBu8(&mut self, r: u8, g: u8, b: u8)
	{
		self.r = r as f32/255.0;
		self.g = g as f32/255.0;
		self.b = b as f32/255.0;
	}
	
	pub fn setRGBAu8(&mut self, r: u8, g: u8, b: u8, a:u8)
	{
		self.r = r as f32/255.0;
		self.g = g as f32/255.0;
		self.b = b as f32/255.0;
		self.a = a as f32/255.0;
	}
	
	pub fn getArray(&self) -> [f32; 4]
	{
		return [self.r,self.b,self.g,self.a]
	}
	
	pub fn toArray(&self) -> [f32; 4]
	{
		[self.r,self.g,self.b,self.a]
	}
	
	
	pub fn toArrayu8(&self) -> [u8; 4]
	{
		[
			(self.r*255.0) as u8,
			(self.g*255.0) as u8,
			(self.b*255.0) as u8,
			(self.a*255.0) as u8
		]
	}
	
	/// return interval between this color and a another, using a percent
	pub fn interval(&self,target: color,percent: f32) -> color
	{
		let percent = percent.clamp(0.0,1.0);
		return color{
			r: self.r + ((target.r - self.r) * percent),
			g: self.g + ((target.g - self.g) * percent),
			b: self.b + ((target.b - self.b) * percent),
			a: self.a + ((target.a - self.a) * percent),
		};
	}
	
	pub fn blend(&self,other: color) -> Self
	{
		return color{
			r: self.r * other.r,
			g: self.g * other.g,
			b: self.b * other.b,
			a: self.a * other.a,
		};
	}
	
}

impl From<[f32;4]> for color
{
	fn from(value: [f32; 4]) -> Self {
		color{
			r: value[0],
			g: value[1],
			b: value[2],
			a: value[3],
		}
	}
}

impl From<[u8;4]> for color
{
	fn from(value: [u8; 4]) -> Self {
		color{
			r: value[0] as f32 / 255.0,
			g: value[1] as f32 / 255.0,
			b: value[2] as f32 / 255.0,
			a: value[3] as f32 / 255.0,
		}
	}
}
