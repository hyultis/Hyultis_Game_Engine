#![allow(non_snake_case)]
#![allow(dead_code)]

use HGE::Textures::Types::TextureChannel;

mod shaders;

#[test]
fn update() {
	assert_eq!(43, 43);
	
	
	println!("toto {:?} {:?} {:?} {:?} {:?}",0u32.to_le_bytes(),100u32.to_le_bytes(),1000000u32.to_le_bytes(),16777216u32.to_le_bytes(),16777217u32.to_le_bytes());
	let original = TextureChannel::new(0,13324);
	let tmp: u32 = original.into();
	let reversed = TextureChannel::from(tmp);
	println!("result : {:?} => {} => {:?} => {}",original,tmp,reversed,reversed.get_textureid());
	let original = TextureChannel::new(2,1000000);
	let tmp: u32 = original.into();
	let reversed = TextureChannel::from(tmp);
	println!("result : {:?} => {} => {:?} => {}",original,tmp,reversed,reversed.get_textureid());
}
