use std::sync::{Arc, OnceLock};
use std::vec;
use dashmap::DashMap;
use Htrace::HTrace;
use Htrace::HTracer::HTracer;
use parking_lot::{Mutex, RwLock};
use rayon::iter::IntoParallelRefIterator;
use vulkano::command_buffer::CopyBufferToImageInfo;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::image::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode};
use crate::HGEMain::{HGEMain, HGEMain_secondarybuffer_type};
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Textures::Textures::{Texture, Texture_part, TextureState};
use crate::Textures::generate;
use crate::Textures::Order::Order;
use crate::Textures::Orders::Order_load::Order_load;
use crate::Textures::Orders::Order_loadPart::Order_loadPart;
use crate::Textures::textureLoader::{textureLoader_fromFile, textureLoader_fromRaw};
use crate::Textures::texturePart::texturePart;
use singletonThread::SingletonThread;
use vulkano::format::Format;
use vulkano::image::{Image, ImageAspects, ImageCreateInfo, ImageSubresourceRange, ImageType, ImageUsage};
use crate::ManagerMemoryAllocator::ManagerMemoryAllocator;
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
use rayon::iter::ParallelIterator;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use crate::Shaders::HGE_shader_3Dsimple::HGE_shader_3Dsimple_holder;
use crate::Shaders::ShaderStruct::ShaderStructHolder;
use HArcMut::HArcMut;
use crate::Textures::TextureDescriptor::TextureDescriptor;
use crate::Textures::Types::TextureChannel;

pub struct ManagerTexture
{
	_textures: DashMap<String, HArcMut<Texture>>,
	_texturesOrder: DashMap<String, HArcMut<Vec<Box<dyn Order + Send + Sync>>>>,
	_texturesCallback: DashMap<String, Box<dyn Fn(&mut Texture) + Send + Sync + 'static>>,
	_samplers: DashMap<String, Arc<Sampler>>,
	_NbTotalTexture: RwLock<u32>,
	_NbTotalLoadedTexture: RwLock<u32>,
	
	_descriptorSets: DashMap<String, TextureDescriptor>,
	
	// thread stuff
	_threadLoading: Mutex<SingletonThread>,
	_threadUpdateDescriptorSets: Mutex<SingletonThread>,
	_haveOneOrderUpdate: RwLock<bool>,
}

static SINGLETON: OnceLock<ManagerTexture> = OnceLock::new();

impl ManagerTexture
{
	pub fn singleton() -> &'static Self
	{
		return SINGLETON.get_or_init(|| {
			Self::new()
		});
	}
	
	pub fn preload(&self)
	{
		let defaultTexture = generate::defaultTexture();
		self.texture_load("default", Order_load::new(
			textureLoader_fromRaw{
				raw: defaultTexture.to_vec(),
				width: defaultTexture.width(),
				height: defaultTexture.height(),
				canReload: true,
			}), Some("pixeled"));
	}
	
	pub fn addSampler(&self, name: impl Into<String>, mut newsampler: SamplerCreateInfo)
	{
		let device = HGEMain::singleton().getDevice().device.clone();
		if(!device.enabled_features().sampler_anisotropy)
		{
			newsampler.anisotropy = None;
		}
		let newsampler = Sampler::new(device, newsampler).unwrap();
		self._samplers.insert(name.into(), newsampler);
	}
	
	pub fn getSampler(&self, name: impl Into<String>) -> Option<Arc<Sampler>>
	{
		match self._samplers.get(&name.into()) {
			None => None,
			Some(sampler) => Some(sampler.value().clone())
		}
	}
	
	pub fn texture_load(&self, name: impl Into<String>, loadOrder: Order_load, sampler: Option<&str>)
	{
		let name: String = name.into();
		let sampler = sampler.unwrap_or("default").to_string();
		HTrace!("ManagerTexture: load {} with sampler {}",name,sampler);
		
		self._textures.insert(name.clone(), HArcMut::new(Texture {
			name: name.clone(),
			sampler,
			..Texture::default()
		}));
		self._texturesOrder.insert(name.clone(),HArcMut::new(Vec::new()));
		*self._NbTotalTexture.write() += 1;
		
		if(loadOrder.isSameThread())
		{
			if let Some(texture ) = self._textures.get(&name)
			{
				texture.update(|texturemut|{
					
					loadOrder.exec(texturemut);
					self.updateTextureOnAllDescriptor(texturemut);
				});
			}
		}
		else
		{
			self.orderAdd(name.clone(),vec![Box::new(loadOrder)]);
		}
	}
	
