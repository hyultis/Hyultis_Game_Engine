use Htrace::HTraceError;
use image::{Rgba, RgbaImage};
use crate::Textures::Order::Order;
use crate::Textures::Textures::Texture;
use image::GenericImage;

#[derive(Clone)]
pub struct Order_partialTextureUpdate
{
	pub raw: RgbaImage,
	pub offset: [u32; 2],
	pub sameThread: bool,
}

impl Order for Order_partialTextureUpdate
{
	fn exec(&self, texture: &mut Texture)
	{
		if let Some(originTextureBuffer) = &mut texture.content
		{
			let mut usableWidth = texture.width.unwrap();
			let mut usableHeight = texture.height.unwrap();
			let mut needResize = false;
			
			// sizecheck
			if (self.offset[0] + self.raw.width() > usableWidth)
			{
				usableWidth = self.offset[0] + self.raw.width();
				needResize = true;
			}
			if (self.offset[1] + self.raw.height() > usableHeight)
			{
				usableHeight = self.offset[1] + self.raw.height();
				needResize = true;
			}
			if (needResize)
			{
				let mut tmp = RgbaImage::from_pixel(usableWidth, usableHeight, Rgba([0, 0, 0, 0]));
				HTraceError!(tmp.copy_from(originTextureBuffer,0,0));
				*originTextureBuffer = tmp;
				texture.width = Some(usableWidth);
				texture.height = Some(usableHeight);
			}
			
			HTraceError!("cannot copy : {}",originTextureBuffer.copy_from(&self.raw, self.offset[0], self.offset[1]));
		}
	}
	
	fn isSameThread(&self) -> bool {
		self.sameThread
	}
}
