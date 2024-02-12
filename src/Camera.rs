use std::sync::Arc;
use cgmath::{Deg, InnerSpace, Matrix4, Point3, Rad, Vector3};
use crate::components::window::window_orientation;
use crate::HGEMain::HGEMain;

const PI: f64 = 3.1415926535897932384626433832795;
const RADIAN: f64 = 0.0174532925199432957692369076848;

#[derive(Clone)]
pub struct Camera
{
	_posx : f32,
	_posy : f32,
	_posz : f32,
	_offset : [f32;3],
	_pitch : Rad<f32>,
	_yaw : Rad<f32>,
	_fovY: Deg<f32>,
	_projFunc: Arc<dyn Fn(&Camera) -> Matrix4<f32> + Send + Sync>
}


impl Camera
{
	pub fn new() -> Camera
	{
		Camera
		{
			_posx : 0.0,
			_posy : 0.0,
			_posz : 0.0,
			_offset : [0.0,0.0,0.0],
			_pitch : Rad(0.0),
			_yaw : Rad(0.0),
			_fovY: Deg(90.0),
			_projFunc: Arc::new(|this|{
				let windowdim = HGEMain::singleton().getWindowInfos();
				let aspect_ratio = match windowdim.orientation {
					window_orientation::NORMAL | window_orientation::ROT_180 => windowdim.ratio_w2h,
					window_orientation::ROT_90 | window_orientation::ROT_270 => windowdim.ratio_h2w
				}; // move ratio into var;
				let near= 0.1;
				let far = 10000.0;
				
				return cgmath::perspective(
					Rad::from(this._fovY),//Rad(std::f32::consts::FRAC_PI_2),
					aspect_ratio,
					near,
					far,
				);
			}),
		}
	}

	pub fn setPositionX(&mut self, x: f32)
	{
		self._posx = x;
	}

	pub fn setPositionY(&mut self, y: f32)
	{
		self._posy = y;
	}

	pub fn setPositionZ(&mut self, z: f32)
	{
		self._posz = z;
	}

	
	pub fn setPositionXYZ(&mut self, x: f32, y: f32, z: f32)
	{
		self._posx = x;
		self._posy = y;
		self._posz = z;
	}
	
	pub fn getPositionXYZ(&self) -> [f32; 3]
	{
		return [self._posx,self._posy,self._posz];
	}
	
	pub fn setPitch(&mut self, pitch: Deg<f32>)
	{
		self._pitch = Rad::from(pitch);
		self.limitPitch();
	}
	
	pub fn getPitch(&self) -> Deg<f32>
	{
		Deg::from(self._pitch)
	}
	
	pub fn setYaw(&mut self, yaw: Deg<f32>)
	{
		self._yaw = Rad::from(yaw);
		self.limitYaw();
	}
	
	pub fn getYaw(&self) -> Deg<f32>
	{
		Deg::from(self._yaw)
	}
	
	pub fn setOffset(&mut self, x: f32, y: f32, z:f32)
	{
		self._offset = [x,y,z];
	}
	
	pub fn getOffset(&self) -> [f32; 3]
	{
		return self._offset;
	}
	
	/// update mouvement from a relative mouvement (-1.0, 0.0, +1.0 on forward / side)
	/// sensibility change the intensity of translate from 0 to u8 max (default 100)
	/// fly allow or not movement on y axis
	pub fn updatePositionFromMouvement(&mut self, forward: f32, side: f32, sensibility: u8, fly: bool)
	{
		let sensibility = sensibility as f32 * 0.01;
		
		let (yaw_sin, yaw_cos) = self._yaw.0.sin_cos();
		let (pitch_sin, pitch_cos) = self._pitch.0.sin_cos();
		//println!("camera : {} {}",self._pitch.0,pitch_sin);
		let forwardC = Vector3::new(yaw_cos * if fly {pitch_cos}else{1.0}, if fly {pitch_sin}else{0.0}, yaw_sin*if fly {pitch_cos}else{1.0}).normalize();
		let mut tmp = forwardC * forward * sensibility;
		let sideC = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
		tmp += sideC * side * sensibility;
		
		self._posx += tmp.x;
		self._posy += tmp.y;
		self._posz += tmp.z;
	}
	
	/// update Pitch and Yaw of camera from relative mousex and mousey (-1.0, 0.0, +1.0)
	/// sensibility change the intensity of translate from 0 to u8 max (default 100)
	pub fn updatePitchYawFromMouse(&mut self, mousex: f64, mousey: f64, sensibility: u8)
	{
		let sensibility = sensibility as f32 * 0.05;
		self._pitch += Rad(-mousey as f32*RADIAN as f32) * sensibility;
		self.limitPitch();
		self._yaw += Rad(-mousex as f32*RADIAN as f32) * sensibility;
		self.limitYaw();
	}
	
	pub fn getPositionMatrix(&self, ajustedYaw: f32) -> Matrix4<f32>
	{
		//println!(" pitch : {} / yaw : {}",self._pitch.0, self._yaw.0);
		let (pitch_sin, pitch_cos) = self._pitch.0.sin_cos();
		let (yaw_sin, yaw_cos) = (self._yaw+Rad::from(Deg(ajustedYaw))).0.sin_cos();
		
		Matrix4::look_to_rh(
			Point3::new(self._posx + self._offset[0], self._posy + self._offset[1], self._posz + self._offset[2]),
			Vector3::new(
				pitch_cos * yaw_cos,
				pitch_sin,
				pitch_cos * yaw_sin
			).normalize(),
			-Vector3::unit_y(),
		)
	}
	
	pub fn setFovY(&mut self, fovy: impl Into<Deg<f32>>)
	{
		self._fovY = fovy.into();
	}
	
	pub fn getFovY(&self) -> Deg<f32>
	{
		self._fovY
	}
	
	pub fn setProjectionMatrix(&mut self, newProjFunc: impl Fn(&Camera) -> Matrix4<f32> + Send + Sync + 'static)
	{
		self._projFunc = Arc::new(newProjFunc);
	}
	
	pub fn getProjectionMatrix(&self) -> Matrix4<f32>
	{
		let tmp = &self._projFunc;
		return tmp(self);
	}
	
	///// PRIVATE //////
	
	fn limitPitch(&mut self)
	{
		if(self._pitch.0< -1.5707)
		{
			self._pitch = Rad(-1.5707);
		}
		else if(self._pitch.0 > 1.5707)
		{
			self._pitch = Rad(1.5707);
		}
	}
	
	fn limitYaw(&mut self)
	{
		
		if(self._yaw.0< -3.14*2.0)
		{
			self._yaw = Rad(self._yaw.0 + (3.14*2.0));
		}
		else if(self._yaw.0> 3.14*2.0)
		{
			self._yaw = Rad(self._yaw.0 - (3.14*2.0));
		}
	}
}
