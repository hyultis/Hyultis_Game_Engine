use dyn_clone::DynClone;
use image::{ImageBuffer, Rgba};
use image::imageops::{flip_horizontal_in_place, flip_vertical_in_place};
use crate::components::color::color;

pub trait Filter: DynClone
{
	fn apply(&self, data: &mut Vec<u8>, width: u32, height: u32);
}

dyn_clone::clone_trait_object!(Filter);

// add a border to opaque part en a image
#[derive(Clone)]
pub struct Filter_addBorder
{
	pub color: [u8;3],
	pub size: u16
}

impl Filter for Filter_addBorder
{
	fn apply(&self, data: &mut Vec<u8>, width: u32, height: u32)
	{
		let mut imgbuf= ImageBuffer::from_raw(width,height,data.clone()).unwrap();
		let oldimg = imgbuf.clone();
		let borderSize = self.size as i32;
		
		for (posx, posy, pixel) in imgbuf.enumerate_pixels_mut()
		{
			let image::Rgba(data) = *pixel;
			if(data[3]==255)
			{
				continue;
			}
			
			let posx = posx as i32;
			let posy = posy as i32;
			let mut itsBorder = 0;
			for x in -borderSize..=borderSize
			{
				for y in -borderSize..=borderSize
				{
					if(posx+x<0 || posx+x>=width as i32 || posy+y < 0 || posy+y>=height as i32)
					{
						continue;
					}
					
					if(x.abs()+y.abs() > borderSize)
					{
						continue;
					}
					
					let image::Rgba(data) = oldimg.get_pixel((posx+x) as u32,(posy+y) as u32);
					if(data[3]==255)
					{
						if(itsBorder==0 || itsBorder>x.abs()+y.abs())
						{
							itsBorder = x.abs()+y.abs();
						}
					}
				}
			}
			
			if(itsBorder>0)
			{
				let image::Rgba(mut data) = *pixel;
				if(itsBorder==borderSize)
				{
					data[0] = self.color[0];
					data[1] = self.color[1];
					data[2] = self.color[2];
					data[3] = 126;
				}
				else
				{
					if(itsBorder==1) // 50% of each color
					{
						//println!("sdfkhsdfl");
						data[0] = (data[0]/2)+(self.color[0]/2);
						data[1] = (data[1]/2)+(self.color[1]/2);
						data[2] = (data[2]/2)+(self.color[2]/2);
					}
					else
					{
						data[0] = self.color[0];
						data[1] = self.color[1];
						data[2] = self.color[2];
					}
					data[3] = 255;
				}
				*pixel = image::Rgba(data);
			}
		}
		
		*data = imgbuf.into_raw();
	}
}

// replace all color to this color (alpha untouched)
#[derive(Clone)]
pub struct Filter_reColor
{
	pub color: [u8;3]
}

impl Filter for Filter_reColor
{
	fn apply(&self, data: &mut Vec<u8>, width: u32, height: u32)
	{
		let mut imgbuf= ImageBuffer::from_raw(width,height,data.clone()).unwrap();

		for (_, _, pixel) in imgbuf.enumerate_pixels_mut()
		{
			let image::Rgba(mut data) = *pixel;
				data[0] = self.color[0];
				data[1] = self.color[1];
				data[2] = self.color[2];
				*pixel = image::Rgba(data);
		}

		*data = imgbuf.into_raw();
	}
}


// allow to flip image
#[derive(Clone,Debug)]
pub enum Filter_flipendo_orientation
{
	HORIZONTAL,
	VERTICAL,
	BOTH
}

#[derive(Clone)]
pub struct Filter_flipendo
{
	pub orientation: Filter_flipendo_orientation
}

impl Filter for Filter_flipendo
{
	fn apply(&self, data: &mut Vec<u8>, width: u32, height: u32)
	{
		
		let mut imgbuf: image::ImageBuffer<Rgba<_>, Vec<u8>>= ImageBuffer::from_raw(width,height,data.clone()).unwrap();
		match self.orientation {
			Filter_flipendo_orientation::HORIZONTAL => flip_horizontal_in_place(&mut imgbuf),
			Filter_flipendo_orientation::VERTICAL => flip_vertical_in_place(&mut imgbuf),
			Filter_flipendo_orientation::BOTH => {
				flip_horizontal_in_place(&mut imgbuf);
				flip_vertical_in_place(&mut imgbuf);
			}
		}
		
		*data = imgbuf.into_raw();
	}
}

