use std::sync::Arc;

#[derive(Clone)]
pub struct HGEconfig_general
{
	pub startFullscreen: bool,
	pub windowTitle: String,
	/// set true if the running device is steamdeck
	/// force windows creation to Fullscreen::Exclusive
	pub isSteamdeck: bool,
	/// set true if the running device is android
	pub isAndroid: bool,
	pub defaultShaderLoader: Option<Arc<dyn Fn() + Sync + Send>>
}

impl Default for HGEconfig_general
{
	fn default() -> Self {
		Self{
			startFullscreen: true,
			windowTitle: "HGE default title".to_string(),
			isSteamdeck: false,
			isAndroid: false,
			defaultShaderLoader: None,
		}
	}
}
