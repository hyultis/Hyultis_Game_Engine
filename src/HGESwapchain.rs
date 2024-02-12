use std::sync::Arc;
use anyhow::anyhow;
use Htrace::{HTrace, HTraceError, TSpawner};
use vulkano::format::Format;
use vulkano::device::DeviceOwned;
use vulkano::image::ImageUsage;
use vulkano::image::view::ImageView;
use vulkano::swapchain::{CompositeAlpha, PresentMode, Surface, SurfaceInfo, SurfaceTransform, Swapchain, SwapchainCreateInfo};
use vulkano::Validated;
use crate::BuilderDevice::BuilderDevice;
use crate::configs::HGEconfig::HGEconfig;
use crate::HGEMain::HGEMain;
use crate::Interface::ManagerInterface::ManagerInterface;

#[derive(Clone)]
pub struct HGESwapchain
{
	_swapChain: Arc<Swapchain>,
	_surfaceMinImage: u32,
	_surfaceMaxImage: u32,
	_images: Vec<Arc<ImageView>>,
	_fpslimiter: u8,
	_systemDesync: bool
}

impl HGESwapchain
{
	pub fn new(builderDevice : BuilderDevice, surface: Arc<Surface>) -> Self
	{
		let surface_capabilities = builderDevice.device
			.physical_device()
			.surface_capabilities(&surface, Default::default())
			.unwrap();
		let image_formats =
			builderDevice.device
				.physical_device()
				.surface_formats(&surface, Default::default()).unwrap();
		
		// can B8G8R8A8_UNORM ?
		let mut format = image_formats[0].0;
		image_formats.iter().for_each(|(thisformat,_)|{
			HTrace!("format image allowed : {:?}",thisformat);
			if(*thisformat==Format::R8G8B8A8_UNORM)
			{
				format = Format::R8G8B8A8_UNORM;
			}
			if(*thisformat==Format::B8G8R8A8_UNORM)
			{
				format = Format::B8G8R8A8_UNORM;
			}
		});
		HTrace!("surface image format : {:?}",image_formats);
		
		let (swapchain, images) = Swapchain::new(
			builderDevice.device.clone(),
			surface.clone(),
			SwapchainCreateInfo {
				min_image_count: surface_capabilities.min_image_count,
				image_format: format,
				image_extent: HGEMain::singleton().getWindowInfos().into(),
				image_usage: ImageUsage::TRANSFER_DST | ImageUsage::COLOR_ATTACHMENT, // ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST |
				present_mode: PresentMode::Fifo, // default
				pre_transform: SurfaceTransform::Identity,
				composite_alpha: surface_capabilities
					.supported_composite_alpha
					.into_iter()
					.next()
					.unwrap_or(CompositeAlpha::Inherit),
				..Default::default()
			},
		)
			.unwrap();
		
		
		let images = images
			.into_iter()
			.map(|image| ImageView::new_default(image).unwrap())
			.collect::<Vec<_>>();
			
		return HGESwapchain
		{
			_swapChain: swapchain,
			_surfaceMinImage: surface_capabilities.min_image_count,
			_surfaceMaxImage: surface_capabilities.max_image_count.unwrap_or(surface_capabilities.min_image_count),
			_images: images,
			_fpslimiter: 0,
			_systemDesync: true
		};
	}
	
	pub fn recreate(&mut self)
	{
		let result = self.internal_recreate(true);
		if result.is_err()
		{
			println!("retry internal_recreate : {:?}",result);
			HTraceError!(self.internal_recreate(false));
		}
	}
	
	pub fn getImageFormat(&self) -> Format
	{
		return self._swapChain.image_format();
	}
	
	pub fn getFpsLimiter(&self) -> u8
	{
		return self._fpslimiter;
	}
	
	pub fn get(&self) -> Arc<Swapchain>
	{
		return self._swapChain.clone();
	}
	
	pub fn getImages(&self) -> Vec<Arc<ImageView>>
	{
		return self._images.clone();
	}
	
	/// check if the system do support "IMMEDIATE" present mode
	pub fn canPresentMode(&self, presentmode: PresentMode) -> bool
	{
		match self._swapChain.device()
			.physical_device()
			.surface_present_modes(self._swapChain.surface(), SurfaceInfo::default()) {
			Ok(ok) => {
				let presentmodes= ok.collect::<Vec<_>>();
				if(presentmodes.contains(&presentmode))
				{
					return true;
				}
			}
			Err(_) => {}
		};
		
		return false;
	}
	
	fn getDefaults(&self) -> (PresentMode, u8)
	{
		let mut presentmode = PresentMode::Fifo;
		
		if cfg!(not(target_os = "android"))
		{
			if(self.canPresentMode(PresentMode::Immediate))
			{
				presentmode = PresentMode::Immediate;
			}
			if(self.canPresentMode(PresentMode::Mailbox))
			{
				presentmode = PresentMode::Mailbox;
			}
		}
		
		return (presentmode,0);
	}
	
	fn internal_recreate(&mut self, withconfig: bool) -> anyhow::Result<()>
	{
		let window = HGEMain::singleton().getWindowInfos();
		//println!("info win : {:?}",window.raw());
		let (mut presentmode, mut fpslimiter) = self.getDefaults();
		HGEconfig::singleton().loadSwapchainFromConfig(presentmode, fpslimiter);
		let mut nbimg = self._surfaceMinImage;
		
		if(withconfig)
		{
			let loadedconfig = HGEconfig::singleton().swapchain_get();
			presentmode = loadedconfig.presentmode;
			fpslimiter = loadedconfig.fpslimiter;
			nbimg = match loadedconfig.presentmode
			{
				PresentMode::Immediate => self._surfaceMinImage,
				PresentMode::Mailbox => self._surfaceMinImage+1,
				_ => {
					let mut img = self._surfaceMinImage;
					if cfg!(target_os = "android")
					{
						if(img<3)
						{
							img=3;
						}
					}
					img
				}
			};
			
			
			if(nbimg>self._surfaceMaxImage)
			{
				nbimg = self._surfaceMaxImage;
			}
		}
		
		match self._swapChain.recreate(SwapchainCreateInfo {
				min_image_count: nbimg,
				image_extent: window.raw(),
				present_mode: presentmode,
				pre_transform: window.orientation.into(),
				..self._swapChain.create_info()
			})
		{
			Ok((new_swapchain, new_images)) => {
				self._swapChain = new_swapchain;
				self._images = new_images
					.into_iter()
					.map(|image| ImageView::new_default(image).unwrap())
					.collect::<Vec<_>>();
				
				self._fpslimiter = fpslimiter;
				
				let _ = TSpawner!(||{
					ManagerInterface::singleton().WindowRefreshed();
				});
				return Ok(());
			},
			Err(e) => {
				return match e {
					Validated::Error(err) => Err(anyhow!("vulkan error : {}",err)),
					Validated::ValidationError(err) => Err(anyhow!("vulkan validation error : {}",err))
				}
			},
		};
	}
}
