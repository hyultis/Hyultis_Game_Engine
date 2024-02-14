use std::sync::{Arc, OnceLock};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use dashmap::mapref::one::RefMut;
use Htrace::HTracer::HTracer;
use parking_lot::RwLock;
use singletonThread::SingletonThread;
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use crate::Models3D::chunk::chunk;
use crate::Shaders::StructAllCache::StructAllCache;

pub struct ManagerModels
{
	_chunks: DashMap<[i32; 3], chunk>,
	_cache: ArcSwap<StructAllCache>,
	_active: ArcSwap<Vec<[i32; 3]>>,
	_threadUpdate: RwLock<SingletonThread>,
}

static SINGLETON: OnceLock<ManagerModels> = OnceLock::new();

impl ManagerModels
{
	fn new() -> ManagerModels
	{
		let thread = SingletonThread::new(||{
			HTracer::threadSetName("ManagerModel_ST");
			ManagerModels::singleton().ModelsUpdate();
		});
		
		ManagerModels
		{
			_chunks: DashMap::new(),
			_cache: ArcSwap::new(Arc::new(StructAllCache::new())),
			_active: ArcSwap::new(Arc::new(Vec::new())),
			_threadUpdate: RwLock::new(thread),
		}
	}
	
	pub fn singleton() -> &'static ManagerModels
	{
		return SINGLETON.get_or_init(|| {
			ManagerModels::new()
		});
	}
	
	pub fn get<'a>(&'a self, pos: [i32; 3]) -> RefMut<'a, [i32; 3], chunk>
	{
		if (!self._chunks.contains_key(&pos))
		{
			self._chunks.insert(pos, chunk::new(pos[0], pos[1], pos[2]));
		}
		self._chunks.get_mut(&pos).unwrap()
	}
	
	pub fn active_chunk_add(&self, mut add: Vec<[i32; 3]>)
	{
		let mut old = self._active.load_full().to_vec();
		old.append(&mut add);
		self._active.swap(Arc::new(old));
	}
	
	pub fn active_chunk_resetAndAdd(&self, add: Vec<[i32; 3]>)
	{
		self._active.swap(Arc::new(add));
	}
	
	pub fn active_chunk_get(&self) -> Arc<Vec<[i32; 3]>>
	{
		self._active.load_full()
	}
	
	pub fn all_chunk_reset(&self)
	{
		self.active_chunk_resetAndAdd(vec![]);
		self._chunks.retain(|_,_|{false});
	}
	
	pub fn ModelsDraw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>)
	{
		//println!("ModelsDraw");
		self._cache.load().holderDraw(cmdBuilder);
		//println!("model duration : {:.>10}",instant.elapsed().as_nanos());
		self._threadUpdate.write().thread_launch();
	}
	
	//////// PRIVATE
	
	pub fn ModelsUpdate(&self)
	{
		let mut cache = StructAllCache::new();
		let mut haveUpdate = false;
		//let mut lastloadednb = 0;
		for pos in self._active.load().iter()
		{
			if let Some(mut chunk) = self._chunks.get_mut(pos)
			{
				//lastloadednb+=chunk.len();
				if(chunk.cacheUpdate())
				{
					haveUpdate=true;
				}
				cache.append(chunk.cache_get());
			}
		}
		
		if(haveUpdate)
		{
			cache.holderUpdate();
			self._cache.swap(Arc::new(cache));
		}
	}
}
