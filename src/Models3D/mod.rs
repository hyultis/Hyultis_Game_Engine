use std::ops::{Index, IndexMut};
use cgmath::{InnerSpace, Vector3};
use crate::Shaders::Shs_3DVertex::HGE_shader_3Dsimple;

pub mod ManagerModels;
pub mod chunk;
pub mod chunk_content;

pub struct ModelUtils
{}

impl ModelUtils
{
	// found on : https://stackoverflow.com/questions/6656358/calculating-normals-in-a-triangle-mesh/6661242#6661242
	pub fn generateNormal(datas: &mut Vec<HGE_shader_3Dsimple>, indices: Vec<u32>)
	{
		if(indices.len()%3 !=0)
		{
			return;
		}
		
		for pass in 0..indices.len()/3
		{
			let vertex1 = Vector3::from(datas.index(indices[pass] as usize).position);
			let vertex2 = Vector3::from(datas.index(indices[pass+1] as usize).position);
			let vertex3 = Vector3::from(datas.index(indices[pass+2] as usize).position);
			
			let vector1 = vertex2 - vertex1;
			let vector2 = vertex3 - vertex1;
			let faceNormal = vector2.cross(vector1);
			faceNormal.normalize();
			
			let tmp = Vector3::from(datas.index(indices[pass] as usize).normal) + faceNormal;
			datas.index_mut(indices[pass] as usize).normal = [tmp.x,tmp.y,tmp.z];
			let tmp = Vector3::from(datas.index(indices[pass+1] as usize).normal) + faceNormal;
			datas.index_mut(indices[pass+1] as usize).normal = [tmp.x,tmp.y,tmp.z];
			let tmp = Vector3::from(datas.index(indices[pass+2] as usize).normal) + faceNormal;
			datas.index_mut(indices[pass+2] as usize).normal = [tmp.x,tmp.y,tmp.z];
		}
	}
}
