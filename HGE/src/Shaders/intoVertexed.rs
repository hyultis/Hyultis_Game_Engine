use vulkano::pipeline::graphics::vertex_input::Vertex;

pub trait IntoVertexted<T>
	where T: Vertex
{
	fn IntoVertexted(&self, descriptorContext: bool) -> Option<T>;
}
