use image::{ImageBuffer, Rgba, RgbaImage};

pub fn defaultTexture() -> ImageBuffer<Rgba<u8>, Vec<u8>>
{
	let mut img = RgbaImage::new(2, 2);
	for (x, y, pixel) in img.enumerate_pixels_mut()
	{
		let Rgba(mut data) = *pixel;
		if((x==0 && y==0) || (x==1 && y==1))
		{
			data[0] = 0;
			data[1] = 0;
			data[2] = 0;
			data[3] = 255;
		}
		if(x==0 && y==1)
		{
			data[0] = 204;
			data[1] = 0;
			data[2] = 204;
			data[3] = 255;
		}
		if(x==1 && y==0)
		{
			data[0] = 204;
			data[1] = 204;
			data[2] = 0;
			data[3] = 255;
		}
		
		*pixel = Rgba(data);
	}
	
	return img;
}


pub fn emptyTexture(width: u32, height: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>>
{
	return RgbaImage::new(width, height);
}
