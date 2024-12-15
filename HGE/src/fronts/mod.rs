#[cfg(feature = "front_sdl")]
pub mod sdl;
#[cfg(feature = "front_winit")]
pub mod winit;

pub mod export
{
	#[cfg(feature = "front_sdl")]
	pub extern crate sdl2;
	#[cfg(feature = "front_winit")]
	pub extern crate winit;
}

pub mod EngineEvent;
