use ahash::{HashMap, HashMapExt};
use anyhow::anyhow;
use csv::ReaderBuilder;
use crate::assetStreamReader::assetManager;
use crate::Textures::Textures::{Texture, Texture_part};

pub trait texturePart
{
	fn load(&self, texture: &Texture) -> anyhow::Result<HashMap<String,Texture_part>>;
}

pub struct texturePart_FromCSV
{
	pub path: String
}

impl texturePart for texturePart_FromCSV
{
	fn load(&self, texture: &Texture) -> anyhow::Result<HashMap<String,Texture_part>>
	{
		let textureWidth = texture.width.ok_or(anyhow!("texture not loaded"))? as f32;
		let textureHeight = texture.height.ok_or(anyhow!("texture not loaded"))? as f32;
		
		
		let fileread = assetManager::singleton().readFile(&self.path);
		if(fileread.is_none())
		{
			return Err(anyhow!("cannot load : {}",self.path));
		}
		let fileread = fileread.unwrap();
		
		let mut csv = ReaderBuilder::new().from_reader(fileread);
		//let mut csv = csv::Reader::from_path(Path::new(format!("{}/{}",Paths::singleton().getStatic(),self.path).as_str()))?;
		let mut array = HashMap::new();
		for result in csv.records() {
			if(result.is_err())
			{
				continue;
			}
			
			let cols = result.unwrap();
			
			let width = cols[3].parse::<f32>().unwrap_or(0.0) as f32;
			let height = cols[4].parse::<f32>().unwrap_or(0.0) as f32;
			let posx = cols[5].parse::<f32>().unwrap_or(0.0);
			let posy = cols[6].parse::<f32>().unwrap_or(0.0);
			
			
			let uv = [
				[
					posx/textureWidth,
					(textureHeight-(posy+height))/textureHeight
				],
				[
					(posx+width)/textureWidth,
					(textureHeight-posy)/textureHeight
				]
			];
			array.insert(cols[7].to_string(),Texture_part{
				uvcoord: uv,
				dim: [width as u32,height as u32],
			});
			//println!("{} : {:?}",cols[7].to_string(),uv);
		}
		return Ok(array);
	}
}
