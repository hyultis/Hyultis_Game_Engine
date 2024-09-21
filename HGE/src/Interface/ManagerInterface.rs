use std::sync::{Arc, OnceLock};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use Htrace::HTracer::HTracer;
use Htrace::namedThread;
use parking_lot::RwLock;
use singletonThread::SingletonThread;
use crate::components::event::{event_trait, event_type};
use crate::Interface::UiPage::UiPage;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub struct ManagerInterface
{
	_pageArray: DashMap<String, UiPage>,
	_activePage: ArcSwap<String>,
	_threadEachTickUpdate: RwLock<SingletonThread>,
	_threadEachSecondUpdate: RwLock<SingletonThread>,
	_threadRefreshwindows: RwLock<SingletonThread>,
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
		let threadRefreshwindows = SingletonThread::new(||{
			HTracer::threadSetName("ManagerInterface_WR");
			let Some(page) = Self::singleton()._pageArray.get(Self::singleton().getActivePage().as_str()) else {return};
			page.eventWinRefresh();
			
			let otherpage = Self::singleton()._pageArray.iter().filter(|x|x.key().ne(&Self::singleton().getActivePage())).map(|x|x.key().clone()).collect::<Vec<String>>();
			for x in otherpage
			{
				if let Some(page) = ManagerInterface::singleton()._pageArray.get(&x)
				{
					page.eventWinRefresh();
				}
			}
		});
		
		return ManagerInterface {
			_pageArray: Default::default(),
			_activePage: ArcSwap::new(Arc::new("default".to_string())),
			_threadEachTickUpdate: RwLock::new(threadEachTick),
			_threadEachSecondUpdate: RwLock::new(threadEachSecond),
			_threadRefreshwindows: RwLock::new(threadRefreshwindows),
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
		let _ = namedThread!(move ||{
			
			let oldpage = Arc::unwrap_or_clone(Self::singleton()._activePage.load_full());
			if(name==oldpage) // si on change pas de page, on refresh juste
			{
				if let Some(mut page) = Self::singleton()._pageArray.get_mut(&name)
				{
					page.cache_checkupdate();
				}
				return;
			}
			
			
			if let Some(mut page) = Self::singleton()._pageArray.get_mut(&oldpage) {
				page.event_trigger(event_type::EXIT);
				page.cache_clear();
			}
			
			Self::singleton()._activePage.swap(Arc::new(name.clone()));
			
			if let Some(mut page) = Self::singleton()._pageArray.get_mut(&name) {
				page.event_trigger(event_type::ENTER);
				page.cache_checkupdate();
			}
			
			// sometime cleaning is too long ?
			thread::sleep(Duration::from_millis(10));
			if let Some(page) = Self::singleton()._pageArray.get_mut(&oldpage) {
				page.cache_clear();
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
		let Some(page) = self._pageArray.get_mut(self.getActivePage().as_str()) else {return false};
		return page.eventMouse(x,y,mouseClick);
	}
	
	pub fn WindowRefreshed(&self)
	{
		self._threadRefreshwindows.write().thread_launch_delayabe();
	}
	
	pub fn UiPageAppend(&self, name: impl Into<String>, page: UiPage)
	{
		let name = name.into();
		let old = self._pageArray.insert(name.clone(),page);
		
		if(self.getActivePage()==name)
		{
			if let Some(oldpage) = old
			{
				oldpage.cache_clear();
			}
			
			if let Some(mut page) = self._pageArray.get_mut(&name)
			{
				page.cache_checkupdate();
			}
		}
	}
	
	pub fn UiPageUpdate(&self, name: &str, func: impl Fn(&mut UiPage))
	{
		if let Some(mut page) = self._pageArray.get_mut(name)
		{
			func(page.value_mut());
			page.cache_checkupdate();
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
		let Some(mut page) = self._pageArray.get_mut(self.getActivePage().as_str()) else {return};
		page.subevent_trigger(event_type::EACH_TICK);
		page.cache_checkupdate();
	}
	
	fn EachSecondUpdate(&self)
	{
		let Some(page) = self._pageArray.get(self.getActivePage().as_str()) else {return};
		page.subevent_trigger(event_type::EACH_SECOND);
	}
}
