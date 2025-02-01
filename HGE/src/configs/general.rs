use std::sync::Arc;
use vulkano::Version;

#[derive(Clone)]
pub struct HGEconfig_general_font
{
	// font used to draw text to user (in is language) - path relative to static
	pub path_fileUser: String,
	// font used to draw universal text (like number, symbol, etc) - path relative to static
	pub path_fileUniversel: String,
	// font used to draw bold stuff - path relative to static
	pub path_fileBold: String,
}

#[derive(Clone)]
pub struct HGEconfig_general
{
	pub startFullscreen: bool,
	pub windowTitle: String,
	pub appVersion: Version,
	/// namefile of the config inside the config directory, without ".json"
	pub configName: String,
	/// set true if the running device is steamdeck
	/// force windows creation to Fullscreen::Exclusive
	pub isSteamdeck: bool,
	/// set true if the running device is android
	pub isAndroid: bool,
	pub defaultShaderLoader: Option<Arc<dyn Fn() + Sync + Send>>,
	pub fonts: HGEconfig_general_font,
	pub debug_showTimer: bool,
}

impl Default for HGEconfig_general
{
	fn default() -> Self
	{
		Self {
			startFullscreen: true,
			windowTitle: "HGE default title".to_string(),
			appVersion: Version { major: 0, minor: 0, patch: 0 },
			configName: "HGE".to_string(),
			isSteamdeck: false,
			isAndroid: false,
			defaultShaderLoader: None,
			fonts: HGEconfig_general_font {
				path_fileUser: "".to_string(),
				path_fileUniversel: "".to_string(),
				path_fileBold: "".to_string(),
			},
			debug_showTimer: false,
		}
	}
}