	pub fn texture_update(&self, name: impl Into<String>, mut updateOrders: Vec<Box<dyn Order + Sync + Send + 'static>>)
	{
		let name = name.into();
		if let Some(bind) = self._textures.get(&name)
		{
			bind.updateIf(|texture|{
				let mut updated = false;
				updateOrders.retain(|sameThreadOrder|{
					if (sameThreadOrder.isSameThread())
					{
						sameThreadOrder.exec(texture);
						updated = true;
						return false;
					}
					else
					{
						return true;
					}
				});
				
				if(updated)
				{
					self.updateTextureOnAllDescriptor(texture);
				}
				
				return false;
			});
			
			self.orderAdd(bind.key().clone(),updateOrders);
		}
	}
	
	pub fn textureDebug(&self)
	{
		for x in self._textures.iter()
		{
			let bind = x.get();
			HTrace!("textureDebug {} : state {:?}",bind.name,bind.state);
		}
	}
	
	pub fn texture_reloadAll(&self)
	{
		for x in self._textures.iter()
		{
			let tmp = x.get();
			if let Some(reloader) = &tmp.reloadLoader
			{
				HTrace!("texture {} : put in reload",tmp.name);
				self.orderAdd(x.key().clone(),vec![Box::new(reloader.clone())]);
			}
		}
	}
	
	pub fn texture_reload(&self, name: impl Into<String>)
	{
		match self._textures.get(&name.into()) {
			None => {}
			Some(x) => {
				if let Some(reloader) = &x.get().reloadLoader
				{
					self.orderAdd(x.key().clone(),vec![Box::new(reloader.clone())]);
				}
			}
		}
	
	}
	
	pub fn texture_setCallback(&self, name: impl Into<String>, func: impl Fn(&mut Texture) + Send + Sync + 'static)
	{
		if let Some(texture) = self._textures.get(&name.into())
		{
			self._texturesCallback.insert(texture.key().clone(),Box::new(func));
		}
	}
	
	pub fn add(&self, name: impl Into<String>, texturePath: impl Into<String>, sampler: Option<&str>)
	{
		self.texture_load(name.into(), Order_load::new(
			textureLoader_fromFile{
				path: texturePath.into(),
			}), sampler);
	}
	
	pub fn addRawPrioritize(&self, name: &str, textureRaw: Vec<u8>, width: u32, height: u32, sampler: Option<&str>)
	{
		self.texture_load(name, Order_load::newPrioritize(
			textureLoader_fromRaw{
				raw: textureRaw,
				width,
				height,
				canReload: true,
			}), sampler);
	}
	
	pub fn get(&self, name: impl Into<String>) -> Option<Arc<Texture>>
	{
		let Some(x) = self._textures.get(&name.into()) else {return None;};
		let tmp = (&*x.get()).clone();
		Some(tmp)
	}
	
	pub fn getState(&self, name: impl Into<String>) -> Option<TextureState>
	{
		return self._textures.get(&name.into()).map(|x|{x.get().state.clone()});
	}
	
	pub fn texture_loadPart(&self, name: impl Into<String>, partLoader: impl texturePart + Sync + Send + 'static)
	{
		self.orderAdd(name.into(),vec![Box::new(Order_loadPart{
			from: Box::new(partLoader),
		})]);
	}
	
	pub fn getPart(&self, name: impl Into<String>, part: impl Into<String>) -> Option<Texture_part>
	{
		let texture = self._textures.get(&name.into());
		match texture {
			None => None,
			Some(x) => {x.get().partUVCoord.get(&part.into()).copied()}
		}
	}
	
	pub fn launchThreads(&self)
	{
		self._threadLoading.lock().thread_launch();
		self._threadUpdateDescriptorSets.lock().thread_launch();
	}
	
	pub fn descriptorSet_create(&self, descriptorSetName: impl Into<String>, textureDescriptor: TextureDescriptor)
	{
		self._descriptorSets.insert(descriptorSetName.into(),textureDescriptor);
	}
	
	pub fn descriptorSet_getVulkanCache(&self, descriptorSetName: impl Into<String>) -> Option<Arc<PersistentDescriptorSet>>
	{
		let descriptorSetName = descriptorSetName.into();
		return self._descriptorSets.get(&descriptorSetName).map(|x| { x.value().getDescriptor().clone() });
	}
	
	pub fn descriptorSet_getIdTexture(&self, descriptorSetNames: impl IntoIterator<Item = impl Into<String>>, textureName: impl Into<String>) -> Option<TextureChannel>
	{
		let descriptorSetNames = descriptorSetNames.into_iter();
		let textureName = textureName.into();
		for name in descriptorSetNames
		{
			let name = name.into();
			if let Some(descriptor) = self._descriptorSets.get(&name)
			{
				if let Some(result) = descriptor.texture_getId(&textureName)
				{
					return Some(result);
				}
			}
		}
		return None;
	}
	
	pub fn getNbTexture(&self) -> u32
	{
		return *self._NbTotalTexture.read();
	}
	
	
	pub fn getNbLoadedTexture(&self) -> u32
	{
		return self._textures.iter().filter(|x|{
			x.get().state == TextureState::LOADED
		}).count() as u32;
	}
	
	///////////// PRIVATE ///////////
	
	fn new() -> ManagerTexture {
		let device = HGEMain::singleton().getDevice().device.clone();
		
		let samplers = DashMap::new();
		samplers.insert("default".to_string(), Self::defaultSampler());
		samplers.insert("pixeled".to_string(), Sampler::new(device.clone(), SamplerCreateInfo {
			mag_filter: Filter::Nearest,
			min_filter: Filter::Nearest,
			address_mode: [SamplerAddressMode::Repeat; 3],
			lod: 0.0..=0.0,
			anisotropy: match device.enabled_features().sampler_anisotropy {
				true => Some(16.0),
				false => None
			},
			mipmap_mode: SamplerMipmapMode::Nearest,
			..Default::default()
		}).unwrap());
		
		let mut orderThread = SingletonThread::newFiltered(|| {
			HTracer::threadSetName("ManagerTexture_order");
			Self::textureOrder();
		}, || -> bool {
			*Self::singleton()._haveOneOrderUpdate.read()
		});
		orderThread.setThreadName("orderThread");
		
		let updateThread = SingletonThread::new(|| {
			HTracer::threadSetName("ManagerTexture_vulkan");
			Self::updateVulkan();
		});
		
		let manager = ManagerTexture {
			_textures: DashMap::new(),
			_texturesOrder: DashMap::new(),
			_texturesCallback: DashMap::new(),
			_samplers: samplers,
			_NbTotalTexture: RwLock::new(0),
			_NbTotalLoadedTexture: RwLock::new(0),
			_descriptorSets: Default::default(),
			_threadLoading: Mutex::new(orderThread),
			_threadUpdateDescriptorSets: Mutex::new(updateThread),
			_haveOneOrderUpdate: RwLock::new(false),
		};
		
		return manager;
	}
	
	fn defaultSampler() -> Arc<Sampler>
	{
		return Sampler::new(HGEMain::singleton().getDevice().device.clone(), SamplerCreateInfo::simple_repeat_linear()).unwrap();
	}
	
	fn emptyDescriptor(set: usize) -> Arc<PersistentDescriptorSet>
	{
		let mut combuilder = HGEMain::singleton().SecondaryCmdBuffer_generate();
		
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
			vec![0,0,0,0],
		)
			.unwrap();
		
		let image = Image::new(
			ManagerMemoryAllocator::singleton().get(),
			ImageCreateInfo{
				image_type: ImageType::Dim2d,
				format: Format::R8G8B8A8_UNORM,
				extent: [1,1,1],
				usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
				..ImageCreateInfo::default()
			},
			AllocationCreateInfo::default(),
		).unwrap();
		
		combuilder
			.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
				upload_buffer,
				image.clone(),
			))
			.unwrap();
			
		let tmp = match set{ 1 | 2 => ImageView::new(image, ImageViewCreateInfo{
			view_type: ImageViewType::Dim2dArray,
			format: Format::R8G8B8A8_UNORM,
			subresource_range: ImageSubresourceRange {
				aspects: ImageAspects::COLOR,
				mip_levels: 0..1,
				array_layers: 0..1,
			},
			usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
			..ImageViewCreateInfo::default()
		}).unwrap(),
			_ => ImageView::new_default(image).unwrap()
		};
		
		HGEMain::SecondaryCmdBuffer_add(HGEMain_secondarybuffer_type::TEXTURE, combuilder.build().unwrap(), ||{});
		let returned = PersistentDescriptorSet::new(
			&HGEMain::singleton().getAllocatorSet(),
			ManagerPipeline::singleton().layoutGetDescriptor(HGE_shader_3Dsimple_holder::pipelineName(), set).unwrap(),
			[WriteDescriptorSet::image_view_sampler(
				0,
				tmp,
				Self::defaultSampler()
			)],
			[]
		).unwrap();
		
		return returned;
	}
	
