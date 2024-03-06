use std::sync::{Arc, OnceLock};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use Htrace::HTracer::HTracer;
use Htrace::TSpawner;
use parking_lot::RwLock;
use singletonThread::SingletonThread;
use crate::components::event::{event_trait, event_type};
use crate::Interface::UiPage::UiPage;

pub struct ManagerInterface
{
	_pageArray: DashMap<String, UiPage>,
	_activePage: ArcSwap<String>,
	_pageChanged: ArcSwap<bool>,
	_threadEachTickUpdate: RwLock<SingletonThread>,
	_threadEachSecondUpdate: RwLock<SingletonThread>,
}

static SINGLETON: OnceLock<ManagerInterface> = OnceLock::new();

impl ManagerInterface
{
	fn new() -> ManagerInterface
	{
		let threadEachTick = SingletonThread::new(||{
			HTracer::threadSetName("ManagerInterface_ST");
			ManagerInterface::singleton().EachTickUpdate();
		});
		let mut threadEachSecond = SingletonThread::new(||{
			HTracer::threadSetName("ManagerInterface_ST");
			ManagerInterface::singleton().EachSecondUpdate();
		});
		threadEachSecond.setDuration_FPS(1);
		
		return ManagerInterface {
			_pageArray: Default::default(),
			_activePage: ArcSwap::new(Arc::new("default".to_string())),
			_pageChanged: ArcSwap::new(Arc::new(false)),
			_threadEachTickUpdate: RwLock::new(threadEachTick),
			_threadEachSecondUpdate: RwLock::new(threadEachSecond),
		};
	}
	
	pub fn singleton() -> &'static ManagerInterface
	{
		return SINGLETON.get_or_init(|| {
			ManagerInterface::new()
		});
	}
	
	pub fn changeActivePage(&self, name: impl Into<String>)
	{
		let name = name.into();
		let _ = TSpawner!(move ||{
			let oldpage = {(*Self::singleton()._activePage.load_full()).clone()};
			if let Some(mut page) = Self::singleton()._pageArray.get_mut(&oldpage) {
				page.event_trigger(event_type::EXIT);
			}
			
			if let Some(mut page) = Self::singleton()._pageArray.get_mut(&name) {
				page.event_trigger(event_type::ENTER);
			}
			
			Self::singleton()._activePage.swap(Arc::new(name.clone()));
			Self::singleton()._pageChanged.swap(Arc::new(true));
			
			Self::singleton().page_resubmit();
			
			if let Some(mut page) = Self::singleton()._pageArray.get_mut(&oldpage) {
				page.event_trigger(event_type::IDLE);
			}
		});
	}
	
	pub fn getActivePage(&self) -> String
	{
		return self._activePage.load().to_string();
	}
	
	pub fn getActivePage_content(&self) -> Option<Ref<String, UiPage>>
	{
		let name = self.getActivePage();
		self._pageArray.get(&name)
	}
	
	pub fn mouseUpdate(&self, x: u16, y: u16, mouseClick: bool) -> bool
	{
		let page = self._pageArray.get_mut(self.getActivePage().as_str());
		if(page.is_none())
		{
			return false;
		}
		let mut page = page.unwrap();
		return page.eventMouse(x,y,mouseClick);
	}
	
	pub fn WindowRefreshed(&self)
	{
		self._pageArray.iter_mut().for_each(|mut page|{
			page.eventWinRefresh();
		});
	}
	
	pub fn UiPageAppend(&self, name: &str, page: UiPage)
	{
		self._pageArray.insert(name.to_string(),page);
	}
	
	pub fn UiPageUpdate(&self, name: &str, func: impl Fn(&mut UiPage))
	{
		if let Some(mut page) = self._pageArray.get_mut(name)
		{
			func(page.value_mut());
		}
	}
	
	pub fn tickUpdate(&self)
	{
		self._threadEachTickUpdate.write().thread_launch();
		self._threadEachSecondUpdate.write().thread_launch();
	}
	
	///////////// PRIVATE //////////////////
	
	fn EachTickUpdate(&self)
	{
		let mut tmp = vec![];
		if let Some(page) = self._pageArray.get(self.getActivePage().as_str())
		{
			tmp = page.subevent_gets(event_type::EACH_TICK);
		}
		
		if(tmp.len()==0)
		{
			return;
		}
		
		if let Some(mut page) = self._pageArray.get_mut(self.getActivePage().as_str())
		{
			page.subevent_trigger(tmp,event_type::EACH_TICK);
		}
	}
	
	fn EachSecondUpdate(&self)
	{
		let mut tmp = vec![];
		if let Some(page) = self._pageArray.get(self.getActivePage().as_str())
		{
			tmp = page.subevent_gets(event_type::EACH_SECOND);
		}
		
		if(tmp.len()==0)
		{
			return;
		}
		
		if let Some(mut page) = self._pageArray.get_mut(self.getActivePage().as_str())
		{
			page.subevent_trigger(tmp,event_type::EACH_SECOND);
		}
	}
	
	fn page_resubmit(&self)
	{
		if let Some(mut page) = self._pageArray.get_mut(self.getActivePage().as_str())
		{
			page.cache_resubmit();
		}
	}
}
