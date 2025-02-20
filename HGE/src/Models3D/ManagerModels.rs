use crate::Models3D::chunk::chunk;
use arc_swap::ArcSwap;
use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use parking_lot::RwLock;
use singletonThread::SingletonThread;
use std::sync::{Arc, OnceLock};
use Htrace::HTracer::HTracer;

pub struct ManagerModels
{
	_chunks: DashMap<[i32; 3], chunk>,
	_active: ArcSwap<Vec<[i32; 3]>>,
	_threadUpdate: RwLock<SingletonThread>,
}

static SINGLETON: OnceLock<ManagerModels> = OnceLock::new();

impl ManagerModels
{
	fn new() -> ManagerModels
	{
		let thread = SingletonThread::new(|| {
			HTracer::threadSetName("ManagerModel_ST");
			ManagerModels::singleton().ModelsUpdate();
		});

		ManagerModels {
			_chunks: DashMap::new(),
			_active: ArcSwap::new(Arc::new(Vec::new())),
			_threadUpdate: RwLock::new(thread),
		}
	}

	pub fn singleton() -> &'static ManagerModels
	{
		return SINGLETON.get_or_init(|| ManagerModels::new());
	}

	pub fn get(&self, pos: [i32; 3]) -> RefMut<[i32; 3], chunk>
	{
		if (!self._chunks.contains_key(&pos))
		{
			self._chunks.insert(pos, chunk::new(pos[0], pos[1], pos[2]));
		}
		self._chunks.get_mut(&pos).unwrap()
	}

	pub fn active_chunk_add(&self, mut add: Vec<[i32; 3]>)
	{
		let mut old = self._active.load().to_vec();
		old.append(&mut add);
		self._active.store(Arc::new(old));
	}

	pub fn active_chunk_resetAndAdd(&self, add: Vec<[i32; 3]>)
	{
		let old = self._active.swap(Arc::new(add.clone()));
		let old = Arc::unwrap_or_clone(old);

		// remove stuff that no here anymore
		for x in &old
		{
			if (!add.contains(&x))
			{
				if let Some(chunk) = self._chunks.get_mut(x)
				{
					chunk.cache_remove();
				}
			}
		}

		// force add stuff that just been added back
		for x in &add
		{
			if (!old.contains(&x))
			{
				if let Some(chunk) = self._chunks.get_mut(x)
				{
					chunk.cacheForceUpdate();
				}
			}
		}
	}

	pub fn active_chunk_get(&self) -> Arc<Vec<[i32; 3]>>
	{
		self._active.load().clone()
	}

	pub fn all_chunk_reset(&self)
	{
		self.active_chunk_resetAndAdd(vec![]);
		self._chunks.retain(|_, x| {
			x.cache_remove();
			false
		});
	}

	pub fn tickUpdate(&self)
	{
		if let Some(mut t) = self._threadUpdate.try_write()
		{
			t.thread_launch();
		}
	}

	//////// PRIVATE

	pub fn ModelsUpdate(&self)
	{
		for pos in self._active.load().iter()
		{
			if let Some(mut chunk) = self._chunks.get_mut(pos)
			{
				chunk.cache_checkupdate();
			}
		}
	}
}
