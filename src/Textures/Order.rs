use dyn_clone::DynClone;
use crate::Textures::Textures::Texture;

pub trait Order: DynClone
{
	fn exec(&self, texture: &mut Texture);
	fn isSameThread(&self) -> bool;
	fn isWaiting(&mut self) -> bool
	{
		return false;
	}
}

dyn_clone::clone_trait_object!(Order);
