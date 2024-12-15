#![deny(unused_crate_dependencies)]
#![allow(unused_variables, unused_parens, non_snake_case)]
use crate::shaders::loadShaders;
use glyph_brush::OwnedText;
use glyph_brush_layout::{HorizontalAlign, Layout};
#[warn(unused_parens)]
use std::fs;
use std::ops::Add;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use HArcMut::HArcMut;
use Hconfig::HConfigManager::HConfigManager;
use Htrace::CommandLine::{CommandLine, CommandLineConfig};
use Htrace::HTracer::HTracer;
use Htrace::Type::Type;
use HGE::components::color::color;
use HGE::components::corners::corner4;
use HGE::components::event::{event_trait_add, event_type};
use HGE::components::interfacePosition::interfacePosition;
use HGE::components::HGEC_offset;
use HGE::configs::general::{HGEconfig_general, HGEconfig_general_font};
use HGE::entities::Plane::Plane;
use HGE::fronts::export::winit::event::{
	DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent,
};
use HGE::fronts::export::winit::event_loop::ActiveEventLoop;
use HGE::fronts::export::winit::keyboard::KeyCode;
use HGE::fronts::export::winit::window::WindowId;
use HGE::fronts::winit::front::HGEwinit;
use HGE::fronts::winit::winit_UserDefinedEventOverride::winit_UserDefinedEventOverride;
use HGE::Animation::{Animation, AnimationUtils};
use HGE::HGEMain::HGEMain;
use HGE::Interface::ManagerInterface::ManagerInterface;
use HGE::Interface::Text::Text;
use HGE::Interface::UiButton::UiButton;
use HGE::Interface::UiPage::{UiPage, UiPageContent};
use HGE::ManagerAnimation::ManagerAnimation;
use HGE::Paths::Paths;
use HGE::Textures::Manager::ManagerTexture;

mod shaders;

fn main()
{
	// creating default paths, and purging traces from last launch
	let _ = fs::create_dir(Paths::singleton().getConfig());
	let _ = fs::create_dir(Paths::singleton().getDynamic());
	let _ = fs::remove_dir_all(format!("{}{}", Paths::singleton().getDynamic(), "/traces"));

	// defining default behavior for HConfigManager and HTracer
	HConfigManager::singleton().setConfPath(Paths::singleton().getConfig());
	HTracer::minlvl_default(Type::WARNING);
	HTracer::appendModule("cli", CommandLine::new(CommandLineConfig::default())).unwrap();
	HTracer::appendModule(
		"file",
		Htrace::File::File::new(Htrace::File::FileConfig {
			path: format!("{}{}", Paths::singleton().getDynamic(), "/traces"),
			byHour: true,
			..Htrace::File::FileConfig::default()
		}),
	)
	.unwrap();
	HTracer::threadSetName("main");

	let mut hgeWinit = HGEwinit::new();
	let mut engineEvent = hgeWinit.event_mut();
	// loading resource used for this example after engine init
	engineEvent.setFunc_PostInit(|| {
		ManagerTexture::singleton().add("image", "image.png", None);
		ManagerTexture::singleton().add("alpha_test", "alpha_test.png", None);
		build2D();
	});

	// main loop using HGEwinit
	engineEvent.setConfig(HGEconfig_general {
		startFullscreen: false,
		windowTitle: "HGEexample".to_string(),
		defaultShaderLoader: Some(Arc::new(|| loadShaders())),
		fonts: HGEconfig_general_font {
			path_fileUser: "fonts/NotoSans-SemiBold.ttf".to_string(),
			path_fileUniversel: "fonts/NotoSans-SemiBold.ttf".to_string(),
			path_fileBold: "fonts/NotoSans-SemiBold.ttf".to_string(),
		},
		..Default::default()
	});

	hgeWinit.run(Some(&mut Simple2dDatas {
		fpsall: 0,
		fpsmin: 999999,
		fpsmax: 0,
		fpsnb: 1u128,
		start: Instant::now(),
		mousex: 0.0,
		mousey: 0.0,
		mousemoved: false,
		mouseleftclick: false,
		mouseleftcliked: false,
	}));
}