	fn textureOrder()
	{
		HTracer::threadSetName("ManagerTexture");
		HTrace!("launched textureOrder");
		//HTrace!((Type::DEBUG)"ManagerTexture singleton thread start");
		let keytorun = Self::singleton()._texturesOrder.iter()
			.filter(|textureOrders|textureOrders.get().len()>0)
			.map(|textureOrders|textureOrders.key().clone()).collect::<Vec<_>>();
		
		keytorun.par_iter().cloned().for_each(|name| // par_iter
		{
			HTracer::threadSetName("ManagerTexture");
			//HTrace!((Type::DEBUG)"textureOrder run : {}",id);
			ManagerTexture::singleton().order_execute_for(name);
		});
		
		//HTrace!((Type::DEBUG)"ManagerTexture singleton thread end");
		
		// exiting but with checking if new order have been added
		let mut tmp = Self::singleton()._haveOneOrderUpdate.write();
		if(Self::singleton()._texturesOrder.iter()
			.find(|textureOrders|textureOrders.get().len()>0)
			.is_none())
		{
			*tmp = false;
		}
	}
	
	fn updateVulkan()
	{
		Self::singleton()._descriptorSets.iter().for_each(|x|{
			x.update();
		});
		
		return;
	}
	
	/*fn updateVulkan()
	{
		{
			let mut updatedbinding = Self::singleton()._haveOneTextureUpdate.write();
			*Self::singleton()._updatedDescriptor.write() = false;
			*updatedbinding = false;
		}
		
		let mut arraytex = Vec::new(); // vec must be ordered ?
		let mut atlasdata = Vec::new();
		let mut atlasnb = 0;
		
		let bindingTexture = Self::singleton()._textures.clone();
		let nbtexture = match bindingTexture.iter().map(|x| {
			*x.key()
		}).max() {
			None => {0}
			Some(max) => {max+1}
		};
		
		// other
		let defaultTexture = {
			match bindingTexture.get(&0) {
				None => return,
				Some(defaultTexture) => {
					
					if(defaultTexture.cache.is_none() && defaultTexture.contentSizeAtlas.is_none())
					{
						*Self::singleton()._updatedDescriptor.write() = true;
						return;
					}
					
					defaultTexture.value().clone()
				}
			}
		};
		
		for id in 0..nbtexture
		{
			let texture = match bindingTexture.get(&id) {
				None => defaultTexture.clone(),
				Some(x) =>	{
					x.value().clone()
				}
			};
			
			if let Some(cache) = &texture.cache {
				arraytex.push((cache.clone() as _, Self::singleton()._samplers.get(&texture.sampler).unwrap().clone()));
			}
			else if let Some(cache) = &defaultTexture.cache
			{
				arraytex.push((cache.clone() as _, Self::singleton()._samplers.get(&defaultTexture.sampler).unwrap().clone()));
			}
			
			if let Some(content) = &texture.contentSizeAtlas
			{
				atlasdata = [atlasdata.as_slice(), content.as_slice()].concat();
				atlasnb +=1;
			}
			else if let Some(content) = &defaultTexture.contentSizeAtlas
			{
				atlasdata = [atlasdata.as_slice(), content.as_slice()].concat();
				atlasnb +=1;
			}
		}
		
		if (arraytex.len() == 0 && atlasnb ==0)
		{
			*Self::singleton()._updatedDescriptor.write() = true;
			return;
		}
		
		//let test = EnginePipelines::getMaxTexture() as usize;
		if(!HGEMain::singleton().getDevice().extensionload_for13)
		{
			let mut cmdbuff = HGEMain::singleton().SecondaryCmdBuffer_generate();
			let defaultsampler = Self::singleton()._samplers.get(&defaultTexture.sampler).unwrap().clone();
			//ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
			//ImageCreateFlags::ARRAY_2D_COMPATIBLE,
			
			let atlas = ImmutableImage::from_iter(
				&ManagerMemoryAllocator::singleton().get(),
				atlasdata,
				ImageDimensions::Dim2d {
					width: 128,
					height: 128,
					array_layers: atlasnb,
				},
				MipmapsCount::Log2,
				Format::R8G8B8A8_UNORM,
				&mut cmdbuff).unwrap();
			
			
			let atlas = ImageView::new(atlas, ImageViewCreateInfo{
				view_type: ImageViewType::Dim2dArray,
				format: Some(Format::R8G8B8A8_UNORM),
				subresource_range: ImageSubresourceRange {
					aspects: ImageAspects::COLOR,
					mip_levels: 0..1,
					array_layers: 0..atlasnb -1,
				},
				usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
				..ImageViewCreateInfo::default()
			}).unwrap();
			
			let binding = PersistentDescriptorSet::new(
				&HGEMain::singleton().getAllocatorSet(),
				ManagerPipeline::singleton().layoutGetDescriptor("ManagerModels", 1).unwrap(),
				[WriteDescriptorSet::image_view_sampler(
					0,
					atlas,
					defaultsampler
				)],
			).unwrap();
			
			HGEMain::singleton().SecondaryCmdBuffer_add(HGEMain_secondarybuffer_type::TEXTURE, cmdbuff.build().unwrap(), move ||{
				Self::singleton()._persistentDescriptorSet.swap(Some(binding.clone()));
				Self::singleton().executeCallback();
				*Self::singleton()._updatedDescriptor.write() = true;
			});
			
		}
		else
		{
			let tmpSetImg = PersistentDescriptorSet::new_variable(
				&HGEMain::singleton().getAllocatorSet(),
				ManagerPipeline::singleton().layoutGetDescriptor("ManagerModels", 1).unwrap(),
				arraytex.len() as u32,
				[WriteDescriptorSet::image_view_sampler_array(
					0,
					0,
					arraytex
				)],
			);
			
			if (tmpSetImg.is_err())
			{
				*Self::singleton()._updatedDescriptor.write() = true;
				return;
			}
			
			Self::singleton()._persistentDescriptorSet.swap(Some(tmpSetImg.unwrap()));
		}
	}*/
	
