
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::sync::OnceLock;
use parking_lot::RwLock;
use crate::Paths::Paths;
use std::path::Path;
use Htrace::HTrace;

#[cfg(target_os = "android")]
use ndk::asset::AssetManager;
#[cfg(target_os = "android")]
use std::ffi::CString;
use std::fs;
#[cfg(target_os = "android")]
use std::io::Write;
#[cfg(target_os = "android")]
use android_activity::AndroidApp;


pub trait SeekRead: Seek + Read + Send + Sync {}

pub struct assetManager
{
	#[cfg(target_os = "android")]
	assetManager: RwLock<Option<AssetManager>>,
	
	#[cfg(not(target_os = "android"))]
	assetManager: RwLock<Option<()>>,
}

static SINGLETON: OnceLock<assetManager> = OnceLock::new();

impl assetManager
{
	pub fn singleton() -> &'static assetManager
	{
		return SINGLETON.get_or_init(|| {
			Self{
				assetManager: RwLock::new(None)
			}
		});
	}
	
	#[cfg(target_os = "android")]
	pub fn setApp(&self,app: &AndroidApp)
	{
		*self.assetManager.write() = Some(app.asset_manager());
	}
	
	pub fn readAllFileInDir(&self,path: impl Into<String>) -> Vec<String>
	{
		let path = format!("{}/{}",Paths::singleton().getStatic(),path.into());
		let mut returnfilename = vec![];
		
		#[cfg(target_os = "android")]
		if let Some(assetmanager) = &*self.assetManager.read()
		{
			if let Ok(pathconv) = &CString::new(path.clone())
			{
				if let Some(dir) = assetmanager.open_dir(pathconv)
				{
					for x in dir {
						if let Ok(tmp) = x.to_str()
						{
							returnfilename.push(tmp.to_string())
						}
					}
					return returnfilename;
				}
			}
		}
		
		if let Ok(dirpath) = Path::new(path.as_str()).read_dir()
		{
			for x in dirpath.filter_map(|e| e.ok())
				.filter(|x| { x.file_type().unwrap().is_file() })
			{
				returnfilename.push(x.file_name().to_str().unwrap().to_string());
			}
		}
		
		return returnfilename;
	}
	
	pub fn checkFile(&self, path: impl Into<String>) -> bool
	{
		let path = format!("{}/{}",Paths::singleton().getStatic(),path.into());
		
		HTrace!("assetManager checkfile {}",path);
		
		#[cfg(target_os = "android")]
		if let Some(assetmanager) = &*self.assetManager.read()
		{
			if let Ok(pathconv) = &CString::new(path.clone())
			{
				if let Some(_) = assetmanager.open(pathconv)
				{
					return true;
				}
			}
		}
		
		if let Ok(_) = File::open(path.clone())
		{
			return true;
		}
		return false;
		
	}
	
	pub fn readFile(&self, path: impl Into<String>) -> Option<Cursor<Vec<u8>>>
	{
		let path = format!("{}/{}",Paths::singleton().getStatic(),path.into());
		let mut returnDatas = vec![];
		HTrace!("assetManager readFile {}",path);
		
		#[cfg(target_os = "android")]
		if let Some(assetmanager) = &*self.assetManager.read()
		{
			if let Ok(pathconv) = &CString::new(path.clone())
			{
				if let Some(mut tmp) = assetmanager.open(pathconv)
				{
					let _ = tmp.read_to_end(&mut returnDatas);
					return Some(Cursor::new(returnDatas));
				}
			}
		}
		
		
		if let Ok(mut tmp) = File::open(path)
		{
			let _ = tmp.read_to_end(&mut returnDatas);
			return Some(Cursor::new(returnDatas));
		}
		return None;
	}
	
	pub fn copyFile(&self, path: impl Into<String>, otherpath: impl Into<String>)
	{
		let path = path.into();
		let otherpath = otherpath.into();
		HTrace!("assetManager copyFile {} => {}",path,otherpath);
		
		#[cfg(target_os = "android")]
		if let Some(assetmanager) = &*self.assetManager.read()
		{
			if let Ok(pathconv) = &CString::new(path.clone())
			{
				HTrace!("assetManager CString {:?}",pathconv);
				if let Some(mut tmp) = assetmanager.open(pathconv)
				{
					HTrace!("assetManager open");
					let mut returnDatas = vec![];
					let _ = tmp.read_to_end(&mut returnDatas);
					if let Ok(mut otherfile) = File::create(otherpath.clone())
					{
						HTrace!("assetManager other ok");
						let _ = otherfile.write_all(returnDatas.as_slice());
						return;
					}
				}
			}
			return;
		}
		
		let _ = fs::copy(path, otherpath);
	}
	
	/*fn returnBufreaderNormalized<B: Read + Seek>(&self, contentBase: impl SeekRead + 'static) -> BufReader<B>
	{
		return BufReader::new(Box::new(contentBase))
	}*/
	
}
