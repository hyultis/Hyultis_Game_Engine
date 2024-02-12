use crate::Textures::Textures::Texture;

pub trait Order
{
	fn exec(&self, id: u32, texture: &mut Texture);
	fn isSameThread(&self) -> bool;
	fn isWaiting(&mut self) -> bool
	{
		return false;
	}
}
