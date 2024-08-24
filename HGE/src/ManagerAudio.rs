use std::io::Cursor;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use ahash::HashMap;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use Htrace::{HTrace, namedThread};
use parking_lot::{Mutex, RwLock};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use crate::assetStreamReader::assetManager;

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub enum audio_channel
{
	MUSIC,
	EFFECT
}

struct audio_queue
{
	name: String,
	channel: audio_channel,
	volume: f32
}

struct audio_storage
{
	path: String,
	level: f32
}

struct audio_channel_sink
{
	pub sink: Sink,
	pub content_level: f32
}

impl audio_channel_sink
{
	pub fn new(handle: &OutputStreamHandle) -> Self
	{
		audio_channel_sink {
			sink: Sink::try_new(&handle).unwrap(),
			content_level: 1.0,
		}
	}
}

pub struct deviceStream
{
	pub stream: OutputStream,
	pub handle: OutputStreamHandle,
	_channel: HashMap<audio_channel, Vec<audio_channel_sink>>,
	_channelRun: HashMap<audio_channel, u8>,
	_isPaused: bool
}

impl deviceStream
{
	fn addChannel(&mut self, channel: audio_channel)
	{
		match self._channel.get_mut(&channel) {
			None => {
				self._channel.insert(channel, vec![audio_channel_sink::new(&self.handle)]);
			}
			Some(data) => {
				data.push(audio_channel_sink::new(&self.handle));
			}
		};
	}
	
	fn checkPause(&mut self, globalpause: bool)
	{
		if (globalpause == self._isPaused)
		{
			return;
		}
		
		if (globalpause)
		{
			self._channel.iter_mut().for_each(|(_, sinks)| {
				sinks.iter_mut().for_each(|sink| {
					sink.sink.pause();
				});
			});
		} else {
			self._channel.iter_mut().for_each(|(_, sinks)| {
				sinks.iter_mut().for_each(|sink| {
					sink.sink.play();
				});
			});
		}
		self._isPaused = globalpause;
	}
}

pub struct ManagerAudio
{
	_loadedSound: DashMap<String, audio_storage>,
	_queueEffect: Mutex<Vec<audio_queue>>,
	_musicList: RwLock<Vec<String>>,
	
	_music_nextplayid: ArcSwap<usize>,
	_music_lastplay: ArcSwap<Instant>,
	_music_nextStart: ArcSwap<Duration>,
	_music_nextStartOk: ArcSwap<bool>,
	_music_swap: ArcSwap<bool>,
	// false = sink n°0, true = sink n°1
	_music_swapval: ArcSwap<f32>,
	_music_speed: ArcSwap<f32>,
	_music_update: ArcSwap<bool>,
	
	_audiolevel_global: ArcSwap<f32>,
	_audiolevel_music: ArcSwap<f32>,
	_audiolevel_effect: ArcSwap<f32>,
	
	_global_pause: ArcSwap<bool>,
	_music_pause: ArcSwap<bool>,
	_music_pause_state: ArcSwap<bool>,
	_effect_clear: ArcSwap<bool>
}

static SINGLETON: OnceLock<ManagerAudio> = OnceLock::new();

impl ManagerAudio
{
	pub fn singleton() -> &'static Self
	{
		return SINGLETON.get_or_init(|| {
			Self {
				_loadedSound: Default::default(),
				_queueEffect: Mutex::new(vec![]),
				_musicList: RwLock::new(vec![]),
				
				_music_nextplayid: ArcSwap::new(Arc::new(0)),
				_music_lastplay: ArcSwap::new(Arc::new(Instant::now())),
				_music_nextStart: ArcSwap::new(Arc::new(Duration::from_secs(0))),
				_music_nextStartOk: ArcSwap::new(Arc::new(true)),
				_music_swap: ArcSwap::new(Arc::new(false)),
				_music_swapval: ArcSwap::new(Arc::new(0.0)),
				_music_speed: ArcSwap::new(Arc::new(1.0)),
				_music_update: ArcSwap::new(Arc::new(false)),
				
				_audiolevel_global: ArcSwap::new(Arc::new(1.0)),
				_audiolevel_music: ArcSwap::new(Arc::new(1.0)),
				_audiolevel_effect: ArcSwap::new(Arc::new(1.0)),
				
				_global_pause: ArcSwap::new(Arc::new(false)),
				_music_pause: ArcSwap::new(Arc::new(false)),
				_music_pause_state: ArcSwap::new(Arc::new(false)),
				_effect_clear: ArcSwap::new(Arc::new(false)),
			}
		});
	}
	
