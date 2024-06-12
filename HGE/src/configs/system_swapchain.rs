use vulkano::swapchain::PresentMode;

#[derive(Copy, Clone, Debug)]
pub struct HGEconfig_system_swapchain
{
	pub presentmode: PresentMode,
	pub fpslimiter: u8,
}

impl HGEconfig_system_swapchain
{
	pub fn getPresentModeString(&self) -> String
	{
		return match self.presentmode {
			PresentMode::Immediate => "Immediate",
			PresentMode::Mailbox => "Mailbox",
			_ => "Fifo"
		}.to_string()
	}
	
	pub fn setPresentModeString(&mut self, presentmode: String)
	{
		self.presentmode = match presentmode.as_str() {
			"Immediate" => PresentMode::Immediate,
			"Mailbox" => PresentMode::Mailbox,
			_ => PresentMode::Fifo
		}
	}
}

impl Default for HGEconfig_system_swapchain
{
	fn default() -> Self {
		Self {
			presentmode: PresentMode::Fifo,
			fpslimiter: 0
		}
	}
}
