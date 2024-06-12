use std::env;
use std::sync::OnceLock;

pub struct Paths_define
{
	_base: Option<String>,
	_staticName: Option<String>,
	_dynamicName: Option<String>,
	_configName: Option<String>,
	_usersaveName: Option<String>
}

impl Default for Paths_define
{
	fn default() -> Self {
		Self{
			_base: None,
			_staticName: None,
			_dynamicName: None,
			_configName: None,
			_usersaveName: None,
		}
	}
}

pub struct Paths
{
	_base: String,
	_staticName: String,
	_dynamicName: String,
	_configName: String,
	_usersaveName: String
}

static SINGLETON: OnceLock<Paths> = OnceLock::new();

impl Paths
{
	fn new() -> Paths
	{
		return Default::default();
	}
	
	pub fn singleton() -> &'static Paths
	{
		return SINGLETON.get_or_init(|| {
			let mut tmp = Paths::new();
			tmp.computePaths();
			tmp
		});
	}
	
	pub fn define(definer: Paths_define)
	{
		SINGLETON.get_or_init(|| {
			let mut tmp = Paths::new();
			if let Some(val) = definer._base
			{
				tmp._base = val;
			}
			if let Some(val) = definer._staticName
			{
				tmp._staticName = val;
			}
			if let Some(val) = definer._dynamicName
			{
				tmp._dynamicName = val;
			}
			if let Some(val) = definer._configName
			{
				tmp._configName = val;
			}
			if let Some(val) = definer._usersaveName
			{
				tmp._usersaveName = val;
			}
			tmp.computePaths();
			tmp
		});
	}
	
	pub fn getStatic(&self) -> String
	{
		return self._staticName.clone();
	}
	
	pub fn getDynamic(&self) -> String
	{
		return self._dynamicName.clone();
	}
	
	pub fn getConfig(&self) -> String
	{
		return self._configName.clone();
	}
	
	pub fn getSave(&self) -> String
	{
		return self._usersaveName.clone();
	}
	
	pub fn getBase(&self) -> String
	{
		return self._base.clone();
	}
	
	pub fn getExec() -> String
	{
		return env::current_exe().unwrap().display().to_string();
	}
	
	fn computePaths(&mut self)
	{
		self._configName = self._configName.replace("{base}",&self._base);
		self._staticName = self._staticName.replace("{base}",&self._base);
		self._dynamicName = self._dynamicName.replace("{base}",&self._base);
		self._usersaveName = self._usersaveName.replace("{base}",&self._base);
	}
}

impl Default for Paths {
	#[inline]
	fn default() -> Self {
		Self {
			_base: ".".to_string(),
			_staticName: "{base}/static".to_string(),
			_dynamicName: "{base}/dynamic".to_string(),
			_configName: "{base}/config".to_string(),
			_usersaveName: "{base}/save".to_string()
		}
	}
}
