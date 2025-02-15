use crate::HGEsubpass::HGEsubpassName;
use crate::Pipeline::EnginePipelines;
use crate::Pipeline::ManagerPipeline::ManagerPipeline;
use crate::Shaders::intoVertexed::IntoVertexted;
use crate::Shaders::names;
use crate::Shaders::HGE_shader_3Dsimple::HGE_shader_3Dsimple_def;
use crate::Shaders::Manager::ManagerShaders;
use crate::Shaders::ShaderDrawerImpl::ShaderDrawerImplStruct;
use crate::Shaders::ShaderStruct::{ShaderStruct, ShaderStructHolder};
use crate::Shaders::ShaderStructCacheInstanced::ShaderStructCacheInstanced;
use crate::Textures::Manager::ManagerTexture;
use anyhow::anyhow;
use arc_swap::{ArcSwap, ArcSwapOption};
use dashmap::DashMap;
use std::convert::TryInto;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use vulkano::buffer::BufferContents;
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::PipelineBindPoint;
use Htrace::HTraceError;

impl IntoVertexted<HGE_shader_3Dinstance> for HGE_shader_3Dsimple_def
{
	fn IntoVertexted(&self, _: bool) -> Option<HGE_shader_3Dinstance>
	{
		return Some(HGE_shader_3Dinstance {
			position: self.position,
			color: self.color,
			texcoord: self.uvcoord,
		});
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Vertex, BufferContents)]
pub struct HGE_shader_3Dinstance
{
	#[format(R32G32B32_SFLOAT)]
	pub position: [f32; 3],
	#[format(R32G32_SFLOAT)]
	pub texcoord: [f32; 2],
	#[format(R32G32B32A32_SFLOAT)]
	pub color: [f32; 4],
}

impl Default for HGE_shader_3Dinstance
{
	fn default() -> Self
	{
		HGE_shader_3Dinstance {
			position: [0.0, 0.0, 0.0],
			texcoord: [0.0, 0.0],
			color: [1.0, 1.0, 1.0, 1.0],
		}
	}
}

impl ShaderStruct for HGE_shader_3Dinstance
{
	fn createPipeline() -> anyhow::Result<()>
	{
		if ManagerShaders::singleton().get(names::instance3D).is_none()
		{
			return Err(anyhow!("missing shader \"{}\"", names::instance3D));
		}

		ManagerPipeline::singleton().addFunc(
			HGE_shader_3Dinstance_holder::pipelineName(),
			|renderpass, transparency| {
				EnginePipelines::singleton().pipelineCreation(
					names::instance3D,
					transparency,
					renderpass.clone(),
					HGEsubpassName::WORLDSOLID.getSubpassID(),
					[HGE_shader_3Dinstance::per_vertex(), HGE_shader_3Dinstance_data::per_instance()],
				)
			},
			PrimitiveTopology::TriangleList,
			true,
		);

		return Ok(());
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Vertex, BufferContents)]
pub struct HGE_shader_3Dinstance_data
{
	#[format(R32G32B32_SFLOAT)]
	pub instance_offset: [f32; 3],
	#[format(R32G32B32_SFLOAT)]
	pub instance_scale: [f32; 3],
	#[format(R32G32B32_SFLOAT)]
	pub instance_rotation: [f32; 3],
	#[format(R32G32B32A32_SFLOAT)]
	pub instance_color: [f32; 4],
	#[format(R32_UINT)]
	pub instance_texture: u32,
	#[format(R32G32_SFLOAT)]
	pub instance_texcoord_offset: [f32; 2],
}

impl Default for HGE_shader_3Dinstance_data
{
	fn default() -> Self
	{
		HGE_shader_3Dinstance_data {
			instance_offset: [0.0, 0.0, 0.0],
			instance_scale: [1.0, 1.0, 1.0],
			instance_rotation: [0.0, 0.0, 0.0],
			instance_color: [1.0, 1.0, 1.0, 1.0],
			instance_texture: 0,
			instance_texcoord_offset: [0.0, 0.0],
		}
	}
}

struct HGE_shader_3Dinstance_subholder
{
	_model: ArcSwap<ShaderDrawerImplStruct<Box<dyn IntoVertexted<HGE_shader_3Dinstance> + Send + Sync>>>,
	_instance: DashMap<String, HGE_shader_3Dinstance_data>,
	_cacheDraw: ArcSwapOption<ShaderStructCacheInstanced<HGE_shader_3Dinstance, HGE_shader_3Dinstance_data>>,
}

impl HGE_shader_3Dinstance_subholder
{
	pub fn compileData(&self) -> (Vec<HGE_shader_3Dinstance>, Vec<u32>, bool)
	{
		let mut vertex = Vec::new();
		let mut indices = Vec::new();
		let mut atleastone = false;

		let mut stop = false;
		let mut tmpvertex = Vec::new();
		let binding = self._model.load();
		for x in &binding.vertex
		{
			let Some(unwraped) = x.IntoVertexted(false)
			else
			{
				stop = true;
				break;
			};
			tmpvertex.push(unwraped);
		}

		if (!stop)
		{
			vertex.append(&mut tmpvertex);
			for x in &binding.indices
			{
				indices.push(*x);
			}
			atleastone = true;
		}

		return (vertex, indices, atleastone);
	}

