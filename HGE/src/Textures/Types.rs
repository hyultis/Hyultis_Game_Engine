#[derive(Copy, Clone, Debug)]
pub struct TextureChannel
{
	channel_id: u8,
	texture_id: [u8;3]
}

impl TextureChannel
{
	/// texture id will be clamped to u24::MAX
	pub fn new(channel: u8, texture: u32) -> Self
	{
		let [a,b,c,_] = texture.clamp(0,16777215).to_le_bytes();
		Self{
			channel_id: channel,
			texture_id: [a,b,c],
		}
	}
	
	pub fn get_textureid(&self) -> u32
	{
		let [a,b,c] = self.texture_id;
		u32::from_le_bytes([a,b,c,0])
	}
	
	pub fn get_channelid(&self) -> u8
	{
		self.channel_id
	}
}

impl Into<u32> for TextureChannel
{
	fn into(self) -> u32 {
		let [a,b,c] = self.texture_id;
		return u32::from_le_bytes([a,b,c,self.channel_id])+1;
	}
}

impl From<u32> for TextureChannel
{
	fn from(value: u32) -> Self {
		let [a,b,c,d] = (value-1).to_le_bytes();
		Self{
			channel_id: d,
			texture_id: [a,b,c],
		}
	}
}

impl Default for TextureChannel
{
	fn default() -> Self {
		Self{
			channel_id: 0,
			texture_id: [0,0,0],
		}
	}
}