	pub fn globalPause_set(&self, pause: bool, stream: &mut Option<deviceStream>)
	{
		if let Some(stream) = stream
		{
			self._global_pause.swap(Arc::new(pause));
			stream.checkPause(pause);
		} else {
			self._global_pause.swap(Arc::new(true));
		}
	}
	
	pub fn globalLevel_set(&self, level: f32)
	{
		self._audiolevel_global.swap(Arc::new(level.clamp(0.0, 1.0)));
	}
	
	pub fn globalLevel_get(&self) -> f32
	{
		*self._audiolevel_global.load_full()
	}
	
	pub fn music_ispaused(&self) -> bool
	{
		**self._music_pause.load()
	}
	
	pub fn music_pause(&self)
	{
		self._music_pause.swap(Arc::new(true));
	}
	
	pub fn music_unpause(&self)
	{
		self._music_pause.swap(Arc::new(false));
	}
	
	pub fn musicLevel_set(&self, level: f32)
	{
		self._audiolevel_music.swap(Arc::new(level.clamp(0.0, 1.0)));
		self._music_update.swap(Arc::new(true));
	}
	
	pub fn musicLevel_get(&self) -> f32
	{
		**self._audiolevel_music.load()
	}
	
	pub fn effectLevel_set(&self, level: f32)
	{
		self._audiolevel_effect.swap(Arc::new(level.clamp(0.0, 1.0)));
	}
	
	pub fn effectLevel_get(&self) -> f32
	{
		**self._audiolevel_effect.load()
	}
	
	pub fn getDeviceStream() -> Option<deviceStream>
	{
		if let Ok((stream, stream_handle)) = OutputStream::try_default()
		{
			let mut tmp = deviceStream {
				stream,
				handle: stream_handle,
				_channel: Default::default(),
				_channelRun: Default::default(),
				_isPaused: false,
			};
			
			for _ in 0..1
			{
				tmp.addChannel(audio_channel::MUSIC);
			}
			
			for _ in 0..8
			{
				tmp.addChannel(audio_channel::EFFECT);
			}
			
			return Some(tmp);
		}
		
		Self::singleton()._global_pause.swap(Arc::new(true));
		return None;
	}
	
	pub fn loadFile(&self, name: impl Into<String>, path: impl Into<String>, localLevel: f32)
	{
		let name = name.into();
		let path = path.into();
		
		HTrace!("manager audio : loadFile {}",path);
		
		if assetManager::singleton().checkFile(path.clone())
		{
			self._loadedSound.insert(name, audio_storage {
				path: path,
				level: localLevel,
			});
		}
	}
	
	pub fn effect_play(&self, name: impl Into<String>)
	{
		self.effect_play_vol(name,1.0);
	}
	
	pub fn effect_play_vol(&self, name: impl Into<String>, volume: f32)
	{
		#[cfg(target_os = "android")]
		{
			// on android, cancel playing of effect too
			if (*self._music_pause.load_full())
			{
				return;
			}
		}
		
		
		let name = name.into();
		let _ = namedThread!(move ||{
			Self::singleton()._queueEffect.lock().push(audio_queue {
				name: name,
				channel: audio_channel::EFFECT,
				volume: volume.clamp(0.0,2.0)
			});
		});
	}
	
	pub fn effect_clearAll(&self)
	{
		self._effect_clear.swap(Arc::new(true));
	}
	
	pub fn music_add(&self, name: impl Into<String>)
	{
		let name = name.into();
		let _ = namedThread!(move ||{
			Self::singleton()._musicList.write().push(name);
		});
	}
	
	pub fn music_changeSpeed(&self, speed: f32)
	{
		self._music_speed.swap(Arc::new(speed.clamp(0.0, 2.0)));
		self._music_update.swap(Arc::new(true));
	}
	
	pub fn resumeAllAudio(&self, stream: &mut Option<deviceStream>)
	{
		if let Some(stream) = stream
		{
			stream._channel.iter().for_each(|(_, list)| {
				list.iter().for_each(|sink| {
					if (!sink.sink.empty())
					{
						sink.sink.play();
					}
				})
			});
		}
	}
	
