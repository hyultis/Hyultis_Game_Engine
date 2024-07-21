use std::collections::BTreeMap;
use std::sync::Arc;
use HGE::components::cgmath::{Deg, Matrix3, Matrix4};
use Htrace::HTraceError;
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use vulkano::pipeline::PipelineLayout;
use HGE::HGEMain::HGEMain;
use HGE::Shaders::{Manager, names};
use HGE::Shaders::Manager::{Shader_type, ShaderContent};

pub mod HGE_rawshader_3Dinstance_vertex {
	vulkano_shaders::shader! {
		ty: "vertex",
		path: "./tests/shaders_glsl/instance3D/vert.glsl"
	}
}

pub mod HGE_rawshader_3Dinstance_frag {
	vulkano_shaders::shader! {
		ty: "fragment",
		path: "./tests/shaders_glsl/instance3D/frag.glsl"
	}
}

pub mod HGE_rawshader_screen_vert {
	vulkano_shaders::shader! {
		ty: "vertex",
		path: "./tests/shaders_glsl/screen/vert.glsl"
	}
}

pub mod HGE_rawshader_screen_frag {
	vulkano_shaders::shader! {
		ty: "fragment",
		path: "./tests/shaders_glsl/screen/frag.glsl"
	}
}

pub mod HGE_rawshader_2Dsimple_vert {
	vulkano_shaders::shader! {
		ty: "vertex",
		path: "./tests/shaders_glsl/simple2D/vert.glsl",
	}
}

pub mod HGE_rawshader_2Dsimple_frag {
	vulkano_shaders::shader! {
		ty: "fragment",
		path: "./tests/shaders_glsl/simple2D/frag.glsl"
	}
}

pub mod HGE_rawshader_3Dsimple_vert {
	vulkano_shaders::shader! {
		ty: "vertex",
		path: "./tests/shaders_glsl/simple3D/vert.glsl",
	}
}

pub mod HGE_rawshader_3Dsimple_frag {
	vulkano_shaders::shader! {
		ty: "fragment",
		path: "./tests/shaders_glsl/simple3D/frag.glsl"
	}
}


pub fn loadShaders()
{
	let device = HGEMain::singleton().getDevice().device.clone();
	let mut shaders = BTreeMap::new();
	shaders.insert(Shader_type::VERTEX, HGE_rawshader_2Dsimple_vert::load(device.clone()).unwrap());
	shaders.insert(Shader_type::FRAGMENT, HGE_rawshader_2Dsimple_frag::load(device.clone()).unwrap());
	Manager::ManagerShaders::singleton().add(names::simple2D, ShaderContent{
		shader: shaders,
		pushConstant_Func: Arc::new(|cmdBuilder, pipeline_layout,offset|{
			let windowdim = HGEMain::singleton().getWindowInfos();
			let rotation = windowdim.orientation.getDeg();
			let worldMatrix = Matrix4::from(Matrix3::from_angle_z(Deg(rotation)));
			let uniform_data = HGE_rawshader_2Dsimple_vert::PushConstants {
				window: windowdim.into(),
				time: HGEMain::singleton().getDurationFromStart().as_secs_f32().into(),
				world: worldMatrix.into(),
			};
			HTraceError!(cmdBuilder.push_constants(pipeline_layout,	offset,uniform_data));
		}),
		constantFunc: "".to_string(),
	});
	
	let func = |cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, pipeline_layout: Arc<PipelineLayout>,offset: u32|{
		let bindingcameraC = HGEMain::singleton().getCamera();
		let cameraC = bindingcameraC.get();
		let windowdim = HGEMain::singleton().getWindowInfos();
		let rotation = windowdim.orientation.getDeg();
		let viewMatrix = cameraC.getPositionMatrix(rotation);
		
		let worldMatrix = Matrix4::from(Matrix3::from_angle_y(Deg(0.0)));
		let projMatrix = cameraC.getProjectionMatrix();
		
		let tmp = projMatrix * viewMatrix * worldMatrix;
		let uniform_data = HGE_rawshader_3Dsimple_vert::PushConstants {
			projviewworld: tmp.into(),
			window: windowdim.into(),
			time: HGEMain::singleton().getDurationFromStart().as_secs_f32().into(),
		};
		
		HTraceError!(cmdBuilder.push_constants(pipeline_layout,	offset,uniform_data));
	};
	
	let mut shaders = BTreeMap::new();
	shaders.insert(Shader_type::VERTEX, HGE_rawshader_3Dinstance_vertex::load(device.clone()).unwrap());
	shaders.insert(Shader_type::FRAGMENT, HGE_rawshader_3Dinstance_frag::load(device.clone()).unwrap());
	Manager::ManagerShaders::singleton().add(names::instance3D, ShaderContent{
		shader: shaders.clone(),
		pushConstant_Func: Arc::new(func.clone()),
		constantFunc: "".to_string(),
	});
	let mut shaders = BTreeMap::new();
	shaders.insert(Shader_type::VERTEX, HGE_rawshader_3Dsimple_vert::load(device.clone()).unwrap());
	shaders.insert(Shader_type::FRAGMENT, HGE_rawshader_3Dsimple_frag::load(device.clone()).unwrap());
	Manager::ManagerShaders::singleton().add(names::simple3D, ShaderContent{
		shader: shaders,
		pushConstant_Func: Arc::new(func),
		constantFunc: "".to_string(),
	});
	
	let mut shaders = BTreeMap::new();
	shaders.insert(Shader_type::VERTEX, HGE_rawshader_screen_vert::load(device.clone()).unwrap());
	shaders.insert(Shader_type::FRAGMENT, HGE_rawshader_screen_frag::load(device.clone()).unwrap());
	Manager::ManagerShaders::singleton().add(names::screen, ShaderContent{
		shader: shaders,
		pushConstant_Func: Arc::new(|cmdBuilder, pipeline_layout,offset|{
			HTraceError!(cmdBuilder.push_constants(pipeline_layout,	offset,HGE_rawshader_screen_vert::PushConstants {
				time: HGEMain::singleton().getDurationFromStart().as_secs_f32().into(),
				window: HGEMain::singleton().getWindowInfos().into()
			}));
		}),
		constantFunc: "".to_string(),
	});
}
