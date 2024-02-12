use Htrace::HTrace;
use image::RgbaImage;
use crate::Textures::Filter::Filter;
use crate::Textures::Order::Order;
use crate::Textures::textureLoader::textureLoader;
use crate::Textures::Textures::{Texture, TextureState};

#[derive(Clone)]
pub struct Order_reload
{
	pub from: Box<dyn textureLoader + Send + Sync>,
	pub filter: Vec<Box<dyn Filter + Send + Sync>>,
}

impl Order_reload
{
	pub fn new(loader: impl textureLoader + Send + Sync + 'static) -> Self
	{
		return Order_reload {
			from: Box::new(loader),
			filter: vec![],
		};
	}
	
	pub fn filter_add(mut self, newfilter: impl Filter + Send + Sync + 'static) -> Self
	{
		self.filter.push(Box::new(newfilter));
		return self;
	}
	
}

impl Order for Order_reload
{
	fn exec(&self, _: u32, texture: &mut Texture)
	{
		let resultLoad = self.from.load();
		if (resultLoad.is_err())
		{
			HTrace!("texture is not reloaded because : {}", resultLoad.err().unwrap());
			return;
		}
		let mut data = resultLoad.unwrap();
		for x in self.filter.iter() {
			x.apply(&mut data.raw, data.width, data.height);
		}
		
		let loadedInBuffer = RgbaImage::from_raw(data.width,data.height,data.raw);
		if (loadedInBuffer.is_none())
		{
			HTrace!("texture is not reloaded because : unable to load from_raw");
			return;
		}
		let loadedInBuffer = loadedInBuffer.unwrap();
		
		texture.width = Some(loadedInBuffer.width());
		texture.height = Some(loadedInBuffer.height());
		texture.content = Some(loadedInBuffer.clone());
		texture.state = TextureState::LOADED;
		
	}
	
	fn isSameThread(&self) -> bool {
		false
	}
	
	fn isWaiting(&mut self) -> bool
	{
		return self.from.isWaiting();
	}
}