fn build2D()
{
	let mut page = UiPage::new();
	page.eventEnter(|_| {
		println!("page default enter");
		false
	});
	let mut blackBackground = Plane::new();
	blackBackground.setSquare(
		interfacePosition::new_percent(0.0, 0.0),
		interfacePosition::new_percent(1.0, 1.0),
	);
	blackBackground.setColor(corner4::same(color::from([0.0, 0.0, 0.0, 1.0])));
	blackBackground.components_mut().origin_mut().setZ(0);
	page.add("bg", blackBackground);

	let mut image = Plane::new();
	image.setSquare(
		interfacePosition::new_pixel(-100, -100),
		interfacePosition::new_pixel(100, 100),
	);
	image.components_mut().texture_mut().set("image");
	*image.components_mut().origin_mut() = interfacePosition::new_percent_z(0.5, 0.5, 200);
	let imagecontent = page.add("image", image);

	for x in 0..10
	{
		let step = (x - 5) * 5;
		let name = format!("alpha_test{}", x);
		let mut alphaimage = Plane::new();
		alphaimage.setSquare(
			interfacePosition::new_pixel(-75 + step, -75 + step),
			interfacePosition::new_pixel(75 + step, 75 + step),
		);
		alphaimage.components_mut().texture_mut().set("alpha_test");
		alphaimage.components_mut().texture_mut().color_mut().a = 0.3;
		*alphaimage.components_mut().origin_mut() =
			interfacePosition::new_percent_z(0.5, 0.5, (300 + x) as u16);
		page.add(name, alphaimage);
	}

	let mut buttoncontent = Plane::new();
	buttoncontent.setSquare(
		interfacePosition::new_pixel(0, 0),
		interfacePosition::new_pixel(100, 100),
	);
	buttoncontent.setColor(corner4::same(color::from([1.0, 1.0, 1.0, 1.0])));
	buttoncontent.components_mut().origin_mut().setZ(100);

	let mut button = UiButton::new();
	button.add(buttoncontent);
	button.setClickedFn(|x| {
		println!("=== go to second");
		ManagerInterface::singleton().changeActivePage("second");
	});
	page.add("button", button);

	let mut text = Text::new();
	let start = Arc::new(RwLock::new(0u8));
	text.addText(
		OwnedText::new("Test")
			.with_scale(24.0)
			.with_color([1.0, 1.0, 0.0, 1.0]),
	);
	text.setLayout(Layout::default_single_line().h_align(HorizontalAlign::Center));
	*text.components_mut().origin_mut() = interfacePosition::new_percent_z(0.5, 0.1, 300);
	text.event_add(event_type::EACH_SECOND, move |x| {
		x.emptyText();
		let mut points = "".to_string();
		let max = *start.clone().read().unwrap();
		for _ in 0..max
		{
			points = points.add(".");
		}
		x.addText(
			OwnedText::new(format!("Test{}", points))
				.with_scale(24.0)
				.with_color([1.0, 1.0, 0.0, 1.0]),
		);

		if (max == 3)
		{
			*start.clone().write().unwrap() = 0
		}
		else
		{
			*start.clone().write().unwrap() += 1;
		}
		true
	});
	page.add("text", text);

	let pos = Arc::new(RwLock::new(0.0));
	let mut text = Text::new();
	let movepos = pos.clone();
	text.addText(
		OwnedText::new("Test")
			.with_scale(16.0)
			.with_color([1.0, 1.0, 0.0, 1.0]),
	);
	text.setLayout(Layout::default_single_line().h_align(HorizontalAlign::Center));
	*text.components_mut().origin_mut() = interfacePosition::new_percent_z(0.5, 0.15, 300);
	text.event_add(event_type::EACH_TICK, move |x| {
		x.emptyText();
		x.addText(
			OwnedText::new(format!("x: {:.3}", movepos.clone().read().unwrap()))
				.with_scale(16.0)
				.with_color([1.0, 1.0, 0.0, 1.0]),
		);
		true
	});
	page.add("text2", text);
	build2DAnimation(imagecontent, pos);

	let mut fontshow = Plane::new();
	fontshow.setSquare(
		interfacePosition::new_percent(0.0, 0.5),
		interfacePosition::new_percent(0.5, 1.0),
	);
	fontshow.components_mut().texture_mut().set("font");
	fontshow.components_mut().origin_mut().setZ(100);
	page.add("fontshow", fontshow);

	ManagerInterface::singleton().UiPageAppend("default", page);
	ManagerInterface::singleton().changeActivePage("default");

	let mut page = UiPage::new();
	page.eventEnter(|_| {
		//println!("page second enter");
		false
	});
	let mut blackBackground = Plane::new();
	blackBackground.setSquare(
		interfacePosition::new_percent(0.0, 0.0),
		interfacePosition::new_percent(1.0, 1.0),
	);
	blackBackground.setColor(corner4::same(color::from([0.0, 0.0, 1.0, 1.0])));
	blackBackground.components_mut().origin_mut().setZ(0);
	page.add("bg", blackBackground);

	for x in 0..10
	{
		for y in 0..10
		{
			let mut buttoncontent = Plane::new();
			buttoncontent.setSquare(
				interfacePosition::new_pixel(x * 20, y * 20),
				interfacePosition::new_pixel(15 + (x * 20), 15 + (y * 20)),
			);
			buttoncontent.setColor(corner4::same(color::from([1.0, 1.0, 1.0, 1.0])));
			buttoncontent.components_mut().origin_mut().setZ(100);

			let mut button = UiButton::new();
			button.add(buttoncontent);
			button.setClickedFn(|x| {
				//println!("=== go to default");
				ManagerInterface::singleton().changeActivePage("default");
			});
			//println!("button{}_{}",x,y);
			page.add(format!("button{}_{}", x, y), button);
		}
	}

	ManagerInterface::singleton().UiPageAppend("second", page);
}

