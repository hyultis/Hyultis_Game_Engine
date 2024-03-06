use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::DynClone;
use crate::components::event::event_trait;
use crate::Shaders::ShaderDrawerImpl::ShaderDrawerImpl;

pub trait chunk_content: ShaderDrawerImpl + Send + Sync + DynClone + event_trait + Downcast
{}

impl_downcast!(chunk_content);
dyn_clone::clone_trait_object!(chunk_content);
