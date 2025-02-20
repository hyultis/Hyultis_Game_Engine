use std::sync::Arc;
use anyhow::anyhow;
use dyn_clone::DynClone;
use Htrace::HTrace;
use image::{GenericImageView, ImageFormat};
use image::io::Reader;
use crate::assetStreamReader::assetManager;
use crate::Textures::Manager::ManagerTexture;
use crate::Textures::Textures::{Texture, TextureState};

pub trait textureLoader: DynClone
{
	fn load(&self) -> anyhow::Result<textureLoader_normalized>;
	fn isWaiting(&mut self) -> bool
	{
		return false;
	}
	
	fn canReload(&self) -> bool
	{
		return true;
	}
}

dyn_clone::clone_trait_object!(textureLoader);

#[derive(Clone)]
pub struct textureLoader_normalized
{
	pub raw: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub clearable: bool
}

impl textureLoader_normalized
{
	pub fn fromTexture(texture: &Texture) -> Self
	{
		match &texture.content {
			None => textureLoader_normalized {
				raw: vec![0,0,0,0],
				width: 1,
				height: 1,
				clearable: true,
			},
			Some(textureContent) =>
				textureLoader_normalized {
					raw: textureContent.to_vec(),
					width: textureContent.width(),
					height: textureContent.height(),
					clearable: texture.clearable,
				}
		}
	}
}


#[derive(Clone)]
pub struct textureLoader_fromFile
{
	pub path: String,
}

impl textureLoader for textureLoader_fromFile
{
	fn load(&self) -> anyhow::Result<textureLoader_normalized>
	{
		//println!("trying loading : {}",self.path);
		let fileread = match assetManager::singleton().readFile(self.path.clone()) {
			None => return Err(anyhow!("cannot load : {}",self.path)),
			Some(x) => x
		};
		
		let im = match Reader::with_format(fileread, ImageFormat::Png).decode() {
			Ok(x) => x,
			Err(err) => return Err(anyhow!("cannot load : {} because {}",self.path,err))
		};
		
		HTrace!("load image : {}", self.path);
		let im = image::DynamicImage::from(im.into_rgba8());
		
		return Ok(textureLoader_normalized {
			width: im.dimensions().0,
			height: im.dimensions().1,
			raw: im.into_bytes(),
			clearable: true,
		});
	}
}

#[derive(Clone)]
pub struct textureLoader_fromRaw
{
	pub raw: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub canReload: bool
}

impl textureLoader_fromRaw
{
	pub fn new() -> Self
	{
		textureLoader_fromRaw {
			raw: vec![],
			width: 0,
			height: 0,
			canReload: true,
		}
	}
}

impl textureLoader for textureLoader_fromRaw
{
	fn load(&self) -> anyhow::Result<textureLoader_normalized>
	{
		return Ok(textureLoader_normalized {
			raw: self.raw.clone(),
			width: self.width.clone(),
			height: self.height.clone(),
			clearable: self.canReload,
		});
	}
	
	fn canReload(&self) -> bool
	{
		return self.canReload;
	}
}


#[derive(Clone)]
pub struct textureLoader_fromCopy
{
	pub name: String,
	pub content: Option<Texture>
}

impl textureLoader for textureLoader_fromCopy
{
	fn load(&self) -> anyhow::Result<textureLoader_normalized>
	{
		return match &self.content {
			None => Err(anyhow!("Texture '{}' not found",self.name)),
			Some(texture) => Ok(textureLoader_normalized::fromTexture(texture))
		};
	}
	
	/// used for check is texture is loaded, and preload data into content (load have texture already loaded, its a thread lock)
	fn isWaiting(&mut self) -> bool
	{
		println!("check is waiting for {}", &self.name);
		let mut tmpreturn = true;
		match ManagerTexture::singleton().get(&self.name) {
			None => {},
			Some(texture) => {
				if (texture.state == TextureState::LOADED)
				{
					if (self.content.is_none())
					{
						self.content = Some(Arc::unwrap_or_clone(texture));
					}
					tmpreturn = false;
				}
			}
		};
		
		println!("check is waiting : {}", tmpreturn);
		return tmpreturn;
	}
}
