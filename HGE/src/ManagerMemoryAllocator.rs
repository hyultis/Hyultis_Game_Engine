use std::sync::{Arc, OnceLock};
use arc_swap::ArcSwapOption;
use vulkano::device::Device;
use vulkano::memory::allocator::StandardMemoryAllocator;

// unsafe to use between thread !
pub struct ManagerMemoryAllocator
{
	_stdmemalls: ArcSwapOption<StandardMemoryAllocator>
}

static SINGLETON: OnceLock<ManagerMemoryAllocator> = OnceLock::new();

impl ManagerMemoryAllocator
{
	fn new() -> ManagerMemoryAllocator {
		return ManagerMemoryAllocator {
			_stdmemalls: ArcSwapOption::default()
		};
	}
	
	pub fn singleton() -> &'static ManagerMemoryAllocator
	{
		return SINGLETON.get_or_init(|| {
			ManagerMemoryAllocator::new()
		});
	}
	
	pub fn update(&self, device: Arc<Device>)
	{
		let memoryalloc = StandardMemoryAllocator::new_default(device.clone());
		let stdmemall = Arc::new(memoryalloc);
		self._stdmemalls.swap(Some(stdmemall));
	}
	
	pub fn get(&self) -> Arc<StandardMemoryAllocator>
	{
		return self._stdmemalls.load().clone().unwrap();
	}
}
