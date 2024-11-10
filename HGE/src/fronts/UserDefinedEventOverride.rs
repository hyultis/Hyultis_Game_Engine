use Htrace::HTraceError;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::WindowId;
use crate::fronts::winit::HGEwinit;
use crate::HGEMain::HGEMain;

pub trait UserDefinedEventOverride
{
	fn resumed(&mut self, eventloop: &ActiveEventLoop);
	fn suspended(&mut self, eventloop: &ActiveEventLoop);
	fn window_event(&mut self, eventloop: &ActiveEventLoop, event: WindowEvent, window_id: WindowId);
	fn device_event(&mut self, eventloop: &ActiveEventLoop, event: DeviceEvent, device_id: DeviceId);
	fn about_to_wait(&mut self, eventloop: &ActiveEventLoop);
}
