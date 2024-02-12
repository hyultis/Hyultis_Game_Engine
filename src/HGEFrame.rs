use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::format::Format;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use crate::HGEMain::HGEMain;
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;

#[derive(Clone)]
pub struct HGEFrame
{
	_img_depthUI: Arc<ImageView>,
	_img_depthSolid: Arc<ImageView>,
	_img_render_UI: Arc<ImageView>,
	_img_render_WorldSolid: Arc<ImageView>,
	_img_render_Full: Arc<ImageView>,
	_img_size: [u32;2],
	_frames: Vec<Arc<Framebuffer>>,
	_ouputFormat: Format
}

impl HGEFrame
{
	pub fn new(format: Format) -> HGEFrame {
		
		let newsize = [1,1];
		let imgsize = [1,1,1];
		
		let tmp1 = HGEFrame::generateNewDefaultImg(imgsize,format, ImageUsage::COLOR_ATTACHMENT | ImageUsage::INPUT_ATTACHMENT);
		let tmp2 = HGEFrame::generateNewDefaultImg(imgsize,format, ImageUsage::COLOR_ATTACHMENT | ImageUsage::INPUT_ATTACHMENT);
		let tmp3 = HGEFrame::generateNewDefaultImg(imgsize,format, ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC);
		let depth1 = HGEFrame::generateNewDefaultImgDepth(imgsize);
		let depth2 = HGEFrame::generateNewDefaultImgDepth(imgsize);
		
		return HGEFrame {
			_img_depthUI: depth1,
			_img_depthSolid: depth2,
			_img_render_UI: tmp1,
			_img_render_WorldSolid: tmp2,
			_img_render_Full: tmp3,
			_img_size: newsize,
			_frames: Vec::new(),
			_ouputFormat: format,
		};
	}
	
	pub fn clearBuffer(&self, cmdBuf: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>, image_index: u32)
	{
		cmdBuf.begin_render_pass(
			RenderPassBeginInfo {
				clear_values: vec![
					Some([0.0, 0.0, 0.0, 0.0].into()),
					Some([0.0, 0.0, 0.0, 0.0].into()), //Some([0.0, 0.0, 1.0, 0.0].into()),
					#[cfg(feature = "dynamicresolution")]
					Some([0.0, 0.0, 0.0, 0.0].into()),
					Some([0.0, 0.0, 0.0, 0.0].into()),
					Some(1f32.into()),
					Some(1f32.into()),
				],
				..RenderPassBeginInfo::framebuffer(
					self.get(image_index as usize) ,
				)
			},
			SubpassBeginInfo{
				contents: SubpassContents::SecondaryCommandBuffers,
				..SubpassBeginInfo::default()
			},
		).unwrap();
	}
	
	pub fn add(&mut self, images: Vec<Arc<ImageView>>, render_pass: Arc<RenderPass>)
	{
		self.resize();
		
		let framebuffers = images
		.into_iter()
		.map(|render_final| {
			Framebuffer::new(
				render_pass.clone(),
				FramebufferCreateInfo {
					attachments: vec![
						self._img_render_UI.clone(),
						self._img_render_WorldSolid.clone(),
						#[cfg(feature = "dynamicresolution")]
						self._img_render_Full.clone(),
						render_final,
						self._img_depthUI.clone(),
						self._img_depthSolid.clone(),
					],
					..Default::default()
				},
			)
				.unwrap()
		})
		.collect::<Vec<_>>();
		self._frames = framebuffers;
	}
	
