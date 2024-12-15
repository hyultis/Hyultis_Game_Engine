use crate::HGEMain::{HGEMain, HGEMain_secondarybuffer_type};
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Textures::generate;
use crate::Textures::Manager::ManagerTexture;
use crate::Textures::TextureDescriptor_type::{
	TextureDescriptor_adaptedTexture, TextureDescriptor_exclude, TextureDescriptor_process,
	TextureDescriptor_type,
};
use crate::Textures::Textures::Texture;
use crate::Textures::Types::TextureChannel;
use anyhow::anyhow;
use arc_swap::{ArcSwap, Guard};
use dashmap::DashMap;
use image::imageops::FilterType;
use image::{DynamicImage, RgbaImage};
use parking_lot::RwLock;
use std::ops::{Range, RangeBounds};
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::{
	AutoCommandBufferBuilder, CopyBufferToImageInfo, SecondaryAutoCommandBuffer,
};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
use vulkano::image::{
	Image, ImageAspects, ImageCreateInfo, ImageSubresourceRange, ImageType, ImageUsage,
};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use Htrace::HTrace;
use Htrace::Type::Type::ERROR;

pub struct TextureDescriptor
{
	// define
	_type: TextureDescriptor_type<Range<u16>>,
	_exclude: TextureDescriptor_exclude,
	_shaderName: String,
	_shaderSetid: usize,
	_sampler: String,

	// datas
	_texturelink: DashMap<String, u32>,
	_textureLinkMax: RwLock<u32>,
	_mustUpdate: ArcSwap<u64>,
	_lastUpdate: ArcSwap<u64>,
	_defaultTexture: Vec<u8>,

	// cache
	_contentAdapted: DashMap<u32, TextureDescriptor_adaptedTexture>,
	_cacheVulkan: ArcSwap<DescriptorSet>,
}

impl TextureDescriptor
{
	pub fn new<T: RangeBounds<u16>>(
		ttype: TextureDescriptor_type<T>,
		exclude: TextureDescriptor_exclude,
		shaderName: impl Into<String>,
		shaderSetId: usize,
		samplerName: impl Into<String>,
	) -> Self
	{
		let mut defaultTexture = DynamicImage::from(generate::defaultTexture());
		match Self::getProcess(&ttype)
		{
			TextureDescriptor_process::RAW =>
			{}
			TextureDescriptor_process::RESIZE(x, y) =>
			{
				defaultTexture =
					defaultTexture.resize_exact(*x as u32, *y as u32, FilterType::Gaussian);
			}
		};

		let shaderName = shaderName.into();
		let samplerName = samplerName.into();
		let empty_cache =
			Self::generate_empty_descriptorCache(&ttype, &shaderName, shaderSetId, &samplerName);

		return Self {
			_type: ttype.normalize(),
			_exclude: exclude,
			_shaderName: shaderName.into(),
			_shaderSetid: shaderSetId,
			_sampler: samplerName.into(),
			_texturelink: Default::default(),
			_textureLinkMax: RwLock::new(0),
			_mustUpdate: ArcSwap::new(Arc::new(0)),
			_lastUpdate: ArcSwap::new(Arc::new(0)),
			_defaultTexture: RgbaImage::from(defaultTexture).as_raw().clone(),
			_contentAdapted: Default::default(),
			_cacheVulkan: ArcSwap::new(empty_cache),
		};
	}

	pub fn texture_getId(&self, texture_name: impl Into<String>) -> Option<TextureChannel>
	{
		let texture_name = texture_name.into();
		self._texturelink
			.get(&texture_name)
			.map(|x| TextureChannel::new(self._shaderSetid as u8, *x.value()))
	}

	pub fn getDescriptor(&self) -> Guard<Arc<DescriptorSet>>
	{
		return self._cacheVulkan.load();
	}

