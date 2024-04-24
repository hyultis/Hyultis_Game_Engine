use std::sync::Arc;
use ahash::HashMap;
use image::RgbaImage;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use crate::Textures::Orders::Order_reload::Order_reload;


#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Texture_atlasType
{
	#[deprecated] NONE,
	SMALL,
	LARGE,
	FONT
}

impl Texture_atlasType
{
	pub fn getSetId(&self) -> usize
	{
		match self {
			Texture_atlasType::NONE => 0,
			Texture_atlasType::SMALL => 1,
			Texture_atlasType::LARGE => 2,
			Texture_atlasType::FONT => 0
		}
	}
	
	pub fn getSize(&self) -> u32
	{
		match self {
			Texture_atlasType::NONE => 0,
			#[cfg(target_os = "android")]
			Texture_atlasType::SMALL => 96,
			#[cfg(not(target_os = "android"))]
			Texture_atlasType::SMALL => 128,
			#[cfg(target_os = "android")]
			Texture_atlasType::LARGE => 256,
			#[cfg(not(target_os = "android"))]
			Texture_atlasType::LARGE => 2048,
			Texture_atlasType::FONT => 0
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Texture_part
{
	pub uvcoord: [[f32;2];2],
	pub dim: [u32;2],
}

impl Default for Texture_part
{
	fn default() -> Self {
		Texture_part
		{
			uvcoord: [[0.0,0.0],[1.0,1.0]],
			dim: [0,0],
		}
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TextureState
{
	CREATED,
	BLOCKED,
	LOADED
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TextureStateGPU
{
	NOTSEND,
	SEND,
	UPDATENOTSEND
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
	pub cache: Option<Arc<ImageView>>,
	pub partUVCoord: HashMap<String,Texture_part>,
	pub reloadLoader: Option<Order_reload>,
	
	pub contentSizeAtlas: Option<Vec<u8>>,
	pub atlasType: Texture_atlasType,
	pub atlasId: Option<u32>,
	
	pub state: TextureState,
	pub sendToGpu: TextureStateGPU,
	pub contentClearable: bool,
}

impl Default for Texture
{
	fn default() -> Self {
		Texture{
			name: "".to_string(),
			content: None,
			width: None,
			height: None,
			sampler: "default".to_string(),
			mipmap: 1,
			format: Format::R8G8B8A8_UNORM,
			cache: None,
			partUVCoord: Default::default(),
			reloadLoader: None,
			
			contentSizeAtlas: None,
			atlasType: Texture_atlasType::NONE,
			atlasId: None,
			
			state: TextureState::CREATED,
			sendToGpu: TextureStateGPU::NOTSEND,
			contentClearable: true,
		}
	}
}

impl Texture
{
	// return width, height
	pub fn getDim(&self) -> (u32,u32)
	{
		return (
			self.width.unwrap_or(0).clone(),
			self.height.unwrap_or(0).clone()
		);
	}
	
	pub fn ratio_w2h(&self) -> f32
	{
		return self.width.unwrap_or(1) as f32/self.height.unwrap_or(1) as f32;
	}
	
	pub fn ratio_h2w(&self) -> f32
	{
		return self.height.unwrap_or(1) as f32/self.width.unwrap_or(1) as f32;
	}
	
	
	pub fn clearContent(&mut self)
	{
		if(self.contentClearable)
		{
			self.content = None;
		}
	}
	
	pub fn discharge(&mut self)
	{
		self.cache = None;
		self.state = TextureState::BLOCKED;
		self.sendToGpu = TextureStateGPU::NOTSEND;
	}
}
