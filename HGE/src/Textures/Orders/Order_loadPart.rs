use crate::Textures::Order::Order;
use crate::Textures::texturePart::texturePart;
use crate::Textures::Textures::Texture;

#[derive(Clone)]
pub struct Order_loadPart
{
	pub from: Box<dyn texturePart + Send + Sync>
}

impl Order for Order_loadPart
{
	fn exec(&self, texture: &mut Texture) {
		if let Ok(result) = self.from.load(texture)
		{
			texture.partUVCoord = result;
		}
	}
	
	fn isSameThread(&self) -> bool {
		false
	}
}