	pub fn texture_isUpdated(&self, texture: &Texture) -> bool
	{
		let mut isok = match &self._type
		{
			TextureDescriptor_type::ALL(_) => true,
			TextureDescriptor_type::ONE(name) => *name == texture.name,
			TextureDescriptor_type::ARRAY(names, _) => names.contains(&texture.name),
			TextureDescriptor_type::SIZE_DEPENDENT(xy, _) =>
			{
				let mut returned = false;
				if let Some(width) = texture.width
				{
					if let Some(height) = texture.height
					{
						returned = xy.contains(&(width as u16)) && xy.contains(&(height as u16))
					}
				}
				returned
			}
			TextureDescriptor_type::SIZE_DEPENDENT_XY(x, y, _) =>
			{
				let mut returned = false;
				if let Some(width) = texture.width
				{
					if let Some(height) = texture.height
					{
						returned = x.contains(&(width as u16)) && y.contains(&(height as u16))
					}
				}
				returned
			}
			TextureDescriptor_type::SIZE_MIN(xy, _) =>
			{
				let mut returned = false;
				if let Some(width) = texture.width
				{
					if let Some(height) = texture.height
					{
						returned = xy.contains(&(width as u16)) || xy.contains(&(height as u16))
					}
				}
				returned
			}
		};

		match &self._exclude
		{
			TextureDescriptor_exclude::NONE =>
			{}
			TextureDescriptor_exclude::ARRAY(names) =>
			{
				if (names.contains(&texture.name))
				{
					isok = false;
				}
			}
		}

		if (!isok)
		{
			return false;
		}

		let id = self.texture_getOrAddId(texture.name.clone());
		self.texture_updateData(texture, id);
		let val = **self._mustUpdate.load();
		self._mustUpdate.swap(Arc::new(val + 1));
		return true;
	}

	pub fn update(&self)
	{
		let mustupdate = **self._mustUpdate.load();
		if (mustupdate == **self._lastUpdate.load() || *self._textureLinkMax.read() == 0)
		{
			return;
		}

		let mut cmdbuff = HGEMain::singleton().SecondaryCmdBuffer_generate();
		let result = match Self::getProcess(&self._type)
		{
			TextureDescriptor_process::RAW => self.update_all(&mut cmdbuff),
			TextureDescriptor_process::RESIZE(x, y) => self.update_resize(*x, *y, &mut cmdbuff),
		};

		let atlas = match result
		{
			Ok(atlas) => atlas,
			Err(err) =>
			{
				HTrace!((ERROR) err);
				return;
			}
		};

		let defaultsampler = ManagerTexture::singleton()
			.getSampler(&self._sampler)
			.unwrap()
			.clone();
		self._cacheVulkan.swap(
			DescriptorSet::new(
				HGEMain::singleton().getDescAllocatorSet(),
				ManagerPipeline::singleton()
					.layoutGetDescriptor(&self._shaderName, self._shaderSetid)
					.unwrap(),
				[WriteDescriptorSet::image_view_sampler(
					0,
					atlas,
					defaultsampler,
				)],
				[],
			)
			.unwrap(),
		);

		self._lastUpdate.swap(Arc::new(mustupdate));

		HGEMain::SecondaryCmdBuffer_add(
			HGEMain_secondarybuffer_type::TEXTURE,
			cmdbuff.build().unwrap(),
			|| {},
		);
	}

	//////////////// PRIVATE ///////////////

	fn getProcess<T: RangeBounds<u16>>(
		ttype: &TextureDescriptor_type<T>,
	) -> &TextureDescriptor_process
	{
		match ttype
		{
			TextureDescriptor_type::ALL(x) => x,
			TextureDescriptor_type::ONE(_) => &TextureDescriptor_process::RAW,
			TextureDescriptor_type::ARRAY(_, x) => x,
			TextureDescriptor_type::SIZE_DEPENDENT(_, x) => x,
			TextureDescriptor_type::SIZE_DEPENDENT_XY(_, _, x) => x,
			TextureDescriptor_type::SIZE_MIN(_, x) => x,
		}
	}

	fn texture_getOrAddId(&self, texture_name: impl Into<String>) -> u32
	{
		let texture_name = texture_name.into();
		if let Some(id) = self._texturelink.get(&texture_name)
		{
			return *id;
		}

		let mut binding = self._textureLinkMax.write();
		let id = *binding;
		*binding += 1;

		self._texturelink.insert(texture_name, id);
		return id;
	}

	fn texture_updateData(&self, texture: &Texture, id: u32)
	{
		let contentorigin = texture.content.clone().unwrap();

		let contentadapted = match Self::getProcess(&self._type)
		{
			TextureDescriptor_process::RAW => TextureDescriptor_adaptedTexture {
				x: contentorigin.width() as u16,
				y: contentorigin.height() as u16,
				content: contentorigin.as_raw().clone(),
			},
			TextureDescriptor_process::RESIZE(x, y) => self.texture_resize(contentorigin, *x, *y),
		};

		self._contentAdapted.insert(id, contentadapted);
	}

	fn texture_resize(&self, origin: RgbaImage, x: u16, y: u16)
		-> TextureDescriptor_adaptedTexture
	{
		let mut returned = TextureDescriptor_adaptedTexture {
			x: origin.width() as u16,
			y: origin.height() as u16,
			content: Vec::new(),
		};

		let tmp = DynamicImage::from(origin);
		let tmp = tmp.resize_exact(x as u32, y as u32, FilterType::Triangle);

		returned.content = tmp.as_rgba8().unwrap().as_raw().clone();

		return returned;
	}

