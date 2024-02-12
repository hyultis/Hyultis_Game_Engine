use std::collections::{HashMap};
use parking_lot::RwLock;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

pub struct Inputs
{
	_keys: HashMap<KeyCode,ElementState>,
	_keysSteal: RwLock<HashMap<KeyCode,bool>>
}

impl Inputs
{
	pub fn new() -> Inputs
	{
		return Inputs
		{
			_keys: Default::default(),
			_keysSteal: Default::default()
		};
	}
	
	pub fn updateFromKeyboard(&mut self, key: KeyCode, state: ElementState)
	{
		//println!("Pressed : {:?}",key);
		//self._keys.remove(&key);
		self._keys.insert(key,state);
		if(state==ElementState::Released)
		{
			self._keysSteal.write().insert(key,false);
		}
	}
	
	// get if the key is actually pressed OR released at the moment
	pub fn getKeyboardState(&self,key: KeyCode) -> ElementState
	{
		return match self._keys.get(&key) {
			None => {
				ElementState::Released
			},
			Some(x) => *x
		};
	}
	
	// the pressed value is valid only one time (use getKeyboardState for continuous check)
	pub fn getKeyboardStateAndSteal(&self,key: KeyCode) -> ElementState
	{
		let mut keystate= self._keys.get(&key).unwrap_or(&ElementState::Released).clone();
		if(keystate==ElementState::Pressed)
		{
			let mut keystealBinding = self._keysSteal.write();
			let keysteal = keystealBinding.get(&key);
			if(keysteal.is_some() && *keysteal.unwrap()==true)
			{
				keystate = ElementState::Released;
			}
			else
			{
				keystealBinding.insert(key,true);
			}
		}
		return keystate;
	}
}