fn build2DAnimation(
	imagecontent: HArcMut<Box<dyn UiPageContent + Sync + Send>>,
	pos: Arc<RwLock<f32>>,
)
{
	let mut animcam = Animation::new(
		Duration::from_secs(10),
		imagecontent,
		0.0,
		1.0,
		move |selfanim, progress| {
			selfanim.source.update(|tmp| {
				let posx = if (progress < 0.5)
				{
					let localprogress = progress * 2.0;
					AnimationUtils::smoothstep(-0.4, 0.4, localprogress)
				}
				else
				{
					let localprogress = (progress - 0.5) * 2.0;
					AnimationUtils::smoothstep(0.4, -0.4, localprogress)
				};

				*pos.clone().write().unwrap() = posx;

				if let Some(movingimage) =
					tmp.as_any_mut().downcast_mut::<Plane<interfacePosition>>()
				{
					movingimage
						.components_mut()
						.offset_mut()
						.origin_mut()
						.setX(posx);
				}
			});
		},
	);
	animcam.setModeRepeat();
	ManagerAnimation::singleton().append(animcam);
}

/**
 * here we stock events, and they're corresponding variable
 */
struct Simple2dDatas
{
	fpsall: u128,
	fpsmin: u128,
	fpsmax: u128,
	fpsnb: u128,
	start: Instant,

	mousex: f64,
	mousey: f64,
	mousemoved: bool,
	mouseleftclick: bool,
	mouseleftcliked: bool,
}

impl winit_UserDefinedEventOverride for Simple2dDatas
{
	fn resumed(&mut self, _: &mut HGEwinit, _: &ActiveEventLoop) {}

	fn suspended(&mut self, _: &mut HGEwinit, _: &ActiveEventLoop) {}

	fn window_event(
		&mut self,
		root: &mut HGEwinit,
		eventloop: &ActiveEventLoop,
		event: &WindowEvent,
		window_id: WindowId,
	)
	{
		match event
		{
			WindowEvent::MouseInput { button, state, .. } =>
			{
				//println!("button {:?} : {:?}",button,state);
				if (*button == MouseButton::Left)
				{
					self.mouseleftclick = *state == ElementState::Pressed;
				}
			}
			WindowEvent::CursorMoved { position, .. } =>
			{
				self.mousex = position.x;
				self.mousey = position.y;
				self.mousemoved = true;
			}
			_ => (),
		}
	}

	fn device_event(&mut self, _: &mut HGEwinit, _: &ActiveEventLoop, _: &DeviceEvent, _: DeviceId)
	{
	}

	fn about_to_render(&mut self, root: &mut HGEwinit, eventloop: &ActiveEventLoop)
	{
		if (self.mousemoved || self.mouseleftclick)
		{
			ManagerInterface::singleton().mouseUpdate(
				self.mousex as u16,
				self.mousey as u16,
				(self.mouseleftclick ^ self.mouseleftcliked),
			);
		}
		self.mouseleftcliked = self.mouseleftclick;
		self.mousemoved = false;
	}

	fn about_to_wait(&mut self, root: &mut HGEwinit, eventloop: &ActiveEventLoop)
	{
		if (root
			.Inputs_getmut()
			.getKeyboardStateAndSteal(KeyCode::Escape)
			== ElementState::Pressed)
		{
			eventloop.exit();
		}

		let fps = HGEMain::singleton().getTimer().getFps() as u128;
		if (self.start.elapsed().as_secs() > 3)
		{
			if (fps > self.fpsmax)
			{
				self.fpsmax = fps;
			}
			if (fps < self.fpsmin)
			{
				self.fpsmin = fps;
			}
			self.fpsall += fps;
			self.fpsnb += 1;
		}

		//println!("fps : {} - {} - {}", self.fpsall / self.fpsnb, self.fpsmax, self.fpsmin);
	}
}