	fn resize(&mut self)
	{
		let newsize: [u32;2] = HGEMain::singleton().getWindowInfos().into();
		
		if(newsize == self._img_size)
		{
			return;
		}
		
		let imgsize = [newsize[0],newsize[1],1];
		self._img_render_UI = HGEFrame::generateNewDefaultImg(imgsize, self._ouputFormat, ImageUsage::COLOR_ATTACHMENT | ImageUsage::INPUT_ATTACHMENT);
		self._img_render_WorldSolid = HGEFrame::generateNewDefaultImg(imgsize, self._ouputFormat, ImageUsage::COLOR_ATTACHMENT | ImageUsage::INPUT_ATTACHMENT);
		self._img_render_Full = HGEFrame::generateNewDefaultImg(imgsize, self._ouputFormat, ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC);
		self._img_depthUI = HGEFrame::generateNewDefaultImgDepth(imgsize);
		self._img_depthSolid = HGEFrame::generateNewDefaultImgDepth(imgsize);
		
		self._img_size = newsize;
	}
	
	pub fn get(&self, image_index: usize) -> Arc<Framebuffer>
	{
		return self._frames[image_index].clone();
	}
	
	pub fn getImgUI(&self) -> Arc<ImageView>
	{
		return self._img_render_UI.clone();
	}
	
	pub fn getImgWS(&self) -> Arc<ImageView>
	{
		return self._img_render_WorldSolid.clone();
	}
	
	pub fn getImgFull(&self) -> Arc<ImageView>
	{
		return self._img_render_Full.clone();
	}
		
	pub fn getImgUIDepth(&self) -> Arc<ImageView>
	{
		return self._img_depthUI.clone();
	}
	pub fn getImgUIDepthSolid(&self) -> Arc<ImageView>
	{
		return self._img_depthSolid.clone();
	}
	
	////// PRIVATE //////////////
	
	fn generateNewDefaultImgDepth(newsize: [u32;3]) -> Arc<ImageView>
	{
		let depthformat = HGEMain::singleton().getDevice().depthformat;
		let mut resultimg = Image::new(
			ManagerMemoryAllocator::singleton().get(),
			ImageCreateInfo{
				image_type: ImageType::Dim2d,
				format: depthformat,
				extent: newsize,
				usage: ImageUsage::TRANSIENT_ATTACHMENT | ImageUsage::DEPTH_STENCIL_ATTACHMENT,
				..ImageCreateInfo::default()
			},
			AllocationCreateInfo::default(),
		);
		
		if let Err(_) = &resultimg
		{
			resultimg = Image::new(
				ManagerMemoryAllocator::singleton().get(),
				ImageCreateInfo{
					image_type: ImageType::Dim2d,
					format: depthformat,
					extent: newsize,
					usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT, // ImageUsage::TRANSIENT_ATTACHMENT | // cause unwrap MemoryAllocatorError::FindMemoryType on some mobile device
					..ImageCreateInfo::default()
				},
				AllocationCreateInfo::default(),
			);
		}
		
		return ImageView::new_default(
			resultimg.unwrap()
		).unwrap();
	}
	
	fn try_stuff(image: ImageCreateInfo)
	{
		match Image::new(
			ManagerMemoryAllocator::singleton().get(),
			image,
			AllocationCreateInfo::default(),
		) {
			Ok(_) => println!("OK !"),
			Err(err) => println!("NOK ! {:?}",err),
		};
	}
	
	fn generateNewDefaultImgFakeDepth(newsize: [u32;3]) -> Arc<ImageView>
	{
		return ImageView::new_default(
			Image::new(
				ManagerMemoryAllocator::singleton().get(),
				ImageCreateInfo{
					image_type: ImageType::Dim2d,
					format: Format::R8_UNORM,
					extent: newsize,
					usage: ImageUsage::INPUT_ATTACHMENT,
					..ImageCreateInfo::default()
				},
				AllocationCreateInfo::default(),
			).unwrap(),
		).unwrap();
	}
	
	pub fn generateNewDefaultImg(newsize: [u32;3], format: Format, usage: ImageUsage) -> Arc<ImageView>
	{
		return ImageView::new_default(
			Image::new(
				ManagerMemoryAllocator::singleton().get(),
				ImageCreateInfo{
					image_type: ImageType::Dim2d,
					format,
					extent: newsize,
					usage,
					..ImageCreateInfo::default()
				},
				AllocationCreateInfo::default(),
			).unwrap(),
		).unwrap();
	}
}
