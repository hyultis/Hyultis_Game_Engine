use crate::HGEMain::HGEMain;
use crate::Shaders::Shs_2DVertex::HGE_shader_2Dsimple;

#[derive(Clone, Debug)]
struct UiHitbox_content
{
	pub left: f32,
	pub right: f32,
	pub top: f32,
	pub bottom: f32,
}

#[derive(Clone, Debug)]
pub struct UiHitbox
{
	_hitbox: Option<UiHitbox_content>,
}

impl UiHitbox
{
	pub fn new() -> Self
	{
		return UiHitbox{
			_hitbox: None,
		};
	}
	
	pub fn getRaw(&self) -> [[f32; 2]; 2]
	{
		let mut content = UiHitbox_content{
			left: 0.0,
			right: 0.0,
			top: 0.0,
			bottom: 0.0,
		};
		
		if let Some(thiscontent) = &self._hitbox
		{
			content = thiscontent.clone();
		}
		
		return [
			[content.left,content.top],
			[content.right,content.bottom]
		];
	}
	
	pub fn getRawWithWH(&self) -> [f32; 4]
	{
		let hitbox = self.getRaw();
		return [hitbox[0][0],hitbox[0][1],hitbox[1][0]-hitbox[0][0],hitbox[1][1]-hitbox[0][1]];
	}
	
	pub fn isInside(&self, x: u16, y: u16) -> bool
	{
		let x = x as f32;
		let y = y as f32;
		
		
		let Some(hitbox) = &self._hitbox else {
			return false;
		};
		
		
		if (x > hitbox.left && x < hitbox.right && y > hitbox.top && y < hitbox.bottom)
		{
			return true;
		}
		
		return false;
	}
	
	pub fn isEmpty(&self) -> bool
	{
		match &self._hitbox {
			None =>return true,
			Some(content) => {
				if(content.bottom-content.top<1.0)
				{
					return true;
				}
				if(content.right-content.left<1.0)
				{
					return true;
				}
				return false;
			}
		};
	}
	
	// update comparing min and max point
	pub fn updateFromHitbox(&mut self, other: UiHitbox)
	{
		if(other.isEmpty())
		{
			return;
		}
		
		let Some(other) = &other._hitbox else
		{
			return;
		};
		
		match &mut self._hitbox {
			None => {
				self._hitbox = Some(other.clone());
			}
			Some(this) => {
				this.left = other.left.min(this.left);
				this.right = other.right.max(this.right);
				this.top = other.top.min(this.top);
				this.bottom = other.bottom.max(this.bottom);
			}
		}
	}
	
	pub fn newFrom2D(cache: &Vec<HGE_shader_2Dsimple>) -> Self
	{
		let mut hitbox = UiHitbox::new();
		let winDim = HGEMain::singleton().getWindowInfos();
		for x in cache {
			let mut posx = x.position[0];
			let mut posy = x.position[1];
			if(x.ispixel==0)
			{
				posx = (((x.position[0]+1.0)/2.0) * winDim.widthF).round();
				posy = (((x.position[1]+1.0)/2.0) * winDim.heightF).round();
			}
			
			hitbox.updateFromPoint(posx, posy);
		};
		
		return hitbox;
	}
	
	pub fn updateFromPoint(&mut self, posx: f32, posy: f32)
	{
		let Some(hitbox) = &mut self._hitbox else
		{
			self._hitbox = Some(UiHitbox_content{
				left: posx,
				right: posx,
				top: posy,
				bottom: posy,
			});
			return;
		};
		
		hitbox.left = posx.min(hitbox.left);
		hitbox.right = posx.max(hitbox.right);
		hitbox.top = posy.min(hitbox.top);
		hitbox.bottom = posy.max(hitbox.bottom);
	}
}
