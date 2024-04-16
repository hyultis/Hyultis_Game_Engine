use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};
use std::vec;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use dashmap::try_result::TryResult;
use Htrace::HTrace;
use Htrace::HTracer::HTracer;
use Htrace::Type::Type;
use parking_lot::{Mutex, RwLock};
use rayon::iter::IntoParallelRefIterator;
use vulkano::command_buffer::{CopyBufferToImageInfo, SecondaryAutoCommandBuffer};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::image::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode};
use crate::HGEMain::{HGEMain, HGEMain_secondarybuffer_type};
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Textures::Textures::{Texture, Texture_atlasType, Texture_part, TextureState, TextureStateGPU};
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
use crate::Textures::Orders::Order_computeCache::Order_computeCache;
use rayon::iter::ParallelIterator;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use crate::Shaders::HGE_shader_3Dsimple::HGE_shader_3Dsimple_holder;
use crate::Shaders::ShaderStruct::ShaderStructHolder;
use HArcMut::HArcMut;

pub struct ManagerTexture
{
	_textures: DashMap<u32, HArcMut<Texture>>,
	_texturesToId: DashMap<String, u32>,
	_texturesOrder: DashMap<u32, HArcMut<Vec<Box<dyn Order + Send + Sync>>>>,
	_texturesCallback: DashMap<u32, Box<dyn Fn(&mut Texture) + Send + Sync + 'static>>,
	_samplers: DashMap<String, Arc<Sampler>>,
	_persistentDescriptorSet: DashMap<Texture_atlasType, ArcSwap<PersistentDescriptorSet>>,
	_commandBufferWaiting: DashMap<u32, Vec<SecondaryAutoCommandBuffer>>,
	_NbTotalTexture: RwLock<u32>,
	_NbTotalLoadedTexture: RwLock<u32>,
	
	_atlasSizeId: DashMap<Texture_atlasType,ArcSwap<u32>>,
	
	// thread stuff
	_threadLoading: Mutex<SingletonThread>,
	_threadUpdateVulkan: Mutex<SingletonThread>,
	_haveOneOrderUpdate: RwLock<bool>,
	_haveOneTextureUpdate: RwLock<bool>,
	_updatedDescriptor: RwLock<bool>,
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
		let (id, found) = self.getIdOrReturnMax(name.clone());
		let sampler = sampler.unwrap_or("default").to_string();
		HTrace!("ManagerTexture: load {} with id {} and sampler {}",name,id,sampler);
		
		self._textures.insert(id, HArcMut::new(Texture {
			name: name.clone(),
			sampler: sampler,
			..Texture::default()
		}));
		self._texturesOrder.insert(id,HArcMut::new(Vec::new()));
		if (!found)
		{
			self._texturesToId.insert(name, id);
			*self._NbTotalTexture.write() += 1;
		}
		
