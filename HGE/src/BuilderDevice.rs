use std::sync::Arc;
use ahash::AHashMap;
use Htrace::HTrace;
use Htrace::Type::Type;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::format::Format;
use vulkano::instance::Instance;
use vulkano::memory::MemoryPropertyFlags;
use vulkano::swapchain::Surface;
use vulkano::Version;


#[derive(Clone)]
pub struct BuilderDeviceMemory
{
	pub heapIndex: usize,
	/// memory size in Mo
	pub heapSize: u64,
}

#[derive(Clone)]
pub struct BuilderDevice
{
	pub device: Arc<Device>,
	_queues: AHashMap<u32,Arc<Queue>>,
	_queueData: AHashMap<u32, [bool; 3]>,

	pub extensionload_for13: bool,
	pub maxpushconstant: u32,
	pub maxtextureshader: Option<u32>,
	pub depthformat: Format,
	pub memory: BuilderDeviceMemory
}

impl BuilderDevice
{
	pub fn newErrorLess(instance: Arc<Instance>, surface: Arc<Surface>) -> BuilderDevice
	{
		BuilderDevice::newInternal(instance, surface, DeviceExtensions {
			..DeviceExtensions::empty()
		}, Version::default())
	}
	
	pub fn new(instance: Arc<Instance>, surface: Arc<Surface>) -> BuilderDevice
	{
		// Version::HEADER_VERSION
		BuilderDevice::newInternal(instance, surface, DeviceExtensions {
			khr_swapchain: true,
			..DeviceExtensions::empty()
		}, Version::V1_1)
	}
	
	pub fn getQueueGraphic(&self) -> Arc<Queue>
	{
		return self.FindQueueForX(0);
	}
	
	pub fn getQueueTransfer(&self) -> Arc<Queue>
	{
		return self.FindQueueForX(1);
	}
		
	pub fn getQueueCompute(&self) -> Arc<Queue>
	{
		return self.FindQueueForX(2);
	}
	
