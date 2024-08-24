#![deny(unused_crate_dependencies)]
#![allow(unused_variables,unused_parens, non_snake_case)]
#[warn(unused_parens)]

use std::fs;
use std::ops::Add;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use glyph_brush::OwnedText;
use glyph_brush_layout::{HorizontalAlign, Layout};
use Hconfig::HConfigManager::HConfigManager;
use HGE::Animation::{Animation, AnimationUtils};
use HGE::components::color::color;
use HGE::components::corners::corner4;
use HGE::components::HGEC_offset;
use HGE::components::interfacePosition::interfacePosition;
use HGE::configs::general::{HGEconfig_general, HGEconfig_general_font};
use HGE::entities::Plane::Plane;
use HGE::fronts::winit::HGEwinit;
use HGE::fronts::winit::winit::event_loop::EventLoop;
use HGE::fronts::winit::winit::event::{ElementState, Event, MouseButton, WindowEvent};
use HGE::HGEMain::HGEMain;
use HGE::Interface::ManagerInterface::ManagerInterface;
use HGE::Interface::UiPage::{UiPage, UiPageContent};
use HGE::ManagerAnimation::ManagerAnimation;
use HGE::Paths::Paths;
use HGE::Textures::Manager::ManagerTexture;
use Htrace::CommandLine::{CommandLine, CommandLineConfig};
use Htrace::HTracer::HTracer;
use Htrace::Type::Type;
use crate::shaders::loadShaders;
use HArcMut::HArcMut;
use HGE::components::event::{event_trait_add, event_type};
use HGE::Interface::Text::Text;
use HGE::Interface::UiButton::UiButton;

mod shaders;

fn main()
{
	let _ = fs::create_dir(Paths::singleton().getConfig());
	let _ = fs::create_dir(Paths::singleton().getDynamic());
	let _ = fs::remove_dir_all(format!("{}{}", Paths::singleton().getDynamic(), "/traces"));
	
	
	HConfigManager::singleton().setConfPath(Paths::singleton().getConfig());
	HTracer::minlvl_default(Type::WARNING);
	HTracer::appendModule("cli", CommandLine::new(CommandLineConfig::default())).unwrap();
	HTracer::appendModule("file", Htrace::File::File::new(Htrace::File::FileConfig {
		path: format!("{}{}", Paths::singleton().getDynamic(), "/traces"),
		byHour: true,
		..Htrace::File::FileConfig::default()
	})).unwrap();
	
	HTracer::threadSetName("main");
	
	HGEwinit::singleton().setFunc_PostInit(||{
		ManagerTexture::singleton().add("image", "image.png", None);
		ManagerTexture::singleton().add("alpha_test", "alpha_test.png", None);
		build2D();
	});
	
	let mut fpsall = 0;
	let mut fpsmin = 999999;
	let mut fpsmax = 0;
	let mut fpsnb = 1u128;
	let start = Instant::now();
	
	let mut mousex = 0.0;
	let mut mousey = 0.0;
	let mut mousemoved = false;
	let mut mouseleftclick = false;
	let mut mouseleftcliked = false;
	
	HGEwinit::run(EventLoop::new().unwrap(),HGEconfig_general{
		startFullscreen: false,
		windowTitle: "HGEexample".to_string(),
		defaultShaderLoader: Some(Arc::new(||{loadShaders()})),
		fonts: HGEconfig_general_font {
			path_fileUser: "fonts/NotoSans-SemiBold.ttf".to_string(),
			path_fileUniversel: "fonts/NotoSans-SemiBold.ttf".to_string(),
			path_fileBold: "fonts/NotoSans-SemiBold.ttf".to_string(),
		},
		..Default::default()
	},&mut move |event, _|{
		match event {
			Event::WindowEvent {
				event: WindowEvent::MouseInput {
					button,
					state, ..
				}, ..
			} => {
				//println!("button {:?} : {:?}",button,state);
				if (*button == MouseButton::Left)
				{
					mouseleftclick = *state == ElementState::Pressed;
				}
			}
			Event::WindowEvent {
				event: WindowEvent::CursorMoved {
					position, ..
				}, ..
			} => {
				mousex = position.x;
				mousey = position.y;
				mousemoved = true;
			},
			_ => ()
		}
		let fps = HGEMain::singleton().getTimer().getFps() as u128;
		if(start.elapsed().as_secs()>3)
		{
			if (fps > fpsmax)
			{
				fpsmax = fps;
			}
			if (fps < fpsmin)
			{
				fpsmin = fps;
			}
			fpsall+=fps;
			fpsnb+=1;
		}
		
		if (mousemoved || mouseleftclick)
		{
			ManagerInterface::singleton().mouseUpdate(mousex as u16, mousey as u16, (mouseleftclick ^ mouseleftcliked));
		}
		
		mouseleftcliked = mouseleftclick;
		mousemoved = false;
		//println!("fps : {} - {} - {fpsmax} - {fpsmin}",fps, fpsall/fpsnb);
	});
}

