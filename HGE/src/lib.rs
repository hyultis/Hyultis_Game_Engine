#![allow(unused_parens)]
#![allow(unused_doc_comments)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code, deprecated)]
#![allow(unused_parens)]

pub mod Animation;
mod BuilderDevice;
pub mod Camera;
mod HGEFrame;
pub mod HGEMain;
pub mod HGEMain_preinit;
mod HGESwapchain;
mod HGErendering;
pub mod HGEsubpass;
pub mod Interface;
pub mod InterpolateTimer;
pub mod ManagerAnimation;
pub mod ManagerAudio;
pub mod ManagerBuilder;
pub mod ManagerMemoryAllocator;
pub mod Models3D;
pub mod Paths;
pub mod Pipeline;
pub mod Shaders;
pub mod Textures;
pub mod assetStreamReader;
pub mod components;
pub mod configs;
pub mod entities;
pub mod fronts;

pub mod export
{
	pub extern crate vulkano;
	pub extern crate vulkano_shaders;
}
