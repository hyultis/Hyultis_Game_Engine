use std::fmt::{Debug, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::sync::Arc;
use cgmath::{Point3, Vector3};
use crate::components::{HGEC_base, HGEC_origin};
use crate::HGEMain::HGEMain;
use crate::Textures::Manager::ManagerTexture;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelType
{
	PIXEL,
	PERCENT
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParentType
{
	ADD,
	SUB
}

#[derive(Clone, Debug)]
struct Parent
{
	pub addsub: ParentType,
	pub position: interfacePosition
}

/// this is a specialized "world" position for interface, using Z as layer, and overlay to add gestion of percent/pixel value
#[derive(Clone)]
pub struct interfacePosition
{
	_x: f32,
	_y: f32,
	_z: u16,
	_type: PixelType,
	_parent: Vec<Parent>,
	_dynx: Option<Arc<dyn Fn() -> f32 + Send + Sync>>,
	_dyny: Option<Arc<dyn Fn() -> f32 + Send + Sync>>
}

impl interfacePosition
{
	/// define a percent positioned position, witch move if window is resized, go to 0.0 ( = 0%) , to 1.0 ( = 100.00%)
	/// with Z as Ui layer
	pub fn new_percent(x: f32, y: f32) -> Self
	{
		return interfacePosition::new_percent_z(x, y, 0);
	}
	
	/// define a percent positioned position, witch move if window is resized, go to 0.0 ( = 0%) , to 1.0 ( = 100.00%)
	pub fn new_percent_z(x: f32, y: f32, z: u16) -> Self
	{
		return interfacePosition
		{
			_x: x,
			_y: y,
			_z: z,
			_type: PixelType::PERCENT,
			_parent: Vec::new(),
			_dynx: None,
			_dyny: None
		};
	}
	
	/// define a pixel positioned position, witch do not move if window is resized (can go out of screen)
	pub fn new_pixel(x: i32, y: i32) -> Self
	{
		return interfacePosition::new_pixel_z(x, y, 0);
	}
	
	/// define a pixel positioned position, witch do not move if window is resized (can go out of screen)
	/// with Z as Ui layer
	pub fn new_pixel_z(x: i32, y: i32, z: u16) -> Self
	{
		return interfacePosition
		{
			_x: x as f32,
			_y: y as f32,
			_z: z,
			_type: PixelType::PIXEL,
			_parent: Vec::new(),
			_dynx: None,
			_dyny: None,
		};
	}
	
	/// do not keep parent !
	pub fn fromSame(x: &interfacePosition, y: &interfacePosition) -> Self
	{
		let y = x.normalizeTo(y);
		let mut tmp = interfacePosition::internal_keepx(x.clone())._parent;
		tmp.append(&mut interfacePosition::internal_keepy(y.clone())._parent);
		
		interfacePosition
		{
			_x: x._x,
			_y: y._y,
			_z: x._z,
			_type: x._type,
			_parent: tmp,
			_dynx: x._dynx.clone(),
			_dyny: y._dyny.clone(),
		}
	}
	
	fn internal_keepx(x: interfacePosition) -> interfacePosition
	{
		interfacePosition
		{
			_x: x._x,
			_y: 0.0,
			_z: x._z,
			_type: x._type,
			_parent: x._parent.iter().map(|x|{
				Parent{
					addsub: x.addsub,
					position: interfacePosition::internal_keepx(x.position.clone())
				}
			}).collect(),
			_dynx: x._dynx.clone(),
			_dyny: None,
		}
	}
	
	fn internal_keepy(x: interfacePosition) -> interfacePosition
	{
		interfacePosition
		{
			_x: 0.0,
			_y: x._y,
			_z: x._z,
			_type: x._type,
			_parent: x._parent.iter().map(|x|{
				Parent{
					addsub: x.addsub,
					position: interfacePosition::internal_keepy(x.position.clone())
				}
			}).collect(),
			_dynx: None,
			_dyny: x._dyny.clone(),
		}
	}
	
	/// shortcut for creating a dynamic interface position for X axis
	pub fn fromDynX(ttype: PixelType, func: impl Fn() -> f32 + Send + Sync + 'static) -> Self
	{
		interfacePosition
		{
			_type: ttype,
			_dynx: Some(Arc::new(func)),
			..interfacePosition::default()
		}
	}
	
	/// shortcut for creating a dynamic interface position for Y axis
	pub fn fromDynY(ttype: PixelType, func: impl Fn() -> f32 + Send + Sync + 'static) -> Self
	{
		interfacePosition
		{
			_type: ttype,
			_dyny: Some(Arc::new(func)),
			..interfacePosition::default()
		}
	}
	
	/// X static value (default), remove dynamic X
	pub fn setX(&mut self, x: f32)
	{
		self._x = x;
		self._dynx = None;
	}
	
	/// Y static value (default), remove dynamic Y
	pub fn setY(&mut self, y: f32)
	{
		self._y = y;
		self._dyny = None;
	}
	
	pub fn setZ(&mut self, z: u16)
	{
		self._z = z;
	}
	
	/// X value is that is resolved later, replace static X
	pub fn setDynX(&mut self, dynX: impl Fn() -> f32 + Send + Sync + 'static)
	{
		self._x = 0.0;
		self._dynx = Some(Arc::new(dynX));
	}
	
	/// Y value is that is resolved later, replace static Y
	pub fn setDynY(&mut self, dynY: impl Fn() -> f32 + Send + Sync + 'static)
	{
		self._y = 0.0;
		self._dyny = Some(Arc::new(dynY));
	}
	
	pub fn getXY(&self) -> [f32; 2]
	{
		[self.getX(), self._y + self.getY()]
	}
	
	pub fn getX(&self) -> f32
	{
		let tmp = self.solveWithParent();
		tmp._x + tmp.internal_getDynX()
	}
	
	pub fn getXraw(&self) -> f32
	{
		self._x + self.internal_getDynX()
	}
	
	pub fn getY(&self) -> f32
	{
		let tmp = self.solveWithParent();
		tmp._y + tmp.internal_getDynY()
	}
	
	pub fn getYraw(&self) -> f32
	{
		self._y + self.internal_getDynY()
	}
	
	pub fn getZ(&self) -> u16
	{
		self._z
	}
	
	pub fn getType(&self) -> PixelType
	{
		return self._type.clone();
	}
	
	pub fn getTypeInt(&self) -> u32
	{
		return match self._type {
			PixelType::PIXEL => { 1 }
			PixelType::PERCENT => { 0 }
		};
	}
	
	pub fn convertToVertex(&self) -> [f32; 3]
	{
		let tmp = self.solveWithParent();
		match tmp._type
		{
			PixelType::PERCENT => {
				return [
					tmp.convertPercentToVertex(tmp.getXraw()),
					tmp.convertPercentToVertex(tmp.getYraw()),
					(1000 - tmp._z) as f32 / 10000.0
				];
			},
			PixelType::PIXEL => {
				return [
					tmp.getXraw(),
					tmp.getYraw(),
					(1000 - tmp._z) as f32 / 10000.0
				];
			}
		}
	}
	
	#[deprecated]
	/// add XY, resolve and reset dynamic content if setted
	pub fn addXY(&self, width: f32, height: f32) -> Self
	{
		let mut newself = self.clone();
		newself._x = newself.getXraw() + width;
		newself._dynx = None;
		newself._y = newself.getYraw() + height;
		newself._dyny = None;
		return newself;
	}
	
	// using newWidth and a texture, apply width and height to the actual position
	// if the position is pixel, is a simple +newWidth and +height(ratio newWidth)
	// if the position is percent, rationalise the texture width with newWidth and use actual screen dimension to convert into window real ratio
	// percent need recalcul if screen dimension ratio have changed
	pub fn addTextureDimByWidth(&self, texturename: &str, newwidth: f32) -> Self
	{
		let mut newself = self.clone();
		let texture = ManagerTexture::singleton().get(texturename);
		if (texture.is_none())
		{
			return newself;
		}
		let texture = texture.unwrap();
		let (width, height) = texture.getDim();
		if (width == 0 || height == 0)
		{
			return newself;
		}
		
		match newself._type {
			PixelType::PIXEL => {
				let ratio = newwidth.abs() / width as f32;
				newself._x += newwidth;
				newself._y += (height as f32 * ratio);
			}
			PixelType::PERCENT => {
				let ratio = height as f32 / width as f32;
				let windowDim = HGEMain::singleton().getWindowInfos();
				let percentedHeight = (newwidth.abs() * ratio) * windowDim.ratio_w2h;
				//println!("addTextureDimByWidth : {} * {} * {}", newwidth.abs(), ratio, windowDim[2]);
				//println!("addTextureDimByWidth : {}/{} => {} {}", width, height, newwidth, percentedHeight);
				
				newself._x += newwidth;
				newself._y += percentedHeight;
			}
		}
		
		return newself;
	}
	
	
	// same as addTextureDimByWidth, but with height
	pub fn addTextureDimByHeight(self, texturename: &str, newheight: f32) -> Self
	{
		let mut newself = self.clone();
		let texture = ManagerTexture::singleton().get(texturename);
		if (texture.is_none())
		{
			return newself;
		}
		let texture = texture.unwrap();
		let (width, height) = texture.getDim();
		
		match newself._type {
			PixelType::PIXEL => {
				let ratio = newheight.abs() as f32 / height as f32;
				newself._y += newheight;
				newself._x += height as f32 * ratio;
			}
			PixelType::PERCENT => {
				let ratio = width as f32 / height as f32;
				let windowDim = HGEMain::singleton().getWindowInfos();
				let percentedWidth = (newheight.abs() * ratio) * windowDim.ratio_w2h;
				//println!("addTextureDimByHeight : {}/{} => {} {}", width, height, newheight, percentedWidth);
				
				newself._y += newheight.abs();
				newself._x += percentedWidth;
			}
		}
		
		return newself;
	}
	
	pub fn normalize(&self) -> interfacePosition
	{
		return self.normalizeTo(&self);
	}
	
	pub fn normalizeTo(&self, other: &interfacePosition) -> interfacePosition
	{
		if (self._type == other._type)
		{
			return other.clone();
		}
		
		let window = HGEMain::singleton().getWindowInfos();
		let mut toreturn = match self._type {
			PixelType::PIXEL => {
				let mut tt = [other.getXraw(), other.getYraw()]; // is in percent
				tt[0] = tt[0] * window.widthF;
				tt[1] = tt[1] * window.heightF;
				interfacePosition::new_pixel_z(tt[0] as i32, tt[1] as i32, other.getZ())
			}
			PixelType::PERCENT => {
				let mut tt = [other.getXraw(), other.getYraw()]; // is in percent
				tt[0] = tt[0] / window.widthF;
				tt[1] = tt[1] / window.heightF;
				interfacePosition::new_percent_z(tt[0], tt[1], other.getZ())
			}
		};
		toreturn._parent = other._parent.clone();
		return toreturn;
	}
	
	
	/////////// PRIVATE /////////////////
	
	fn convertPercentToVertex(&self, v: f32) -> f32
	{
		return (v * 2.0) - 1.0;
	}
	
	fn solveWithParent(&self) -> interfacePosition
	{
		let mut parents = self.clone();
		for x in self._parent.iter().cloned()
		{
			let normalizedParent = parents.normalizeTo(&x.position).solveWithParent();
			match x.addsub {
				ParentType::ADD => {
					parents._x = parents._x + normalizedParent.getXraw();
					parents._y = parents._y + normalizedParent.getYraw();
					parents._z = parents._z.max(normalizedParent._z);
				}
				ParentType::SUB =>
					{
						parents._x = parents._x - normalizedParent.getXraw();
						parents._y = parents._y - normalizedParent.getYraw();
						parents._z = parents._z.max(normalizedParent._z);
					}
			}
		}
		
		return parents;
	}
	
	fn internal_getDynX(&self) -> f32
	{
		let val = match &self._dynx {
			None => 0.0,
			Some(func) => func()
		};
		return val;
	}
	
	fn internal_getDynY(&self) -> f32
	{
		let val = match &self._dyny {
			None => 0.0,
			Some(func) => func()
		};
		return val;
	}
}

impl Debug for interfacePosition
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("interfacePosition")
			.field("_x", &self._x)
			.field("_y", &self._y)
			.field("_type", &self._type)
			.field("_parent", &self._parent)
			.field("_dynx", &self.internal_getDynX())
			.field("_dyny", &self.internal_getDynY())
			.finish()
	}
}

impl HGEC_base<interfacePosition> for interfacePosition
{
	fn compute(&self, vertex: &mut interfacePosition) {
		*vertex += self.clone();
	}
}

impl HGEC_origin for interfacePosition
{
	fn get(&self) -> [f32; 3] {
		[self._x, self._y, self._z as f32]
	}
	
	fn set(&mut self, new: [f32; 3]) {
		self._x = new[0];
		self._y = new[1];
	}
	
	fn toPoint3(&self) -> Point3<f32>
	{
		return Point3::new(self._x, self._y, self._z as f32);
	}
	
	fn toVec3(&self) -> Vector3<f32>
	{
		return Vector3::new(self._x, self._y, self._z as f32);
	}
}

impl Default for interfacePosition
{
	fn default() -> Self {
		interfacePosition
		{
			_x: 0.0,
			_y: 0.0,
			_z: 0,
			_type: PixelType::PERCENT,
			_parent: Vec::new(),
			_dynx: None,
			_dyny: None,
		}
	}
}

impl Add for interfacePosition {
	type Output = interfacePosition;
	
	fn add(self, rhs: Self) -> Self::Output {
		let mut newself = self;
		let newparent = Parent {
			addsub: ParentType::ADD,
			position: rhs,
		};
		newself._parent.push(newparent);
		return newself;
	}
}

impl AddAssign for interfacePosition {
	fn add_assign(&mut self, rhs: Self) {
		let newparent = Parent {
			addsub: ParentType::ADD,
			position: rhs,
		};
		self._parent.push(newparent);
	}
}

impl Sub for interfacePosition {
	type Output = interfacePosition;
	
	fn sub(self, rhs: Self) -> Self::Output {
		let mut newself = self;
		let newparent = Parent {
			addsub: ParentType::SUB,
			position: rhs,
		};
		newself._parent.push(newparent);
		return newself;
	}
}

impl SubAssign for interfacePosition {
	fn sub_assign(&mut self, rhs: Self) {
		let newparent = Parent {
			addsub: ParentType::SUB,
			position: rhs,
		};
		self._parent.push(newparent);
	}
}