fn build2D()
{
	let mut page = UiPage::new();
	page.eventEnter(|_|{
		println!("page default enter");
		false
	});
	let mut blackBackground = Plane::new();
	blackBackground.setSquare(interfacePosition::new_percent(0.0, 0.0), interfacePosition::new_percent(1.0, 1.0));
	blackBackground.setColor(corner4::same(color::from([0.0, 0.0, 0.0, 1.0])));
	blackBackground.components_mut().origin_mut().setZ(0);
	page.add("bg", blackBackground);
	
	let mut image = Plane::new();
	image.setSquare(interfacePosition::new_pixel(-100,-100),interfacePosition::new_pixel(100,100));
	image.components_mut().texture_mut().set("image");
	*image.components_mut().origin_mut() = interfacePosition::new_percent_z(0.5,0.5,200);
	let imagecontent = page.add("image", image);
	
	for x in 0..10
	{
		let step = (x-5)*5;
		let name = format!("alpha_test{}",x);
		let mut alphaimage = Plane::new();
		alphaimage.setSquare(interfacePosition::new_pixel(-75+step, -75+step), interfacePosition::new_pixel(75+step, 75+step));
		alphaimage.components_mut().texture_mut().set("alpha_test");
		alphaimage.components_mut().texture_mut().color_mut().a = 0.3;
		*alphaimage.components_mut().origin_mut() = interfacePosition::new_percent_z(0.5, 0.5, (300 + x) as u16);
		page.add(name, alphaimage);
	}
	
	
	let mut buttoncontent = Plane::new();
	buttoncontent.setSquare(interfacePosition::new_pixel(0, 0), interfacePosition::new_pixel(100, 100));
	buttoncontent.setColor(corner4::same(color::from([1.0, 1.0, 1.0, 1.0])));
	buttoncontent.components_mut().origin_mut().setZ(100);
	
	let mut button = UiButton::new();
	button.add(buttoncontent);
	button.setClickedFn(|x|{
		println!("=== go to second");
		ManagerInterface::singleton().changeActivePage("second");
	});
	page.add("button",button);
	
	let mut text = Text::new();
	let start = Arc::new(RwLock::new(0u8));
	text.addText(OwnedText::new("Test").with_scale(24.0).with_color([1.0,1.0,0.0,1.0]));
	text.setLayout(Layout::default_single_line().h_align(HorizontalAlign::Center));
	*text.components_mut().origin_mut() = interfacePosition::new_percent_z(0.5,0.1,300);
	text.event_add(event_type::EACH_SECOND,move |x|{
		x.emptyText();
		let mut points = "".to_string();
		let max = *start.clone().read().unwrap();
		for _ in 0..max
		{
			points = points.add(".");
		}
		x.addText(OwnedText::new(format!("Test{}",points)).with_scale(24.0).with_color([1.0,1.0,0.0,1.0]));
		
		if(max==3)
		{
			*start.clone().write().unwrap() = 0
		}
		else
		{
			*start.clone().write().unwrap() += 1;
		}
		x.commit();
		
		true
	});
	page.add("text",text);
	
	let pos = Arc::new(RwLock::new(0.0));
	let mut text = Text::new();
	let movepos = pos.clone();
	text.addText(OwnedText::new("Test").with_scale(16.0).with_color([1.0,1.0,0.0,1.0]));
	text.setLayout(Layout::default_single_line().h_align(HorizontalAlign::Center));
	*text.components_mut().origin_mut() = interfacePosition::new_percent_z(0.5,0.15,300);
	text.event_add(event_type::EACH_TICK,move |x|{
		x.emptyText();
		x.addText(OwnedText::new(format!("x: {:.3}",movepos.clone().read().unwrap())).with_scale(16.0).with_color([1.0,1.0,0.0,1.0]));
		x.commit();
		true
	});
	page.add("text2",text);
	build2DAnimation(imagecontent, pos);
	
	let mut fontshow = Plane::new();
	fontshow.setSquare(interfacePosition::new_percent(0.0, 0.5), interfacePosition::new_percent(0.5, 1.0));
	fontshow.components_mut().texture_mut().set("font");
	fontshow.components_mut().origin_mut().setZ(100);
	page.add("fontshow",fontshow);
	
	
	ManagerInterface::singleton().UiPageAppend("default", page);
	ManagerInterface::singleton().changeActivePage("default");
	
	let mut page = UiPage::new();
	page.eventEnter(|_|{
		//println!("page second enter");
		false
	});
	let mut blackBackground = Plane::new();
	blackBackground.setSquare(interfacePosition::new_percent(0.0, 0.0), interfacePosition::new_percent(1.0, 1.0));
	blackBackground.setColor(corner4::same(color::from([0.0, 0.0, 1.0, 1.0])));
	blackBackground.components_mut().origin_mut().setZ(0);
	page.add("bg", blackBackground);
	
	for x in 0..10
	{
		for y in 0..10
		{
			let mut buttoncontent = Plane::new();
			buttoncontent.setSquare(interfacePosition::new_pixel(x*20, y*20), interfacePosition::new_pixel(15+(x*20), 15+(y*20)));
			buttoncontent.setColor(corner4::same(color::from([1.0, 1.0, 1.0, 1.0])));
			buttoncontent.components_mut().origin_mut().setZ(100);
			
			let mut button = UiButton::new();
			button.add(buttoncontent);
			button.setClickedFn(|x|{
				//println!("=== go to default");
				ManagerInterface::singleton().changeActivePage("default");
			});
			//println!("button{}_{}",x,y);
			page.add(format!("button{}_{}",x,y),button);
		}
	}
	
	ManagerInterface::singleton().UiPageAppend("second", page);
}

fn build2DAnimation(imagecontent: HArcMut<Box<dyn UiPageContent + Sync + Send>>, pos: Arc<RwLock<f32>>)
{
	let mut animcam = Animation::new(Duration::from_secs(10), imagecontent, 0.0, 1.0, move |selfanim, progress|
		{
			selfanim.source.update(|tmp|{
				let posx= if (progress < 0.5)
				{
					let localprogress = progress * 2.0;
					AnimationUtils::smoothstep(-0.4, 0.4, localprogress)
				} else {
					let localprogress = (progress - 0.5) * 2.0;
					AnimationUtils::smoothstep(0.4, -0.4, localprogress)
				};
				
				*pos.clone().write().unwrap() = posx;
				
				if let Some(movingimage) = tmp.as_any_mut().downcast_mut::<Plane<interfacePosition>>()
				{
					movingimage.components_mut().offset_mut().origin_mut().setX(posx);
				}
			});
		});
	animcam.setModeRepeat();
	ManagerAnimation::singleton().append(animcam);
}
