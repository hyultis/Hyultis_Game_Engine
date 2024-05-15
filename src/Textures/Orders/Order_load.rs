use Htrace::HTrace;
use image::RgbaImage;
use vulkano::format::Format;
use crate::Textures::Filter::Filter;
use crate::Textures::Order::Order;
use crate::Textures::Orders::Order_reload::Order_reload;
use crate::Textures::textureLoader::textureLoader;
use crate::Textures::Textures::{Texture, TextureState};

#[derive(Clone)]
pub struct Order_load
{
	pub from: Box<dyn textureLoader + Send + Sync>,
	pub filter: Vec<Box<dyn Filter + Send + Sync>>,
	pub mipmap: u32,
	pub format: Format,
	pub sameThread: bool,
}

impl Order_load
{
	pub fn new(loader: impl textureLoader + Send + Sync + 'static) -> Self
	{
		return Order_load {
			from: Box::new(loader),
			filter: vec![],
			mipmap: 1,
			format: Format::R8G8B8A8_UNORM,
			sameThread: false,
		};
	}
	
	pub fn newPrioritize(loader: impl textureLoader + Send + Sync + 'static) -> Self
	{
		return Order_load {
			from: Box::new(loader),
			filter: vec![],
			mipmap: 1,
			format: Format::R8G8B8A8_UNORM,
			sameThread: true,
		};
	}
	
	pub fn filter_add(mut self, newfilter: impl Filter + Send + Sync + 'static) -> Self
	{
		self.filter.push(Box::new(newfilter));
		return self;
	}
	
}

impl Order for Order_load
{
	fn exec(&self, texture: &mut Texture)
	{
		let resultLoad = self.from.load();
		if (resultLoad.is_err())
		{
			HTrace!("texture is blocked for : {}", resultLoad.err().unwrap());
			texture.state = TextureState::BLOCKED;
			return;
		}
		let mut data = resultLoad.unwrap();
		for x in self.filter.iter() {
			x.apply(&mut data.raw, data.width, data.height);
		}
		
		let loadedInBuffer = RgbaImage::from_raw(data.width, data.height, data.raw);
		if (loadedInBuffer.is_none())
		{
			HTrace!("texture is blocked for : unable to load from_raw");
			texture.state = TextureState::BLOCKED;
			return;
		}
		let loadedInBuffer = loadedInBuffer.unwrap();
		
		texture.width = Some(loadedInBuffer.width());
		texture.height = Some(loadedInBuffer.height());
		texture.content = Some(loadedInBuffer.clone());
		texture.state = TextureState::LOADED;
		texture.mipmap = self.mipmap;
		texture.format = self.format;
		texture.reloadLoader = None;
		
		texture.contentClearable = data.clearable;
		if (self.from.canReload())
		{
			texture.reloadLoader = Some(Order_reload {
				from: self.from.clone(),
				filter: self.filter.to_vec(),
			});
		}
	}
	
	fn isSameThread(&self) -> bool {
		self.sameThread
	}
	
	fn isWaiting(&mut self) -> bool
	{
		return self.from.isWaiting();
	}
}