		if(loadOrder.isSameThread())
		{
			if let Some(texture ) = self._textures.get(&id)
			{
				let mut texturemut = texture.get_mut();
				loadOrder.exec(id, &mut texturemut);
				Order_computeCache::new().exec(id, &mut texturemut);
				if(texturemut.atlasType!=Texture_atlasType::FONT)
				{
					*self._haveOneTextureUpdate.write() = true;
				}
			}
		}
		else
		{
			self.orderAdd(id,vec![Box::new(loadOrder)]);
			self.orderAdd(id,vec![Box::new(Order_computeCache::new())]);
		}
	}
	
	pub fn texture_update(&self, name: impl Into<String>, mut updateOrders: Vec<Box<dyn Order + Sync + Send + 'static>>)
	{
		let name = name.into();
		if let Some(bind) = self.getMut(name.clone())
		{
			let mut texture = bind.get_mut();
			//HTrace!("ManagerTexture: texture_update for {}",name);
			updateOrders.retain(|sameThreadOrder|{
				if (sameThreadOrder.isSameThread())
				{
					sameThreadOrder.exec(*bind.key(), &mut texture);
					
					if(texture.atlasType!=Texture_atlasType::FONT)
					{
						*self._haveOneTextureUpdate.write() = true;
					}
					
					return false;
				}
				else
				{
					return true;
				}
			});
			
			self.orderAdd(*bind.key(),updateOrders);
		}
	}
	
	pub fn textureDebug(&self)
	{
		let mut tmp = BTreeMap::new();
		for x in self._textures.iter()
		{
			let bind = x.get();
			HTrace!("textureDebug {} : state {:?} - sendToGpu {:?} - atlas Type {:?} - atlas id {:?}",bind.name,bind.state,bind.sendToGpu,bind.atlasType,bind.atlasId);
			let debug = format!("{} => {:?}",bind.name,bind.atlasId);
			match tmp.get_mut(&bind.atlasType) {
				None => {tmp.insert(bind.atlasType,vec![debug]);},
				Some(vec) => {
					vec.push(debug)
				}
			}
		}
		println!("\n\n");
		
		for (typea, vec) in tmp {
			println!("\n\nfor atlas : {:?}",typea);
			for x in vec {
				
				println!("  - {}",x);
			}
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
				self.orderAdd(*x.key(),vec![Box::new(reloader.clone())]);
				self.orderAdd(*x.key(),vec![Box::new(Order_computeCache::new())]);
			}
		}
	}
	
	pub fn texture_reload(&self, name: impl Into<String>)
	{
		match self.getMut(name) {
			None => {}
			Some(x) => {
				if let Some(reloader) = &x.get().reloadLoader
				{
					self.orderAdd(*x.key(),vec![Box::new(reloader.clone())]);
					self.orderAdd(*x.key(),vec![Box::new(Order_computeCache::new())]);
				}
			}
		}
	
	}
	
	pub fn texture_setCallback(&self, name: impl Into<String>, func: impl Fn(&mut Texture) + Send + Sync + 'static)
	{
		if let Some(texture) = self.getMut(name.into())
		{
			self._texturesCallback.insert(*texture.key(),Box::new(func));
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
	
	pub fn get(&self, name: impl Into<String>) -> Option<Texture>
	{
		let textureid = {
			let returning = match self._texturesToId.get(&name.into()) {
				None => return None,
				Some(textureid) => textureid.value().clone()
			};returning
		};
		
		return match self._textures.get(&textureid) {
			None => None,
			Some(x) => {
				Some((&**x.get()).clone())
			}
		};
	}
	
	pub fn getState(&self, name: impl Into<String>) -> Option<TextureState>
	{
		let name : String = name.into();
		let textureid = {
			let returning = match self._texturesToId.get(&name) {
				Some(textureid) => textureid.value().clone(),
				None => {
					return None
				}
			};
			returning
		};
		println!("have textureid : {} {}",name,textureid);
		
		return self._textures.get(&textureid).map(|x|{x.get().state.clone()});
	}
	
	pub fn texture_loadPart(&self, name: &str, partLoader: impl texturePart + Sync + Send + 'static)
	{
		let id = match self._texturesToId.get(name) {
			None => {
				HTrace!("TEXTURE texture_loadPart NONE");
				return;
			}
			Some(id) => {
				id.value().clone()
			}
		};
		
		self.orderAdd(id,vec![Box::new(Order_loadPart{
			from: Box::new(partLoader),
		})]);
	}
	
	pub fn getPart(&self, name: impl Into<String>, part: impl Into<String>) -> Option<Texture_part>
	{
		let texture = self.getMut(name);
		match texture {
			None => None,
			Some(x) => {x.get().partUVCoord.get(&part.into()).copied()}
		}
	}
	
	// returned value is +1 ... the shader use 0 as "no color"
	pub fn getTextureToId(&self, name: impl Into<String>) -> Option<u32>
	{
		let name = name.into();
		match self.get(&name) {
			None => None,
			Some(tex) => {
				match tex.atlasType {
					Texture_atlasType::NONE => None,
					Texture_atlasType::SMALL => Some(tex.atlasId.unwrap_or(0)+1),
					Texture_atlasType::LARGE =>	Some(tex.atlasId.unwrap_or(0)+200+1),
					Texture_atlasType::FONT => Some(9999+1)
				}
			}
		}
	}
	
	pub fn launchThreads(&self)
	{
		self._threadLoading.lock().thread_launch();
		self._threadUpdateVulkan.lock().thread_launch();
		//return;
	}
	
	pub fn getPersistentDescriptorSet(&self) -> &DashMap<Texture_atlasType, ArcSwap<PersistentDescriptorSet>>
	{
		return &self._persistentDescriptorSet;
	}
	
	pub fn getNbTexture(&self) -> u32
	{
		return *self._NbTotalTexture.read();
	}
	
	pub fn getNbLoadedTexture(&self) -> u32
	{
		return *self._NbTotalLoadedTexture.read();
	}
	
	pub fn forceReload(&self)
	{
		*self._haveOneTextureUpdate.write() = true;
		//*self._updatedDescriptor.write() = true;
		Self::updateVulkan();
	}
	
	pub fn textureSetSendToGpu(&self, id: u32)
	{
		match self._textures.get(&id) {
			None => {}
			Some(texture) => {
				let mut bind = texture.get_mut();
				bind.sendToGpu = TextureStateGPU::SEND;
				bind.clearContent();
			}
		}
	}
	
	pub fn getNewAtlasId(&self, atlastype: Texture_atlasType) -> u32
	{
		match self._atlasSizeId.get(&atlastype) {
			None => {
				self._atlasSizeId.insert(atlastype,ArcSwap::new(Arc::new(0)));
				0
			}
			Some(id) => {
				let value = **id.load()+1;
				id.swap(Arc::new(value));
				value
			}
		}
	}
	
	pub fn fontDescriptorUpdate(&self, newdescriptor: Arc<PersistentDescriptorSet>)
	{
		if let Some(descriptor) = self._persistentDescriptorSet.get(&Texture_atlasType::FONT)
		{
			descriptor.swap(newdescriptor);
		}
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
		
		let mapToId = DashMap::new();
		//mapToId.insert("default".to_string(), 0);
		
		let persistentDescriptorDefault = DashMap::new();
		for x in [Texture_atlasType::SMALL,Texture_atlasType::LARGE,Texture_atlasType::FONT]
		{
			persistentDescriptorDefault.insert(x,ArcSwap::new(Self::emptyDescriptor(x.getSetId())));
		}
		
		let mut orderThread = SingletonThread::newFiltered(|| {
			HTracer::threadSetName("ManagerTexture_order");
			Self::textureOrder();
		}, || -> bool {
			*Self::singleton()._haveOneOrderUpdate.read()
		});
		orderThread.setThreadName("orderThread");
		
		let updateThread = SingletonThread::newFiltered(|| {
			HTracer::threadSetName("ManagerTexture_vulkan");
			Self::updateVulkan();
		}, ||{
			*Self::singleton()._haveOneTextureUpdate.read() && *Self::singleton()._updatedDescriptor.read()
		});
		
		let manager = ManagerTexture {
			_textures: DashMap::new(),
			_texturesToId: mapToId,
			_texturesOrder: DashMap::new(),
			_texturesCallback: DashMap::new(),
			_samplers: samplers,
			_persistentDescriptorSet: persistentDescriptorDefault,
			_commandBufferWaiting: DashMap::new(),
			_NbTotalTexture: RwLock::new(0),
			_NbTotalLoadedTexture: RwLock::new(0),
			_atlasSizeId: DashMap::new(),
			_threadLoading: Mutex::new(orderThread),
			_threadUpdateVulkan: Mutex::new(updateThread),
			_haveOneOrderUpdate: RwLock::new(false),
			_haveOneTextureUpdate: RwLock::new(false),
			_updatedDescriptor: RwLock::new(true),
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
		
		keytorun.par_iter().copied().for_each(|id| // par_iter
		{
			HTracer::threadSetName("ManagerTexture");
			//HTrace!((Type::DEBUG)"textureOrder run : {}",id);
			ManagerTexture::singleton().order_execute_for(id);
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
		{
			let mut updatedbinding = Self::singleton()._haveOneTextureUpdate.write();
			*Self::singleton()._updatedDescriptor.write() = false;
			*updatedbinding = false;
		}
		
		
		// other
		let defaultTexture = {
			match Self::singleton()._textures.get(&0) {
				None => {
					*Self::singleton()._updatedDescriptor.write() = true;
					*Self::singleton()._haveOneTextureUpdate.write() = true;
					return;
				},
				Some(defaultTexture) => {
					let bind = defaultTexture.get();
					if(/*defaultTexture.cache.is_none() && */bind.contentSizeAtlas.is_none())
					{
						*Self::singleton()._updatedDescriptor.write() = true;
						*Self::singleton()._haveOneTextureUpdate.write() = true;
						return;
					}
					
					(&**bind).clone()
				}
			}
		};
		let defaulttextureslice = defaultTexture.contentSizeAtlas.unwrap();
		
		let mut emptyrun = true;
		let mut idTextureUsed = Vec::new();
		let mut descriptorlist = BTreeMap::new();
		let mut cmdbuff = HGEMain::singleton().SecondaryCmdBuffer_generate();
		for altastype in [Texture_atlasType::SMALL, Texture_atlasType::LARGE]
		{
			let mut atlasnb = 0;
			let mut finalDataArray = BTreeMap::new();
			Self::singleton()._textures
				.iter()
				.filter(|tex|{tex.get().atlasType==altastype})
				.for_each(|tex|{
					let bind = tex.get();
					if let Some(atlasid) = bind.atlasId
					{
						idTextureUsed.push(*tex.key());
						atlasnb+=1;
						finalDataArray.insert(atlasid,match &bind.contentSizeAtlas {
							None => defaulttextureslice.clone(), Some(content) => content.clone()
						});
					}
					
				});
			
			let finalData: Vec<u8> = finalDataArray.into_iter().flat_map(|(_,data)|{data}).collect();
			//println!("vulkan mem len : {}",finalData.len());
			HTrace!((Type::DEBUG) "updatevulkan : {:?} len {} {}",altastype,altastype.getSize(),atlasnb);
			
			if(atlasnb==0)
			{
				continue;
			}
			
		
			let defaultsampler = Self::singleton()._samplers.get(&defaultTexture.sampler).unwrap().clone();
			
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
				finalData,
			)
				.unwrap();
			
			//println!("updateVulkan atlas: {}",atlasnb);
			let atlas = Image::new(
				ManagerMemoryAllocator::singleton().get(),
				ImageCreateInfo{
					image_type: ImageType::Dim2d,
					format: Format::R8G8B8A8_UNORM,
					extent: [altastype.getSize(),altastype.getSize(),1],
					array_layers: atlasnb,
					usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
					..ImageCreateInfo::default()
				},
				AllocationCreateInfo::default(),
			).unwrap();
			
			if cmdbuff
				.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
					upload_buffer,
					atlas.clone(),
				)).is_err()
			{
				continue;
			}
			
			let atlas = ImageView::new(atlas, ImageViewCreateInfo{
				view_type: ImageViewType::Dim2dArray,
				format: Format::R8G8B8A8_UNORM,
				subresource_range: ImageSubresourceRange {
					aspects: ImageAspects::COLOR,
					mip_levels: 0..1,
					array_layers: 0..atlasnb,
				},
				usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
				..ImageViewCreateInfo::default()
			}).unwrap();
			
			let mut set = 1;
			if altastype == Texture_atlasType::LARGE
			{
				set = 2;
			}
			
			descriptorlist.insert(altastype, PersistentDescriptorSet::new(
				&HGEMain::singleton().getAllocatorSet(),
				ManagerPipeline::singleton().layoutGetDescriptor(HGE_shader_3Dsimple_holder::pipelineName(), set).unwrap(),
				[WriteDescriptorSet::image_view_sampler(
					0,
					atlas,
					defaultsampler
				)],
				[]
			).unwrap());
			
			emptyrun=false;
		}
		
		if(emptyrun)
		{
			*Self::singleton()._updatedDescriptor.write() = true;
			*Self::singleton()._haveOneTextureUpdate.write() = true;
			return;
		}
		
		
		let newnb = Self::singleton()._textures
			.iter()
			.filter(|tex|{tex.get().state!=TextureState::CREATED}).count() as u32; // texture font statut is ignored
		
		// TODO FIX DISORDER
		HGEMain::SecondaryCmdBuffer_add(HGEMain_secondarybuffer_type::TEXTURE, cmdbuff.build().unwrap(), move ||{
			descriptorlist.iter().for_each(|(a,b)|{
				Self::singleton()._persistentDescriptorSet.insert(a.clone(),ArcSwap::new(b.clone()));
			});
			Self::singleton().executeCallback();
			*Self::singleton()._updatedDescriptor.write() = true;
			let mut tmp = Self::singleton()._NbTotalLoadedTexture.write();
			if(*tmp<newnb)
			{
				*tmp = newnb;
			}
		});
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
	
	fn order_execute_for(&self, id: u32)
	{
		// check Waiting
		if let Some(tmp) = self._texturesOrder.get(&id)
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
		
		
		let Some((_,orders)) = self._texturesOrder.remove(&id) else {return};
		
		if let Some(texture) = self._textures.get(&id)
		{
			let mut bind = texture.get_mut();
			for oneorder in orders.get().iter()
			{
				oneorder.exec(id, &mut bind);
			}
			if(bind.atlasType!=Texture_atlasType::FONT)
			{
				*self._haveOneTextureUpdate.write() = true;
			}
		}
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
	
	fn getIdOrReturnMax(&self, name: impl Into<String>) -> (u32, bool)
	{
		let name: String = name.into();
		match self._texturesToId.get(&name) {
			Some(id) => return (*id.value(), true),
			None => {
				if (name == "default")
				{
					return (0, false);
				}
				
				match self._textures.iter().max_by_key(|a| a.key().clone())
					.map(|x| x.key().clone()) {
					None => return (1, false), // 0 is reserved
					Some(max) => return (max + 1, false) // because 0 is reserved, we move all by +1
				}
			}
		}
	}
	
	fn internal_getStateId(&self, id: &u32) -> Option<TextureState>
	{
		match ManagerTexture::singleton()._textures.try_get(&id) {
			TryResult::Present(x) => return Some(x.get().state.clone()),
			TryResult::Absent => return None,
			TryResult::Locked => return None,
		}
	}
	
	fn getMut(&self, name: impl Into<String>) -> Option<Ref<u32, HArcMut<Texture>>>
	{
		let textureid = {
			let returning = match self._texturesToId.get(&name.into()) {
				None => return None,
				Some(textureid) => textureid.value().clone()
			};returning
		};
		
		return self._textures.get(&textureid);
	}
	
	fn orderAdd(&self, id: u32, mut orders: Vec<Box<dyn Order + Sync + Send + 'static>>)
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