	fn order_execute_for(&self, name: String)
	{
		// check Waiting
		if let Some(tmp) = self._texturesOrder.get(&name)
		{
			let mut bind = tmp.get_mut();
			for x in bind.iter_mut()
			{
				if (x.isWaiting())// we're waiting for something, so we are waiting executing these order list
				{
					return;
				}
			}
		}
		
		
		let Some((_,orders)) = self._texturesOrder.remove(&name) else {return};
		
		if let Some(texture) = self._textures.get(&name)
		{
			texture.update(|bind| {
				self.order_execute_texture_exec(&*orders.get(), bind);
			});
		}
	}
	
	fn order_execute_texture_exec(&self,orders: &Vec<Box<dyn Order + Sync + Send>>,texture: &mut Texture)
	{
		for oneorder in orders.iter()
		{
			oneorder.exec(texture);
		}
		
		self.updateTextureOnAllDescriptor(texture);
	}
	
	fn updateTextureOnAllDescriptor(&self, texture: &mut Texture)
	{
		for x in self._descriptorSets.iter()
		{
			if(x.texture_isUpdated(&*texture))
			{
				break;
			}
		};
		
		texture.clearContent();
	}
	
	fn executeCallback(&self)
	{
		self._texturesCallback.retain(|id,func|{
			if let Some(texture) = self._textures.get(id)
			{
				let mut bind = texture.get_mut();
				func(&mut bind);
			}
			false
		});
	}
	
	fn orderAdd(&self, id: String, mut orders: Vec<Box<dyn Order + Sync + Send + 'static>>)
	{
		if(orders.len()==0)
		{
			return;
		}
		
		match self._texturesOrder.get(&id) {
			None => {
				self._texturesOrder.insert(id,HArcMut::new(orders));
			}
			Some(vec) => {
				let mut bind = vec.get_mut();
				bind.append(&mut orders);
			}
		}
		*Self::singleton()._haveOneOrderUpdate.write() = true;
	}
}
