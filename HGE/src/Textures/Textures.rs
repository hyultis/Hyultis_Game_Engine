use crate::Textures::Orders::Order_reload::Order_reload;
use foldhash::HashMap;
use image::RgbaImage;
use vulkano::format::Format;

#[derive(Clone, Copy, Debug)]
pub struct Texture_part
{
	pub uvcoord: [[f32; 2]; 2],
	pub dim: [u32; 2],
}

impl Default for Texture_part
{
	fn default() -> Self
	{
		Texture_part {
			uvcoord: [[0.0, 0.0], [1.0, 1.0]],
			dim: [0, 0],
		}
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TextureState
{
	CREATED,
	BLOCKED,
	LOADED,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TextureStateGPU
{
	NOTSEND,
	SEND,
	UPDATENOTSEND,
}

#[derive(Clone)]
pub struct Texture
{
	pub name: String,

	pub content: Option<RgbaImage>,
	pub width: Option<u32>,
	pub height: Option<u32>,
	pub sampler: String,
	pub mipmap: u32,
	pub format: Format,
	pub partUVCoord: HashMap<String, Texture_part>,
	pub reloadLoader: Option<Order_reload>,
	pub state: TextureState,
	pub clearable: bool,
}

impl Default for Texture
{
	fn default() -> Self
	{
		Texture {
			name: "".to_string(),
			content: None,
			width: None,
			height: None,
			sampler: "default".to_string(),
			mipmap: 1,
			format: Format::R8G8B8A8_UNORM,
			partUVCoord: Default::default(),
			reloadLoader: None,

			state: TextureState::CREATED,
			clearable: false,
		}
	}
}

impl Texture
{
	// return width, height
	pub fn getDim(&self) -> (u32, u32)
	{
		return (
			self.width.unwrap_or(0).clone(),
			self.height.unwrap_or(0).clone(),
		);
	}

	pub fn ratio_w2h(&self) -> f32
	{
		return self.width.unwrap_or(1) as f32 / self.height.unwrap_or(1) as f32;
	}

	pub fn ratio_h2w(&self) -> f32
	{
		return self.height.unwrap_or(1) as f32 / self.width.unwrap_or(1) as f32;
	}

	/// TODO put this somewhere
	pub fn clearContent(&mut self)
	{
		if (self.clearable)
		{
			self.content = None;
		}
	}
}
