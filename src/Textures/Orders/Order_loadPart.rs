use crate::Textures::Order::Order;
use crate::Textures::texturePart::texturePart;
use crate::Textures::Textures::Texture;

pub struct Order_loadPart
{
	pub from: Box<dyn texturePart + Send + Sync>
}

impl Order for Order_loadPart
{
	fn exec(&self, _: u32, texture: &mut Texture) {
		if let Ok(result) = self.from.load(texture)
		{
			texture.partUVCoord = result;
		}
	}
	
	fn isSameThread(&self) -> bool {
		false
	}
}
