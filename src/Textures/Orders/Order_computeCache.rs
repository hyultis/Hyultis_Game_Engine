use image::DynamicImage;
use image::imageops::FilterType;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::CopyBufferToImageInfo;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use crate::HGEMain::{HGEMain, HGEMain_secondarybuffer_type};
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Shaders::ShaderStruct::ShaderStructHolder;
use crate::Shaders::HGE_shader_3Dsimple::HGE_shader_3Dsimple_holder;
use crate::Textures::Manager::ManagerTexture;
use crate::Textures::Order::Order;
use crate::Textures::Textures::{Texture, Texture_atlasType, TextureStateGPU};

pub struct Order_computeCache
{
	sameThread : bool
}

impl Order_computeCache
{
	pub fn new() -> Self
	{
		return Order_computeCache {
			sameThread: false,
		};
	}
	
	pub fn newPrioritize() -> Self
	{
		return Order_computeCache {
			sameThread: true,
		};
	}
}

impl Order for Order_computeCache
{
	fn exec(&self, id: u32, texture: &mut Texture)
	{
		if let Some(content) = &texture.content
		{
			if texture.atlasType == Texture_atlasType::FONT
			{
				let mut builder = HGEMain::singleton().SecondaryCmdBuffer_generate();
				
				let upload_buffer = Buffer::from_iter(
					ManagerMemoryAllocator::singleton().get(),
					BufferCreateInfo {
						usage: BufferUsage::TRANSFER_SRC,
						..Default::default()
					},
					AllocationCreateInfo {
						memory_type_filter: MemoryTypeFilter::PREFER_HOST
							| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
						..Default::default()
					},
					content.to_vec(),
				)
					.unwrap();
				
				let image = Image::new(
					ManagerMemoryAllocator::singleton().get(),
					ImageCreateInfo{
						image_type: ImageType::Dim2d,
						format: texture.format,
						extent: [texture.width.unwrap_or(0),texture.height.unwrap_or(0),1],
						usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
						mip_levels: texture.mipmap,
						..ImageCreateInfo::default()
					},
					AllocationCreateInfo::default(),
				).unwrap();
				
				builder
					.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
						upload_buffer,
						image.clone(),
					))
					.unwrap();
				
				let tmp = ImageView::new_default(image).unwrap();
				texture.sendToGpu = TextureStateGPU::NOTSEND;
				
				let descriptor = PersistentDescriptorSet::new(
					&HGEMain::singleton().getAllocatorSet(),
					ManagerPipeline::singleton().layoutGetDescriptor(HGE_shader_3Dsimple_holder::pipelineName(), Texture_atlasType::FONT.getSetId()).unwrap(),
					[WriteDescriptorSet::image_view_sampler(
						0,
						tmp,
						ManagerTexture::singleton().getSampler(&texture.sampler).unwrap()
					)],
					[]
				).unwrap();
				
				HGEMain::SecondaryCmdBuffer_add(HGEMain_secondarybuffer_type::TEXTURE, builder.build().unwrap(), move ||{
					ManagerTexture::singleton().textureSetSendToGpu(id);
					ManagerTexture::singleton().fontDescriptorUpdate(descriptor.clone());
				});
				return;
			}
			
			let size = texture.atlasType.getSize();
			if(texture.width.unwrap_or(1) != size || texture.height.unwrap_or(1) != size)
			{
				let tmp = DynamicImage::from(content.clone());
				let tmp = tmp.resize_exact(size, size, FilterType::Gaussian);
				texture.contentSizeAtlas = Some(tmp.as_rgba8().unwrap().as_raw().clone());
			}
			else
			{
				texture.contentSizeAtlas = Some(content.as_raw().clone());
			}
			
			texture.sendToGpu = TextureStateGPU::SEND;
			texture.clearContent();
			
			/*
				let mut builder = HGEMain::singleton().SecondaryCmdBuffer_generate();
				let image = ImmutableImage::from_iter(
					&ManagerMemoryAllocator::singleton().get(),
					content.to_vec(),
					ImageDimensions::Dim2d {
						width: texture.width.unwrap_or(0),
						height: texture.height.unwrap_or(0),
						array_layers: 1,
					},
					texture.mipmap,
					texture.format,
					&mut builder,
				).unwrap();
				
				texture.cache = Some(ImageView::new_default(image).unwrap());
				texture.sendToGpu = TextureStateGPU::NOTSEND;
				HGEMain::singleton().SecondaryCmdBuffer_add(HGEMain_secondarybuffer_type::TEXTURE, builder.build().unwrap(), move ||{ManagerTexture::singleton().textureSetSendToGpu(id)});
			}*/
		}
	}
	
	fn isSameThread(&self) -> bool {
		self.sameThread
	}
	
	fn isWaiting(&mut self) -> bool
	{
		false
	}
}
