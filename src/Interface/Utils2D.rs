use palette::{IntoColor};

pub fn hsluvFrom(hue: f32, saturation: f32,I : f32) -> [f32;4]
{
	let color: palette::Srgba = palette::Hsluv::new(hue,saturation,I).into_color();
	return [color.red,color.green,color.blue,color.alpha];
}

pub fn hsluvaFrom(hue: f32, saturation: f32,I : f32,alpha :f32) -> [f32;4]
{
	let color: palette::Srgba = palette::Hsluva::new(hue,saturation,I,alpha).into_color();
	return [color.red,color.green,color.blue,color.alpha];
}

pub fn rgbFrom(red: f32, green: f32,blue : f32) -> [f32;4]
{
	let color = palette::Srgb::new(red,green,blue);
	return [color.red,color.green,color.blue,1.0];
}

pub fn rgbaFrom(red: f32, green: f32,blue : f32,alpha :f32) -> [f32;4]
{
	let color = palette::Srgba::new(red,green,blue,alpha);
	return [color.red,color.green,color.blue,color.alpha];
}

pub fn Color_u8IntoF32(base: [u8;4]) -> [f32;4]
{
	return [
		base[0] as f32/255.0,
		base[1] as f32/255.0,
		base[2] as f32/255.0,
		base[3] as f32/255.0
	]
}

pub fn ColorInterval(base: [f32;4],target: [f32;4],percent: f32) -> [f32;4]
{
	let percent = percent.clamp(0.0,1.0);
	return [
		base[0] + ((target[0] - base[0]) * percent),
		base[1] + ((target[1] - base[1]) * percent),
		base[2] + ((target[2] - base[2]) * percent),
		base[3] + ((target[3] - base[3]) * percent),
	];
}
