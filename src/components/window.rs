use derive_more::Display;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::swapchain::SurfaceTransform;

/// rotation is clock wise
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Debug, Display)]
pub enum window_orientation
{
	NORMAL,
	ROT_90,
	ROT_180,
	ROT_270
}

impl window_orientation
{
	pub fn getDeg(&self) -> f32
	{
		match self {
			window_orientation::NORMAL => 0.0,
			window_orientation::ROT_90 => 90.0,
			window_orientation::ROT_180 => 180.0,
			window_orientation::ROT_270 => 270.0
		}
	}
}

impl Default for window_orientation
{
	fn default() -> Self {
		Self::NORMAL
	}
}

impl From<SurfaceTransform> for window_orientation
{
	fn from(value: SurfaceTransform) -> Self {
		match value {
			SurfaceTransform::Rotate90 => Self::ROT_90,
			SurfaceTransform::Rotate180 => Self::ROT_180,
			SurfaceTransform::Rotate270 => Self::ROT_270,
			_ => Self::NORMAL,
		}
	}
}

impl Into<SurfaceTransform> for window_orientation
{
	fn into(self) -> SurfaceTransform {
		match self {
			Self::ROT_90 => SurfaceTransform::Rotate90,
			Self::ROT_180 => SurfaceTransform::Rotate180,
			Self::ROT_270 => SurfaceTransform::Rotate270,
			_ => SurfaceTransform::Identity,
		}
	}
}

/// rotation is clock wise
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Debug, Display)]
pub enum window_type
{
	FULLSCREEN,
	BORDERLESS,
	WINDOW
}


#[derive(Copy, Clone, Debug)]
pub struct window_infos
{
	pub originx: f32,
	pub originy: f32,
	pub width: u32,
	pub height: u32,
	pub widthF: f32,
	pub heightF: f32,
	pub raw_width: u32,
	pub raw_height: u32,
	pub raw_widthF: f32,
	pub raw_heightF: f32,
	pub ratio_w2h: f32,
	pub ratio_h2w: f32,
	pub isWide: bool,
	
	pub HDPI: f32,
	pub orientation: window_orientation,
}

impl window_infos
{
	/// return wide if window is wider or tall
	pub fn if_wide<T>(&self, wide: T, tall: T) -> T
	{
		if(self.isWide)
		{
			return wide;
		}
		return tall;
	}
	
	pub fn ViewPort(&self) -> Viewport
	{
		Viewport {
			offset: [self.originx, self.originy],
			extent: [self.widthF,self.heightF],
			depth_range: 0.0..=1.0,
		}
	}
	
	pub fn raw(&self) -> [u32; 2]
	{
		[self.raw_width,self.raw_height]
	}
}

impl Default for window_infos
{
	fn default() -> Self {
		Self{
			originx: 0.0,
			originy: 0.0,
			width: 1,
			height: 1,
			widthF: 1.0,
			heightF: 1.0,
			raw_width: 1,
			raw_height: 1,
			raw_widthF: 1.0,
			raw_heightF: 1.0,
			ratio_w2h: 1.0,
			ratio_h2w: 1.0,
			isWide: false,
			HDPI: 1.0,
			orientation: Default::default(),
		}
	}
}

impl Into<[f32; 2]> for window_infos
{
	fn into(self) -> [f32; 2] {
		[self.widthF, self.heightF]
	}
}

impl Into<[f32; 4]> for window_infos
{
	fn into(self) -> [f32; 4] {
		[self.widthF, self.heightF, self.ratio_w2h, self.ratio_h2w]
	}
}

impl Into<[u32; 2]> for window_infos
{
	fn into(self) -> [u32; 2] {
		[self.width, self.height]
	}
}
