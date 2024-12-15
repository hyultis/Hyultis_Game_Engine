use sdl2::event::Event;

pub trait sdl_UserDefinedEventOverride
{
	/// event when application is resumed by os
	fn resumed(&mut self);
	/// event when application is suspended by os
	fn suspended(&mut self);
	/// event on sdl event, return if sdl loop stop (false = continue, true = engine exit)
	fn event(&mut self, eventloop: &Event) -> bool;
	/// about to render, any UI computation must append here to not appear laggy by the user (just before RedrawRequested)
	fn about_to_render(&mut self);
	/// event after rendering, best place to launch thread stuff
	fn about_to_wait(&mut self);
}
