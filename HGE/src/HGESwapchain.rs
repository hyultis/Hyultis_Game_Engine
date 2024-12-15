use crate::configs::HGEconfig::HGEconfig;
use crate::BuilderDevice::BuilderDevice;
use crate::HGEMain::HGEMain;
use crate::Interface::ManagerInterface::ManagerInterface;
use anyhow::anyhow;
use std::sync::Arc;
use vulkano::device::DeviceOwned;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::ImageUsage;
use vulkano::swapchain::{
	CompositeAlpha, PresentMode, Surface, SurfaceInfo, SurfaceTransform, Swapchain,
	SwapchainCreateInfo,
};
use vulkano::Validated;
use Htrace::{namedThread, HTrace, HTraceError};

#[derive(Clone)]
pub struct HGESwapchain
{
	_swapChain: Arc<Swapchain>,
	_surfaceMinImage: u32,
	_surfaceMaxImage: u32,
	_images: Vec<Arc<ImageView>>,
	_fpslimiter: u8,
	_systemDesync: bool,
}

impl HGESwapchain
{
	pub fn new(builderDevice: Arc<BuilderDevice>, surface: Arc<Surface>) -> Self
	{
		let surface_capabilities = builderDevice
			.device
			.physical_device()
			.surface_capabilities(&surface, Default::default())
			.unwrap();
		let image_formats = builderDevice
			.device
			.physical_device()
			.surface_formats(&surface, Default::default())
			.unwrap();

		// can B8G8R8A8_UNORM ?
		let mut format = image_formats[0].0;
		image_formats.iter().for_each(|(thisformat, _)| {
			HTrace!("format image allowed : {:?}", thisformat);
			if (*thisformat == Format::R8G8B8A8_UNORM)
			{
				format = Format::R8G8B8A8_UNORM;
			}
			if (*thisformat == Format::B8G8R8A8_UNORM)
			{
				format = Format::B8G8R8A8_UNORM;
			}
		});
		HTrace!("surface image format : {:?}", image_formats);

		let (swapchain, images) = match Swapchain::new(
			builderDevice.device.clone(),
			surface.clone(),
			SwapchainCreateInfo {
				min_image_count: surface_capabilities.min_image_count,
				image_format: format,
				image_extent: surface_capabilities.min_image_extent,
				image_usage: ImageUsage::TRANSFER_DST | ImageUsage::COLOR_ATTACHMENT, // ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST |
				present_mode: PresentMode::Fifo,                                      // default
				pre_transform: SurfaceTransform::Identity,
				composite_alpha: surface_capabilities
					.supported_composite_alpha
					.into_iter()
					.next()
					.unwrap_or(CompositeAlpha::Inherit),
				..Default::default()
			},
		)
		{
			Ok(x) => x,
			Err(err) => panic!("vulkan validation error : {}", err),
		};

		let images = images
			.into_iter()
			.map(|image| ImageView::new_default(image).unwrap())
			.collect::<Vec<_>>();

		return HGESwapchain {
			_swapChain: swapchain,
			_surfaceMinImage: surface_capabilities.min_image_count,
			_surfaceMaxImage: surface_capabilities
				.max_image_count
				.unwrap_or(surface_capabilities.min_image_count),
			_images: images,
			_fpslimiter: 0,
			_systemDesync: true,
		};
	}

	pub fn recreate(&mut self)
	{
		let result = self.internal_recreate(true);
		if result.is_err()
		{
			println!("retry internal_recreate : {:?}", result);
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
		match self
			._swapChain
			.device()
			.physical_device()
			.surface_present_modes(self._swapChain.surface(), SurfaceInfo::default())
		{
			Ok(ok) =>
			{
				if (ok.contains(&presentmode))
				{
					return true;
				}
			}
			Err(_) =>
			{}
		};

		return false;
	}

	fn getDefaults(&self) -> (PresentMode, u8)
	{
		let mut presentmode = PresentMode::Fifo;

		if cfg!(not(target_os = "android"))
		{
			if (self.canPresentMode(PresentMode::Immediate))
			{
				presentmode = PresentMode::Immediate;
			}
			if (self.canPresentMode(PresentMode::Mailbox))
			{
				presentmode = PresentMode::Mailbox;
			}
		}

		return (presentmode, 0);
	}

	fn internal_recreate(&mut self, withconfig: bool) -> anyhow::Result<()>
	{
		let window = HGEMain::singleton().getWindowInfos();
		let surfaceCap = match &window.surfaceCapabilities
		{
			None => return Err(anyhow!("no surface capabilities found")),
			Some(x) => x.clone(),
		};

		println!(
			"info win : {:?} {:?} {:?}",
			window.raw(),
			surfaceCap.min_image_extent,
			surfaceCap.max_image_extent
		);

		let (mut presentmode, mut fpslimiter) = self.getDefaults();
		HGEconfig::singleton().loadSwapchainFromConfig(presentmode, fpslimiter);
		let mut nbimg = surfaceCap.min_image_count;

		if (withconfig)
		{
			let loadedconfig = HGEconfig::singleton().swapchain_get();
			presentmode = loadedconfig.presentmode;
			fpslimiter = loadedconfig.fpslimiter;
			nbimg = match &loadedconfig.presentmode
			{
				PresentMode::Immediate => 1,
				PresentMode::Mailbox => 3,
				_ => surfaceCap.min_image_count,
			};
		}

		println!(
			"image_count : {:?} {} {} {:?} {:?}",
			presentmode,
			nbimg,
			surfaceCap.min_image_count,
			surfaceCap.max_image_count,
			surfaceCap.compatible_present_modes
		);
		if (nbimg < surfaceCap.min_image_count + 1)
		{
			nbimg = surfaceCap.min_image_count + 1;
		}

		if let Some(max_image_count) = surfaceCap.max_image_count
		{
			if (nbimg > max_image_count)
			{
				nbimg = max_image_count;
			}
		}

		println!("swapchain recreate");
		match self._swapChain.recreate(SwapchainCreateInfo {
			min_image_count: nbimg,
			image_extent: window.raw(),
			present_mode: presentmode,
			pre_transform: window.orientation.into(),
			..self._swapChain.create_info()
		})
		{
			Ok((new_swapchain, new_images)) =>
			{
				println!("swapchain recreated");
				self._swapChain = new_swapchain;
				self._images = new_images
					.into_iter()
					.map(|image| ImageView::new_default(image).unwrap())
					.collect::<Vec<_>>();

				self._fpslimiter = fpslimiter;

				let _ = namedThread!(|| {
					ManagerInterface::singleton().WindowRefreshed();
				});
				return Ok(());
			}
			Err(e) =>
			{
				println!("swapchain error");
				return match e
				{
					Validated::Error(err) => Err(anyhow!("vulkan error : {}", err)),
					Validated::ValidationError(err) =>
					{
						Err(anyhow!("vulkan validation error : {}", err))
					}
				};
			}
		};
	}
}
