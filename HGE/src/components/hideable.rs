pub trait hideable
{
	fn hide(&mut self);
	fn show(&mut self);
	fn isShow(&self)->bool;
}
