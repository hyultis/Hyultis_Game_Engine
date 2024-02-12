use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::DynClone;
use crate::components::event::event_trait;
use crate::Shaders::StructAllCache::StructAllCache;

pub trait chunk_content: Send + Sync + DynClone + event_trait + Downcast
{
	fn cache_isUpdated(&self) -> bool;
	fn cache_update(&mut self);
	fn cache_get(&self) -> &StructAllCache;
	
	fn pipeline(&self) -> String
	{
		"".to_string()
	}
}
impl_downcast!(chunk_content);
dyn_clone::clone_trait_object!(chunk_content);