	fn update_all(
		&self,
		cmdbuff: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
	) -> anyhow::Result<Arc<ImageView>>
	{
		match self._type
		{
			TextureDescriptor_type::ONE(_) =>
			{}
			_ =>
			{
				return Err(anyhow!("Unsupported RAW update for multiple image"));
			}
		}

		let Some(texture) = self._contentAdapted.get(&0)
		else
		{
			return Err(anyhow!("Image not adapted"));
		};

		return Self::generate_atlas_imageview(
			&self._type,
			texture.content.clone(),
			texture.x as u32,
			texture.y as u32,
			1,
			cmdbuff,
		);
	}

	fn update_resize(
		&self,
		x: u16,
		y: u16,
		cmdbuff: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
	) -> anyhow::Result<Arc<ImageView>>
	{
		let nbmax = *self._textureLinkMax.read();
		if (nbmax == 0)
		{
			return Err(anyhow!("no texture link"));
		}
		let mut finalData: Vec<u8> = Vec::new();
		for x in 0..nbmax
		{
			if let Some(content) = self._contentAdapted.get(&x)
			{
				finalData.extend_from_slice(content.content.as_slice());
			}
			else
			{
				finalData.extend_from_slice(self._defaultTexture.as_slice());
			}
		}

		return Self::generate_atlas_imageview(
			&self._type,
			finalData,
			x as u32,
			y as u32,
			nbmax,
			cmdbuff,
		);
	}

	fn generate_atlas_imageview<T: RangeBounds<u16>>(
		ttype: &TextureDescriptor_type<T>,
		finalData: Vec<u8>,
		x: u32,
		y: u32,
		mut nblayer: u32,
		cmdbuff: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
	) -> anyhow::Result<Arc<ImageView>>
	{
		let upload_buffer = match Buffer::from_iter(
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
			finalData,
		)
		{
			Ok(x) => x,
			Err(err) => return Err(anyhow!("Cannot create buffer atlas : {}", err)),
		};

		let atlas = Image::new(
			ManagerMemoryAllocator::singleton().get(),
			ImageCreateInfo {
				image_type: ImageType::Dim2d,
				format: Format::R8G8B8A8_UNORM,
				extent: [x, y, 1],
				array_layers: nblayer,
				usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
				..ImageCreateInfo::default()
			},
			AllocationCreateInfo::default(),
		)?;

		if let Err(err) = cmdbuff.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
			upload_buffer,
			atlas.clone(),
		))
		{
			return Err(anyhow!("Cannot create final atlas : {}", err));
		}

		let viewtype = match ttype
		{
			TextureDescriptor_type::ONE(_) =>
			{
				nblayer = 1;
				ImageViewType::Dim2d
			}
			_ => ImageViewType::Dim2dArray,
		};

		let atlas = ImageView::new(
			atlas,
			ImageViewCreateInfo {
				view_type: viewtype,
				format: Format::R8G8B8A8_UNORM,
				subresource_range: ImageSubresourceRange {
					aspects: ImageAspects::COLOR,
					mip_levels: 0..1,
					array_layers: 0..nblayer,
				},
				usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
				..ImageViewCreateInfo::default()
			},
		)?;

		return Ok(atlas);
	}

	fn generate_empty_descriptorCache<T: RangeBounds<u16>>(
		ttype: &TextureDescriptor_type<T>,
		shaderName: &String,
		shaderSetId: usize,
		samplerName: &String,
	) -> Arc<DescriptorSet>
	{
		let mut combuilder = HGEMain::singleton().SecondaryCmdBuffer_generate();

		let result =
			Self::generate_atlas_imageview(ttype, vec![0, 0, 0, 0], 1, 1, 1, &mut combuilder)
				.unwrap();

		let returned = DescriptorSet::new(
			HGEMain::singleton().getDescAllocatorSet(),
			ManagerPipeline::singleton()
				.layoutGetDescriptor(shaderName, shaderSetId)
				.unwrap(),
			[WriteDescriptorSet::image_view_sampler(
				0,
				result,
				ManagerTexture::singleton()
					.getSampler(samplerName)
					.unwrap()
					.clone(),
			)],
			[],
		)
		.unwrap();

		HGEMain::SecondaryCmdBuffer_add(
			HGEMain_secondarybuffer_type::TEXTURE,
			combuilder.build().unwrap(),
			|| {},
		);

		return returned;
	}
}