	pub fn executeAudioQueue(&self, stream: &mut Option<deviceStream>)
	{
		if let Some(stream) = stream
		{
			if (*self._music_pause.load_full() != *self._music_pause_state.load_full())
			{
				if (*self._music_pause.load_full())
				{
					stream._channel.get_mut(&audio_channel::MUSIC).unwrap().iter_mut().for_each(|sink| {
						if (!sink.sink.empty())
						{
							sink.sink.stop();
						}
					});
				}
				let _ = *self._music_pause_state.swap(self._music_pause.load_full().clone());
			}
			
			if (!*self._music_pause_state.load_full())
			{
				let musiclist = self._musicList.read();
				if (musiclist.len() != 0)
				{
					if stream._channel.get(&audio_channel::MUSIC).unwrap().iter().find(|x| { !x.sink.empty() }).is_none()
					{
						let mut nextid = *self._music_nextplayid.load_full();
						if (nextid > musiclist.len())
						{
							nextid = 0;
						}
						
						if let Some(keymusic) = musiclist.get(nextid)
						{
							if let Some(storage) = self._loadedSound.get(keymusic)
							{
								//let file = BufReader::new(File::open(storage.path.clone()).unwrap());
								if let Some(file) = assetManager::singleton().readFile(storage.path.clone())
								{
									if let Ok(source) = Decoder::new(file)
									{
										source.total_duration();
										self.sinkplay(stream, &audio_queue {
											name: keymusic.clone(),
											channel: audio_channel::MUSIC,
											volume: 1.0
										}, source, **self._audiolevel_music.load(), storage.level);
									}
								}
							}
						}
						
						self._music_nextplayid.swap(Arc::new(nextid + 1));
					}
					
					if (**self._music_update.load())
					{
						stream._channel.get(&audio_channel::MUSIC).unwrap().iter().for_each(|x| {
							x.sink.set_speed(**self._music_speed.load());
							x.sink.set_volume(**self._audiolevel_global.load() * **self._audiolevel_music.load() * x.content_level);
						});
					}
				}
			}
			
			// cannot work because symphonia in rodio is borked, keep it for later
			/*if(self._music_lastplay.load().elapsed() > self._music_nextStart.load_full().checked_sub(Duration::from_millis(500)).unwrap_or(Duration::from_secs(0)))
			{
				let mut lastid = *self._music_lastplayid.load_full() + 1;
				let musiclist = self._musicList.read();
				if(lastid > musiclist.len())
				{
					lastid=0;
				}
				
				if let Some(keymusic) = musiclist.get(lastid)
				{
					println!("plausing music : {}",keymusic);
					if let Some(path) = self._loadedSound.get(keymusic)
					{
						let file = BufReader::new(File::open(path.value()).unwrap());
						if let Ok(source) = Decoder::new(file)
						{
							println!("ddd {:?}",source.total_duration());
							//durationofmusic = source.total_duration().unwrap();
							/*self.sinkplay(stream, &audio_queue{
								name: keymusic.clone(),
								channel: audio_channel::MUSIC,
								local_level: 1.0,
							}, source);*/
						}
					}
					
					
					self._music_lastplay.swap(Arc::new(Instant::now()));
					self._music_nextStart.swap(Arc::new(durationofmusic));
					self._music_lastplayid.swap(Arc::new(lastid));
				}
			}*/
			
			if (*self._effect_clear.load_full())
			{
				self._effect_clear.swap(Arc::new(false));
				stream._channel.get(&audio_channel::EFFECT).unwrap().iter().for_each(|x| {
					x.sink.clear();
				});
			}
			
			let toexecute: Vec<audio_queue> = self._queueEffect.lock().drain(0..).collect();
			for data in toexecute
			{
				if let Some(storage) = self._loadedSound.get(&data.name)
				{
					//let file = BufReader::new(File::open(storage.path.clone()).unwrap());
					if let Some(file) = assetManager::singleton().readFile(storage.path.clone())
					{
						if let Ok(source) = Decoder::new(file)
						{
							self.sinkplay(stream, &data, source, **self._audiolevel_effect.load(), storage.level);
						}
					}
				}
			}
		}
	}
	
	fn sinkplay(&self, stream: &mut deviceStream, data: &audio_queue, source: Decoder<Cursor<Vec<u8>>>, glevel: f32, storage_level: f32)
	{
		let mut last = *stream._channelRun.get(&data.channel).unwrap_or(&0) as usize;
		
		let Some(test) = stream._channel.get_mut(&data.channel) else {
			return;
		};
		let max = test.len();
		if let Some(sink) = test.get_mut(last)
		{
			if (!sink.sink.empty())
			{
				sink.sink.stop();
			}
			sink.content_level = storage_level * data.volume;
			sink.sink.set_volume(*self._audiolevel_global.load_full() * glevel * storage_level * data.volume);
			sink.sink.append(source);
			if (sink.sink.is_paused())
			{
				sink.sink.play();
			}
		}
		
		last += 1;
		if (last >= max)
		{
			last = 0;
		}
		
		stream._channelRun.insert(data.channel.clone(), last as u8);
	}
}
