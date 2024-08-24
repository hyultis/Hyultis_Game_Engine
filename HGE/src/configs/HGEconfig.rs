use std::sync::{Arc, OnceLock};
use arc_swap::{ArcSwap, Guard};
use Hconfig::HConfigManager::HConfigManager;
use Hconfig::serde_json::Value as JsonValue;
use Htrace::HTrace;
use vulkano::swapchain::PresentMode;
use crate::configs::general::HGEconfig_general;
use crate::configs::system_swapchain::HGEconfig_system_swapchain;

pub struct HGEconfig
{
	_system_video: ArcSwap<HGEconfig_system_swapchain>,
	_generalConfig: ArcSwap<HGEconfig_general>
}

static SINGLETON: OnceLock<HGEconfig> = OnceLock::new();

impl HGEconfig
{
	pub fn defineGeneral(general: HGEconfig_general)
	{
		if(SINGLETON.get().is_some())
		{
			return;
		}
		
		let _ = SINGLETON.set(Self{
			_system_video: Default::default(),
			_generalConfig: ArcSwap::new(Arc::new(general)),
		});
	}
	
	pub fn singleton() -> &'static Self
	{
		match SINGLETON.get() {
			Some(val) => val,
			_ => panic!("need to \"defineGeneric\" before anything else")
		}
	}
	
	pub fn loadSwapchainFromConfig(&self, defaultPresentmode: PresentMode, defaultFpslimiter: u8)
	{
		let mut config = HConfigManager::singleton().get(self._generalConfig.load().configName.clone());
		let mut updatedconfig = *self._system_video.load_full().clone();
		
		updatedconfig.setPresentModeString(config.getOrSetDefault("system/swapchain/presentmode", JsonValue::String(match defaultPresentmode {
			PresentMode::Immediate => "Immediate",
			PresentMode::Mailbox => "Mailbox",
			_ => "Fifo"
		}.to_string())).as_str().unwrap_or("Fifo").to_string());
		updatedconfig.fpslimiter = config.getOrSetDefault("system/swapchain/fpslimiter", JsonValue::from(defaultFpslimiter)).as_u64().unwrap_or(0) as u8;
		
		HTrace!("loaded swapchain config : {:?}",updatedconfig);
		
		let _ =	config.save();
		
		self._system_video.swap(Arc::new(updatedconfig));
	}
	
	pub fn saveSwapchainFromConfig(&self)
	{
		let mut config = HConfigManager::singleton().get(self._generalConfig.load().configName.clone());
		let updatedconfig = *self._system_video.load_full().clone();
		
		config.set("system/swapchain/presentmode", updatedconfig.getPresentModeString());
		config.set("system/swapchain/fpslimiter", updatedconfig.fpslimiter);
		
		let _ =	config.save();
		
		self._system_video.swap(Arc::new(updatedconfig));
	}
	
	pub fn swapchain_get(&self) -> Guard<Arc<HGEconfig_system_swapchain>>
	{
		self._system_video.load()
	}
	
	pub fn swapchain_set(&self,update: HGEconfig_system_swapchain)
	{
		self._system_video.swap(Arc::new(update));
	}
	
	pub fn general_get(&self) -> Guard<Arc<HGEconfig_general>>
	{
		self._generalConfig.load()
	}
	
}