	pub fn cache_reset(&self)
	{
		self._cacheDraw.store(None);
	}

	pub fn cache_update(&self, vertex: Vec<HGE_shader_3Dinstance>, indices: Vec<u32>, instance: Vec<HGE_shader_3Dinstance_data>)
	{
		let mut newcache = ShaderStructCacheInstanced::new();
		newcache.update(vertex, indices, instance);
		self._cacheDraw.store(Some(Arc::new(newcache)));
	}

	pub fn cache_draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, pipelinename: String)
	{
		if let Some(cache) = &*self._cacheDraw.load()
		{
			cache.draw(cmdBuilder, pipelinename);
		}
	}
}

///// HOLDER
pub struct HGE_shader_3Dinstance_holder
{
	_haveUpdate: AtomicBool,
	_datas: DashMap<String, HGE_shader_3Dinstance_subholder>,
}

impl HGE_shader_3Dinstance_holder
{
	pub fn addInstance(&self, modelname: impl Into<String>, instancename: impl Into<String>, instance: HGE_shader_3Dinstance_data)
	{
		let modelname = modelname.into();
		if let Some(this) = self._datas.get(&modelname)
		{
			this._instance.insert(instancename.into(), instance);
			self._haveUpdate.store(true, Ordering::Release);
		}
	}

	pub fn removeInstance(&self, modelname: impl Into<String>, instancename: impl Into<String>)
	{
		let modelname = modelname.into();
		if let Some(this) = self._datas.get(&modelname)
		{
			this._instance.remove(&instancename.into());
			self._haveUpdate.store(true, Ordering::Release);
		}
	}

	pub fn importModel(&self, modelname: impl Into<String>, mut model: ShaderDrawerImplStruct<impl IntoVertexted<HGE_shader_3Dinstance> + Send + Sync + 'static>)
	{
		let modelname = modelname.into();
		if (!self._datas.contains_key(&modelname))
		{
			self._datas.insert(
				modelname.clone(),
				HGE_shader_3Dinstance_subholder {
					_model: Default::default(),
					_instance: Default::default(),
					_cacheDraw: Default::default(),
				},
			);
		}

		if let Some(this) = self._datas.get(&modelname)
		{
			let mut newvertex = Vec::new();

			for x in model.vertex.drain(0..)
			{
				let tmp: Box<dyn IntoVertexted<HGE_shader_3Dinstance> + Send + Sync> = Box::new(x);
				newvertex.push(tmp);
			}

			this._model.store(Arc::new(ShaderDrawerImplStruct {
				vertex: newvertex,
				indices: model.indices.clone(),
			}));
		}
	}
}

impl ShaderStructHolder for HGE_shader_3Dinstance_holder
{
	fn init() -> Self
	{
		Self {
			_haveUpdate: AtomicBool::new(false),
			_datas: Default::default(),
		}
	}

	fn pipelineName() -> String
	{
		names::instance3D.to_string()
	}

	fn pipelineNameResolve(&self) -> String
	{
		Self::pipelineName()
	}

	fn reset(&self)
	{
		self._datas.clear();
		self._haveUpdate.store(false, Ordering::Release);
	}

	fn update(&self)
	{
		if (self._haveUpdate.compare_exchange(true, false, Ordering::Release, Ordering::Acquire).is_err())
		{
			return;
		}
		let mut oneismissing = false;

		self._datas
			.iter()
			.filter(|selfdata| {
				let tmp = selfdata._model.load();
				tmp.vertex.len() != 0 && tmp.indices.len() != 0
			})
			.for_each(|selfdata| {
				let (vertex, indices, atleastone) = selfdata.compileData();
				if (atleastone)
				{
					let instances = selfdata._instance.iter().map(|x| x.value().clone()).collect::<Vec<HGE_shader_3Dinstance_data>>();

					selfdata.cache_update(vertex, indices, instances);
				}
				else
				{
					oneismissing = true;
					selfdata.cache_reset();
				}
			});

		if (oneismissing)
		{
			self._haveUpdate.store(true, Ordering::Release);
		}
	}

	fn draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, pipelinename: String)
	{
		let Some(pipelineLayout) = ManagerPipeline::singleton().layoutGet(&pipelinename)
		else
		{
			return;
		};
		if (ManagerShaders::singleton().push_constants(names::instance3D, cmdBuilder, pipelineLayout.clone(), 0) == false)
		{
			return;
		}

		for setid in 0..3
		{
			let Some(descriptorCache) = ManagerTexture::singleton().descriptorSet_getVulkanCache(format!("HGE_set{}", setid))
			else
			{
				return;
			};
			HTraceError!(cmdBuilder.bind_descriptor_sets(PipelineBindPoint::Graphics, pipelineLayout.clone(), setid, descriptorCache,));
		}

		self._datas.iter().for_each(|selfdata| {
			selfdata.cache_draw(cmdBuilder, pipelinename.clone());
		});
	}
}
