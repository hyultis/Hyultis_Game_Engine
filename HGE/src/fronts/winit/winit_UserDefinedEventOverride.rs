use crate::fronts::winit::front::HGEwinit;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

pub trait winit_UserDefinedEventOverride
{
	/// event when application is resumed by os
	fn resumed(&mut self, root: &mut HGEwinit, eventloop: &ActiveEventLoop);
	/// event when application is suspended by os
	fn suspended(&mut self, root: &mut HGEwinit, eventloop: &ActiveEventLoop);
	/// event when window event (not called for RedrawRequested)
	/// called for each window event, must be fast
	fn window_event(
		&mut self,
		root: &mut HGEwinit,
		eventloop: &ActiveEventLoop,
		event: &WindowEvent,
		window_id: WindowId,
	);
	/// event when device event
	/// called for each window event, must be fast
	fn device_event(
		&mut self,
		root: &mut HGEwinit,
		eventloop: &ActiveEventLoop,
		event: &DeviceEvent,
		device_id: DeviceId,
	);
	/// about to render, any UI computation must append here to not appear laggy by the user (just before RedrawRequested)
	fn about_to_render(&mut self, root: &mut HGEwinit, eventloop: &ActiveEventLoop);
	/// event after rendering, best place to launch thread stuff
	fn about_to_wait(&mut self, root: &mut HGEwinit, eventloop: &ActiveEventLoop);
}
