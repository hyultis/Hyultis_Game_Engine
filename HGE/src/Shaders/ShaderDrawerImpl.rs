pub trait ShaderDrawerImpl //: DynClone + Downcast
{
	fn cache_mustUpdate(&self) -> bool;
	fn cache_submit(&mut self);
	fn cache_remove(&mut self);
}

//impl_downcast!(ShaderDrawerImpl);
//dyn_clone::clone_trait_object!(ShaderDrawerImpl);

pub trait ShaderDrawerImplReturn<A>: ShaderDrawerImpl
{
	fn cache_get(&mut self) -> Option<ShaderDrawerImplStruct<A>>;
}

#[derive(Clone)]
pub struct ShaderDrawerImplStruct<A>
{
	pub vertex: Vec<A>,
	pub indices: Vec<u32>
}

impl<A> ShaderDrawerImplStruct<A>
{
	pub fn combine(&mut self, other: &mut ShaderDrawerImplStruct<A>)
	{
		let oldindices = self.vertex.len() as u32;
		other.vertex.drain(0..).for_each(|x| {
			self.vertex.push(x);
		});
		other.indices.drain(0..).for_each(|x| {
			self.indices.push(x + oldindices);
		});
	}
}

impl<A> Default for ShaderDrawerImplStruct<A>
{
	fn default() -> Self {
		ShaderDrawerImplStruct{
			vertex: Vec::new(),
			indices: Vec::new(),
		}
	}
}
