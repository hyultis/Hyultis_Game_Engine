use std::fmt::{Debug, Formatter};
use uuid::Uuid;
use crate::Shaders::ShaderDrawer::ShaderDrawer_Manager;

#[derive(Debug, Copy, Clone, Eq, PartialOrd, PartialEq)]
pub enum cacheInfos_state
{
	PRESENT,
	ABSENT
}

#[derive(Copy, Clone)]
pub struct cacheInfos
{
	needUpdate: bool,
	uuid: Uuid,
	state: cacheInfos_state
}

impl cacheInfos
{
	pub fn setPresent(&mut self)
	{
		self.state = cacheInfos_state::PRESENT;
	}
	
	pub fn setAbsent(&mut self)
	{
		self.state = cacheInfos_state::ABSENT;
	}
	
	pub fn isPresent(&self) -> bool
	{
		self.state == cacheInfos_state::PRESENT
	}
	
	pub fn isAbsent(&self) -> bool
	{
		self.state == cacheInfos_state::ABSENT
	}
	
	pub fn setNeedUpdate(&mut self, val: bool)
	{
		self.needUpdate = val;
	}
	
	pub fn isNeedUpdate(&self) -> bool
	{
		self.needUpdate
	}
	
	/// return true if absent OR needupdate
	pub fn isNotShow(&self) -> bool
	{
		self.needUpdate || self.isAbsent()
	}
}

impl Into<Uuid> for cacheInfos
{
	fn into(self) -> Uuid {
		self.uuid
	}
}

impl Into<cacheInfos_state> for cacheInfos
{
	fn into(self) -> cacheInfos_state {
		self.state
	}
}

impl Debug for cacheInfos
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("cacheInfos")
			.field("needUpdate", &self.needUpdate)
			.field("state", &self.state)
			.field("uuid", &self.uuid)
			.finish()
	}
}

impl Default for cacheInfos
{
	fn default() -> Self {
		Self{
			needUpdate: false,
			uuid: ShaderDrawer_Manager::uuid_generate(),
			state: cacheInfos_state::ABSENT,
		}
	}
}