// ajust constract / brightness, ignore alpha
#[derive(Clone)]
pub struct Filter_contrast
{
	/// 1 = actual, 0.5 = half, 2 = double
	pub contrast: f32,
}

impl Filter_contrast
{
	fn formula(&self,data: u8) -> u8
	{
		let contrast = self.contrast;
		let data = data as f32 - 128.0;
		
		let returned = (contrast * data) + 128.0;
		
		return returned.clamp(0.0,255.0) as u8;
	}
}

impl Filter for Filter_contrast
{
	fn apply(&self, data: &mut Vec<u8>, width: u32, height: u32)
	{
		let mut imgbuf= ImageBuffer::from_raw(width,height,data.clone()).unwrap();
		
		for (_, _, pixel) in imgbuf.enumerate_pixels_mut()
		{
			let image::Rgba(mut data) = *pixel;
			for x in 0..3
			{
				data[x] = self.formula(data[x]);
			}
			*pixel = image::Rgba(data);
		}
		
		*data = imgbuf.into_raw();
	}
}

#[derive(Clone)]
pub struct Filter_brightness
{
	pub brightness: i16,
}

impl Filter_brightness
{
	fn formula(&self,data: u8) -> u8
	{
		let returned = data as i16 + self.brightness;
		
		return returned.clamp(0,255) as u8;
	}
}

impl Filter for Filter_brightness
{
	fn apply(&self, data: &mut Vec<u8>, width: u32, height: u32)
	{
		let mut imgbuf= ImageBuffer::from_raw(width,height,data.clone()).unwrap();
		
		for (_, _, pixel) in imgbuf.enumerate_pixels_mut()
		{
			let image::Rgba(mut data) = *pixel;
			for x in 0..3
			{
				data[x] = self.formula(data[x]);
			}
			*pixel = image::Rgba(data);
		}
		
		*data = imgbuf.into_raw();
	}
}

#[derive(Clone)]
pub struct Filter_clamps
{
	/// re-adjust min/max value depending on these
	pub clamp_top: Option<u8>,
	pub clamp_bottom: Option<u8>,
}

impl Filter_clamps
{
	fn clamps(&self, data: u8) -> u8
	{
		if(self.clamp_top.is_none() && self.clamp_bottom.is_none())
		{
			return data;
		}
		
		let top = self.clamp_top.unwrap_or(255) as f32;
		let mut bottom = self.clamp_bottom.unwrap_or(0) as f32;
		if(bottom>top)
		{
			bottom = top;
		}
		let ratio = 255.0/(top-bottom);
		
		let dataclamp = (data as f32).clamp(bottom,top);
		let returned = (dataclamp - bottom) * ratio;
		
		return returned.clamp(0.0,255.0) as u8;
	}
}

impl Filter for Filter_clamps
{
	fn apply(&self, data: &mut Vec<u8>, width: u32, height: u32)
	{
		let mut imgbuf= ImageBuffer::from_raw(width,height,data.clone()).unwrap();
		
		for (_, _, pixel) in imgbuf.enumerate_pixels_mut()
		{
			let image::Rgba(mut data) = *pixel;
			for x in 0..3
			{
				data[x] = self.clamps(data[x]);
			}
			*pixel = image::Rgba(data);
		}
		
		*data = imgbuf.into_raw();
	}
}

// add blend color
#[derive(Clone)]
pub struct Filter_blend
{
	pub blend: color,
}

impl Filter for Filter_blend
{
	fn apply(&self, data: &mut Vec<u8>, width: u32, height: u32)
	{
		let mut imgbuf= ImageBuffer::from_raw(width,height,data.clone()).unwrap();
		
		for (_, _, pixel) in imgbuf.enumerate_pixels_mut()
		{
			let image::Rgba(data) = *pixel;
			*pixel = image::Rgba(color::from(data).blend(self.blend).toArrayu8());
		}
		
		*data = imgbuf.into_raw();
	}
}
