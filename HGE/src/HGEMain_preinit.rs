use crate::configs::general::HGEconfig_general;
use crate::configs::HGEconfig::HGEconfig;
use anyhow::anyhow;
use state_shift::{impl_state, type_state};
use std::sync::Arc;
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions};
use vulkano::VulkanLibrary;
use Htrace::HTrace;
use Htrace::Type::Type;

#[type_state(
    states = (Initial, Configured, Ready), // defines the available states
    slots = (Initial) // defines how many concurrent states will be there, and the initial values for these states
)]
pub struct HGEMain_preinitState
{
	_instance: Option<Arc<Instance>>,
}

#[impl_state]
impl HGEMain_preinitState
{
	#[require(Initial)]
	pub(crate) fn new() -> HGEMain_preinitState
	{
		return HGEMain_preinitState {
			_instance: None,
			_state: Default::default(),
		};
	}

	/// first step
	/// set config file for HGE
	#[require(Initial)]
	#[switch_to(Configured)]
	pub fn setConfig(
		self,
		config: HGEconfig_general,
	) -> anyhow::Result<HGEMain_preinitState<Configured>>
	{
		if (config.defaultShaderLoader.is_none())
		{
			HTrace!((Type::ERROR) "general configuration for loading shader is empty in \"defaultShaderLoader\"");
			return Err(anyhow!(
				"general configuration for loading shader is empty in \"defaultShaderLoader\""
			));
		}

		HGEconfig::defineGeneral(config);
		return Ok(HGEMain_preinitState {
			_instance: None,
			_state: Default::default(),
		});
	}

	/// second step
	/// set config file for HGE
	#[require(Configured)]
	#[switch_to(Ready)]
	pub fn setInstance(
		self,
		required_extensions: InstanceExtensions,
	) -> anyhow::Result<HGEMain_preinitState<Ready>>
	{
		HTrace!("Engine pre-initialization: creating vulkan instance");
		let instance = Self::Init_Instance(required_extensions)?;

		Ok(HGEMain_preinitState {
			_instance: Some(instance),
		})
	}

	#[require(Ready)]
	pub fn getInstance(&self) -> Arc<Instance>
	{
		return self._instance.as_ref().unwrap().clone();
	}

	#[require(A)]
	fn Init_Instance(mut required_extensions: InstanceExtensions) -> anyhow::Result<Arc<Instance>>
	{
		let library = VulkanLibrary::new().unwrap();
		let mut debuglayer = Vec::new();
		let config = HGEconfig::singleton().general_get();

		if (cfg!(feature = "debuglayer"))
		{
			debuglayer.push("VK_LAYER_KHRONOS_validation".to_string());
		}

		HTrace!("List of instance extension {:?}", required_extensions);

		required_extensions.ext_surface_maintenance1 = true;

		// Now creating the instance.
		let instance = Instance::new(
			library,
			InstanceCreateInfo {
				application_name: Some(config.windowTitle.clone()),
				application_version: config.appVersion,
				engine_name: Some(crate::HGEMain::HGE_STRING.to_string()),
				engine_version: crate::HGEMain::HGE_VERSION,
				enabled_layers: debuglayer,
				enabled_extensions: required_extensions,
				// Enable enumerating devices that use non-conformant vulkan implementations. (ex. MoltenVK)
				flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
				..Default::default()
			},
		)
		.map_err(|e| anyhow!("HGE cannot initialize instance because : {:?}", e))?;

		Ok(instance)
	}
}