	fn newInternal(instance: Arc<Instance>, surface: Arc<Surface>, device_extensions: DeviceExtensions, minversion: Version) -> BuilderDevice
	{
		match instance.enumerate_physical_devices() {
			Ok(devices) => {
				if(devices.len()==0)
				{
					HTrace!("no any vulkan devices");
					panic!("no any vulkan devices");
				}
			}
			Err(err) => {
				HTrace!("Cannot list vulkan devices : {}",err);
				panic!("Cannot list vulkan devices : {}",err);
			}
		}
		
		
		instance
			.enumerate_physical_devices()
			.unwrap().for_each(|p| {
			
			HTrace!("device: {} (type: {:?})",p.properties().device_name,p.properties().device_type);
			/*HTrace!((Htrace::Type::Type::DEBUG)"max_cull_distances: {}\n\
max_bound_descriptor_sets: {}\n\
max_descriptor_set_uniform_buffers_dynamic: {}\n\
max_framebuffer width/height/layer: {}/{}/{}\n\
max_image d1/d2/d3 - layers: {}/{}/{} - {}\n\
max_instance_count : {}\n\
max_per_set_descriptors : {}\n\
max_memory_allocation_count : {}\n\
max_memory_allocation_size : {}\n\
min_memory_map_alignment : {}\n\
max_subsampled_array_layers: {}\n\
max_image_array_layers: {}\n\
compute_units_per_shader_array: {}\n\
shader_uniform_buffer_array_non_uniform_indexing_native: {}",
				p.properties().max_cull_distances,
				p.properties().max_bound_descriptor_sets,
				p.properties().max_descriptor_set_uniform_buffers_dynamic,
				p.properties().max_framebuffer_width,
				p.properties().max_framebuffer_height,
				p.properties().max_framebuffer_layers,
				p.properties().max_image_dimension1_d,
				p.properties().max_image_dimension2_d,
				p.properties().max_image_dimension3_d,
				p.properties().max_image_array_layers,
				p.properties().max_instance_count.unwrap_or(0),
				p.properties().max_per_set_descriptors.unwrap_or(0),
				p.properties().max_memory_allocation_count,
				p.properties().max_memory_allocation_size.unwrap_or(0),
				p.properties().min_memory_map_alignment,
				p.properties().max_subsampled_array_layers.unwrap_or(0),
				p.properties().max_image_array_layers,
				p.properties().compute_units_per_shader_array.unwrap_or(0),
				p.properties().shader_uniform_buffer_array_non_uniform_indexing_native.unwrap_or(false)
			);*/
		});
		
		
		let (physical_device, _) = instance
			.enumerate_physical_devices()
			.unwrap()
			.filter(|p| {
				HTrace!((Type::DEBUG) "comparing vulkan version : {}.{} on {}.{}",p.api_version().major,p.api_version().minor,minversion.major,minversion.minor);
				(p.api_version().major == minversion.major && p.api_version().minor >= minversion.minor) || p.api_version().major > minversion.major
			})
			.filter(|p| {
				HTrace!((Type::DEBUG) "support extension : {:?}",p.supported_extensions());
				HTrace!((Type::DEBUG) "check extension : {:?}",&device_extensions);
				p.supported_extensions().contains(&device_extensions)
			})
			.filter_map(|p| {
				p.queue_family_properties()
					.iter()
					.enumerate()
					.position(|(i, q)| {
						q.queue_flags.intersects(QueueFlags::GRAPHICS)
							&& p.surface_support(i as u32, &surface).unwrap_or(false)
					})
					.map(|i| (p, i as u32))
			})
			.min_by_key(|(p, _)| match p.properties().device_type {
				PhysicalDeviceType::DiscreteGpu => 0,
				PhysicalDeviceType::IntegratedGpu => 1,
				PhysicalDeviceType::VirtualGpu => 2,
				PhysicalDeviceType::Cpu => 3,
				PhysicalDeviceType::Other => 4,
				_ => 5,
			})
			.expect("No suitable physical device found");
		
		// Some little debug infos.
		HTrace!(
			"Using device: {} (type: {:?})",
			physical_device.properties().device_name,
			physical_device.properties().device_type
		);
		
		let mut QueueData = AHashMap::new();
		let mut QueueNb = 0;
		let mut QueueVec = Vec::new();
		for x in physical_device.queue_family_properties()
		{
			QueueData.insert(QueueNb,[x.queue_flags.intersects(QueueFlags::GRAPHICS),x.queue_flags.intersects(QueueFlags::TRANSFER),x.queue_flags.intersects(QueueFlags::COMPUTE)]);
			QueueVec.push(QueueCreateInfo {
				queue_family_index: QueueNb,  // first is zero AND normaly a GPU have one queue minimal
				queues: vec![1.0],
				..Default::default()
			});
			QueueNb+=1;
		}
		
		let pushconstantsize =	physical_device.properties().max_push_constants_size;
		//let mut can13 = physical_device.api_version() >= Version::V1_3;
		let can13 = false; // need to false if VULKAN11COMP in shader define.glsl is set
		let mut feature = Features { // pc default
			sampler_anisotropy: true,
			//dynamic_rendering: true,
			//fill_mode_non_solid: true,
			//descriptor_indexing: true,
			//shader_uniform_buffer_array_non_uniform_indexing: true, // permet un tableau de texture
			runtime_descriptor_array: true,
			descriptor_binding_variable_descriptor_count: true,
			//extended_dynamic_state: true,
			..Features::empty()
		};
		if(!can13) // android
		{
			feature = Features {
				//sampler_anisotropy: true,
				//dynamic_rendering: true,
				//fill_mode_non_solid: true,
				//descriptor_indexing: true,
				//shader_uniform_buffer_array_non_uniform_indexing: true, // permet un tableau de texture
				//runtime_descriptor_array: true,
				//descriptor_binding_variable_descriptor_count: true,
				//extended_dynamic_state: true,
				..Features::empty()
			};
		}
		
		// Now initializing the device. This is probably the most important object of Vulkan.
		//
		// The iterator of created queues is returned by the function alongside the device.
		let (device, queues) = Device::new(
			// Which physical device to connect to.
			physical_device,
			DeviceCreateInfo {
				// A list of optional features and extensions that our program needs to work correctly.
				// Some parts of the Vulkan specs are optional and must be enabled manually at device
				// creation. In this example the only thing we are going to need is the `khr_swapchain`
				// extension that allows us to draw to a window.
				enabled_extensions: device_extensions,
				
				enabled_features: feature,
				
				// The list of queues that we are going to use. Here we only use one queue, from the
				// previously chosen queue family.
				queue_create_infos: QueueVec,
				
				..Default::default()
			},
		).unwrap();
		
		let mut tmpqueues = AHashMap::new();
		for x in queues {
			tmpqueues.insert(x.queue_family_index(),x);
		}
		
		let mut memorySize = 0;
		let deviceIndex = device.physical_device().memory_properties().memory_types.iter()
			.filter(|x|x.property_flags.intersects(MemoryPropertyFlags::DEVICE_LOCAL))
			.next().map(|x|x.heap_index).unwrap_or(0) as usize;
		device.physical_device().memory_properties().memory_heaps.iter().for_each(|x|{println!("tst {} {:?}",x.size as u64,x.flags)});
		if let Some(gpuram) = device.physical_device().memory_properties().memory_heaps.get(deviceIndex)
		{
			memorySize = ((gpuram.size as u64)/1024)/1024;
		}
		println!("device max memory size : {} {:.2}",memorySize,((memorySize as f64) / 1024.0f64) / 1024.0f64);
		
		let mut format = Format::D16_UNORM;
		if let Ok(_) = device.physical_device().format_properties(Format::D32_SFLOAT)
		{
			format = Format::D32_SFLOAT
		}
		HTrace!("depth format : {:?}",format);
		
		BuilderDevice {
			device: device,
			_queues: tmpqueues,
			_queueData: QueueData,
			extensionload_for13: can13,
			maxpushconstant: pushconstantsize,
			maxtextureshader: Some(128),
			depthformat: format,
			memory: BuilderDeviceMemory { heapIndex: deviceIndex, heapSize: memorySize },
		}
	}
	
	pub fn getQueueSharing(&self) -> Vec<u32>
	{
		let tmp = self._queues.iter().map(|(_,x)|{x.queue_family_index()}).collect();
		return tmp;
	}
	
	/////// PRIVATE
	
	fn FindQueueForX(&self, ElementToFind: usize) -> Arc<Queue>
	{
		let mut SecondaryElement = 1;
		let mut ThirdElement = 2;
		if(ElementToFind==1)
		{
			SecondaryElement = 0;
			ThirdElement = 2;
		}
		if(ElementToFind==2)
		{
			SecondaryElement = 1;
			ThirdElement = 0;
		}
		
		let mut found = self._queues.get(&0);
		
		// search specialized queue
		for (key,queue) in self._queueData.iter()
		{
			if(*key==0 || !queue[ElementToFind])
			{
				continue;
			}
				if(!queue[SecondaryElement] && !queue[ThirdElement]) // specialised queue, so we leave directly
				{
					found = self._queues.get(key);
					break;
				}
				if(!queue[SecondaryElement] || !queue[ThirdElement])
				{
					found = self._queues.get(key);
				}
			
		}
		
		if(found.is_none())
		{
			return self._queues.get(&0).unwrap().clone();
		}
		
		return found.unwrap().clone();
	}
	
}
