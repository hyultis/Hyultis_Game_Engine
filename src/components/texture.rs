use std::fmt::Debug;
use crate::components::{HGEC_base, HGEC_texture};
use crate::components::color::{color, colorBlend};
use crate::components::uvcoord::uvcoord;
use crate::Textures::Manager::ManagerTexture;
use crate::Textures::Textures::TextureState;

#[derive(Clone, Debug, Default)]
pub struct texture
{
	pub id: u32,
	pub uvcoord: uvcoord,
	pub color: color,
	pub colorBlend: colorBlend
}

#[derive(Clone, Debug)]
pub struct textureAsset
{
	isok: bool,
	name: Option<String>,
	part: Option<String>,
	id: Option<u32>,
	uvcoord: Option<uvcoord>,
	color: color,
	colorBlend: colorBlend
}

impl textureAsset
{
	pub fn getName(&self) -> &Option<String> {
		&self.name
	}
	pub fn getPart(&self) -> Option<String> {
		self.part.clone()
	}
	pub fn getId(&self) -> u32 {
		self.id.unwrap_or(0)
	}
	
	pub fn getUvcoord(&self) -> uvcoord {
		self.uvcoord.unwrap_or_default()
	}
	
	pub fn setUvcoord(&mut self, val: uvcoord) {
		self.uvcoord = Some(val);
	}
	
	pub fn setUvcoord_none(&mut self) {
		self.uvcoord = None;
	}
	
	pub fn set(&mut self, name: impl Into<String>) {
		let tmpname = name.into();
		self.part = None;
		self.isok = false;
			
		if(tmpname.contains("#"))
		{
			let tmp: Vec<&str> = tmpname.split("#").collect();
			let tmp = tmp.clone();
			self.part = Some(tmp[1].to_string());
			self.name = Some(tmp[0].to_string());
		}
		else
		{
			self.name = Some(tmpname);
		}
	}
	
	pub fn unset(&mut self)
	{
		self.name = None;
		self.part = None;
		self.id = None;
	}
	
	pub fn color(&self) -> &color
	{
		&self.color
	}
	
	pub fn color_mut(&mut self) -> &mut color
	{
		&mut self.color
	}
	
	pub fn colorBlend(&self) -> &colorBlend
	{
		&self.colorBlend
	}
	
	pub fn colorBlend_mut(&mut self) -> &mut colorBlend
	{
		&mut self.colorBlend
	}
}

impl Default for textureAsset
{
	fn default() -> Self {
		textureAsset{
			isok: true,
			name: None,
			part: None,
			id: Some(0),
			uvcoord: Some(uvcoord::default()),
			color: color::default(),
			colorBlend: colorBlend::MUL,
		}
	}
}

impl HGEC_base<Option<texture>> for textureAsset
{
	fn compute(&self, texture: &mut Option<texture>)
	{
		if(self.isok)
		{
			*texture = Some(texture {
				id: self.id.unwrap_or_default(),
				uvcoord: self.uvcoord.unwrap_or_default(),
				color: self.color,
				colorBlend: self.colorBlend,
			});
		}
		else
		{
			*texture = None;
		}
	}
}

impl HGEC_texture for textureAsset
{
	fn check(&mut self)
	{
		if(self.isok)
		{
			return;
		}
		
		let Some(texturename) = &self.name else {return};
		
		let textureok = match ManagerTexture::singleton().get(texturename) {
			None => false,
			Some(texture) => {
				texture.state != TextureState::CREATED
			}
		};
		
		if(!textureok)
		{
			return;
		}
		
		self.id = ManagerTexture::singleton().getTextureToId(texturename);
		//println!("resolved vertex texture : {} -> {:?}",&self.name,&self.id);
		if(self.id.is_none())
		{
			return;
		}
		
		if let Some(namepart) = &self.part
		{
			let uvcoordok = match ManagerTexture::singleton().getPart(texturename, namepart) {
				None => false,
				Some(part) => {
					self.uvcoord = Some(uvcoord::from(part));
					true
				}
			};
			if(uvcoordok)
			{
				self.isok = true;
			}
		}
		else
		{
			self.isok = true;
		}
		
	}
}

#[derive(Clone, Default, Debug)]
pub struct texture_none
{
}

impl HGEC_base<Option<texture>> for texture_none {
	fn compute(&self, _: &mut Option<texture>) {
	
	}
}

impl HGEC_texture for texture_none
{
	fn check(&mut self)
	{
	
	}
}
