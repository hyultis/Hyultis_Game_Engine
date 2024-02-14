use std::fmt::{Debug, Display, Formatter};

pub mod Manager;
pub mod HGE_shader_screen;
pub mod HGE_shader_2Dsimple;
pub mod HGE_shader_3Dsimple;
pub mod ShaderStruct;
pub mod StructAllCache;
pub mod HGE_shader_3Dinstance;

pub enum names
{
	simple3D,
	instance3D,
	simple2D,
	screen, // simple shader of vec2 vertex
}

impl names
{
	pub fn txt(&self) -> &str
	{
		match *self
		{
			names::simple3D => "HGE_3Dsimple",
			names::instance3D => "HGE_3Dinstance",
			names::simple2D => "HGE_2Dsimple",
			names::screen => "HGE_screen",
		}
	}
}

impl Into<String> for names
{
	fn into(self) -> String {
		self.txt().to_string()
	}
}

impl Debug for names
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.txt())
	}
}

impl Display for names
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.txt())
	}
}
