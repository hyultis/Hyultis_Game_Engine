use crate::ManagerBuilder::ManagerBuilder;
use crate::Shaders::ShaderStruct::ShaderStructHolder_utils;
use vulkano::buffer::{BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, SecondaryAutoCommandBuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};

pub struct ShaderStructCacheInstanced<T, I>
	where
		T: BufferContents,
		I: BufferContents
{
	_cacheDatasMem: Option<Subbuffer<[T]>>,
	_cacheIndicesMem: Option<Subbuffer<[u32]>>,
	_cacheIndicesLen: u32,
	_cacheInstanceMem: Option<Subbuffer<[I]>>,
	_cacheInstanceLen: u32,
}

impl<T, I> Default for ShaderStructCacheInstanced<T, I>
	where
		T: BufferContents,
		I: BufferContents
{
	fn default() -> Self {
		Self::new()
	}
}

impl<T, I> ShaderStructCacheInstanced<T, I>
	where
		T: BufferContents,
		I: BufferContents
{
	pub fn new() -> Self
	{
		Self {
			_cacheDatasMem: Default::default(),
			_cacheIndicesMem: Default::default(),
			_cacheIndicesLen: Default::default(),
			_cacheInstanceMem: Default::default(),
			_cacheInstanceLen: Default::default(),
		}
	}
	
	pub fn update(&mut self, vertex: Vec<T>, indices: Vec<u32>, intance: Vec<I>)
	{
		ShaderStructHolder_utils::updateBuffer(vertex, &mut self._cacheDatasMem, BufferCreateInfo {
			usage: BufferUsage::VERTEX_BUFFER,
			..Default::default()
		}, AllocationCreateInfo {
			memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
				| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
			..Default::default()
		});
		
		self._cacheIndicesLen = ShaderStructHolder_utils::updateBuffer(indices, &mut self._cacheIndicesMem, BufferCreateInfo {
			usage: BufferUsage::INDEX_BUFFER,
			..Default::default()
		}, AllocationCreateInfo {
			memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
				| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
			..Default::default()
		});
		
		self._cacheInstanceLen = ShaderStructHolder_utils::updateBuffer(intance, &mut self._cacheInstanceMem, BufferCreateInfo {
			usage: BufferUsage::VERTEX_BUFFER,
			..Default::default()
		}, AllocationCreateInfo {
			memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
				| MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
			..Default::default()
		})
	}
	
	pub fn draw(&self, cmdBuilder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>, pipelinename: String)
	{
		if (self._cacheIndicesLen == 0 || self._cacheInstanceLen == 0) { return }
		let Some(datamem) = &self._cacheDatasMem else { return };
		let Some(indicemem) = &self._cacheIndicesMem else { return };
		let Some(instancemem) = &self._cacheInstanceMem else { return };
		
		ManagerBuilder::builderAddPipeline(cmdBuilder, &pipelinename);
		
		cmdBuilder
			.bind_vertex_buffers(0, (datamem.clone(), instancemem.clone())).unwrap()
			.bind_index_buffer(indicemem.clone()).unwrap()
			.draw_indexed(self._cacheIndicesLen, self._cacheInstanceLen, 0, 0, 0).unwrap();
		
		if ManagerBuilder::builderAddPipelineTransparency(cmdBuilder, &pipelinename)
		{
			cmdBuilder
				.bind_vertex_buffers(0, (datamem.clone(), instancemem.clone())).unwrap()
				.bind_index_buffer(indicemem.clone()).unwrap()
				.draw_indexed(self._cacheIndicesLen, self._cacheInstanceLen, 0, 0, 0).unwrap();
		}
	}
}
