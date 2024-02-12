use Htrace::{HTrace, HTraceError};
use image::{Rgba, RgbaImage};
use crate::Textures::Order::Order;
use crate::Textures::Textures::{Texture, TextureState};
use image::GenericImage;
use image::GenericImageView;

pub struct Order_resize
{
	pub newWidth: u32,
	pub newHeight: u32,
	pub sameThread: bool
}

impl Order for Order_resize
{
	fn exec(&self, _: u32, texture: &mut Texture)
	{
		let textureWidth = texture.width.unwrap_or(0);
		let textureHeight = texture.width.unwrap_or(0);
		if (textureWidth == self.newWidth && textureHeight == self.newHeight)
		{
			return;
		}
		
		if let Some(originTextureBuffer) = &mut texture.content
		{
			if (textureWidth < self.newWidth && textureHeight < self.newHeight)
			{
				// simple up resize
				let mut tmp = RgbaImage::from_pixel(self.newWidth, self.newHeight, Rgba([0, 0, 0, 0]));
				HTraceError!("[Order_resize] Cannot copy : {}",tmp.copy_from(originTextureBuffer,0,0));
				*originTextureBuffer = tmp;
			}
			else
			{
				let minWidth = textureWidth.min(self.newWidth);
				let minHeight = textureHeight.min(self.newHeight);
				
				// here we need to resize down one dimension, so we need to become tricky
				let mut tmp = RgbaImage::from_pixel(self.newWidth, self.newHeight, Rgba([0, 0, 0, 0]));
				HTraceError!(tmp.copy_from(&originTextureBuffer.view(0,0,minWidth,minHeight).to_image(),0,0));
			}
			
			texture.width = Some(self.newWidth);
			texture.height = Some(self.newHeight);
			texture.state = TextureState::LOADED;
			HTrace!("resized to {}x{}",self.newWidth,self.newHeight);
			
		}
	}
	
	fn isSameThread(&self) -> bool {
		self.sameThread
	}
}
