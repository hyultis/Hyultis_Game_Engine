use crate::components::event::{event_trait, event_type};
use crate::Interface::UiPage::UiPage;
use arc_swap::ArcSwap;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use parking_lot::RwLock;
use singletonThread::SingletonThread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread::sleep;
use std::time::Duration;
use Htrace::namedThread;
use Htrace::HTracer::HTracer;

pub struct ManagerInterface
{
	_pageArray: DashMap<String, UiPage>,
	_activePage: ArcSwap<String>,
	_threadEachTickUpdate: RwLock<SingletonThread>,
	_threadEachSecondUpdate: RwLock<SingletonThread>,
	_threadRefreshwindows: RwLock<SingletonThread>,
	_canChangePage: AtomicBool,
}

static SINGLETON: OnceLock<ManagerInterface> = OnceLock::new();

impl ManagerInterface
{
	fn new() -> ManagerInterface
	{
		let threadEachTick = SingletonThread::new(|| {
			HTracer::threadSetName("ManagerInterface_ST");
			ManagerInterface::singleton().EachTickUpdate();
		});
		let mut threadEachSecond = SingletonThread::new(|| {
			HTracer::threadSetName("ManagerInterface_ST");
			ManagerInterface::singleton().EachSecondUpdate();
		});
		threadEachSecond.setDuration_FPS(1);
		let threadRefreshwindows = SingletonThread::new(|| {
			HTracer::threadSetName("ManagerInterface_WR");
			sleep(Duration::from_millis(50));
			let Some(page) = Self::singleton()._pageArray.get(Self::singleton().getActivePage().as_str())
			else
			{
				return;
			};
			page.eventWinRefresh();

			let otherpage = Self::singleton()
				._pageArray
				.iter()
				.filter(|x| x.key().ne(&Self::singleton().getActivePage()))
				.map(|x| x.key().clone())
				.collect::<Vec<String>>();
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
			_canChangePage: AtomicBool::new(true),
		};
	}

	pub fn singleton() -> &'static ManagerInterface
	{
		return SINGLETON.get_or_init(|| ManagerInterface::new());
	}

	pub fn changeActivePage(&self, name: impl Into<String>)
	{
		if (self._canChangePage.compare_exchange(true, false, Ordering::Release, Ordering::Acquire).is_err())
		{
			return;
		}

		let name = name.into();
		let _ = namedThread!(|| {
			let oldpage = (&*Self::singleton()._activePage.swap(Arc::new(name.clone()))).clone();

			// si on change pas de page, on refresh juste
			if (name == oldpage)
			{
				if let Some(mut page) = Self::singleton()._pageArray.get_mut(&name)
				{
					page.cache_checkupdate();
				}

				Self::singleton()._canChangePage.store(true, Ordering::Release);
				return;
			}

			if let Some(mut page) = Self::singleton()._pageArray.get_mut(&oldpage)
			{
				page.event_trigger(event_type::EXIT);
				page.cache_remove();
			};

			if let Some(mut page) = Self::singleton()._pageArray.get_mut(&name)
			{
				page.event_trigger(event_type::ENTER);
				page.cache_checkupdate();
			}

			Self::singleton()._canChangePage.store(true, Ordering::Release);
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
		let Some(page) = self._pageArray.get_mut(self.getActivePage().as_str())
		else
		{
			return false;
		};
		return page.eventMouse(x, y, mouseClick);
	}

	pub fn WindowRefreshed(&self)
	{
		if let Some(mut t) = self._threadRefreshwindows.try_write()
		{
			t.thread_launch_delayabe();
		}
	}

	pub fn UiPageAppend(&self, name: impl Into<String>, page: UiPage)
	{
		let name = name.into();
		if let Some(oldpage) = self._pageArray.insert(name.clone(), page)
		{
			oldpage.cache_remove();
		}

		if (self.getActivePage() == name)
		{
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
		if let Some(mut t) = self._threadEachTickUpdate.try_write()
		{
			t.thread_launch();
		}
		if let Some(mut t) = self._threadEachSecondUpdate.try_write()
		{
			t.thread_launch();
		}
	}

	///////////// PRIVATE //////////////////

	fn EachTickUpdate(&self)
	{
		let Some(mut page) = self._pageArray.get_mut(self.getActivePage().as_str())
		else
		{
			return;
		};
		page.subevent_trigger(event_type::EACH_TICK);
		page.cache_checkupdate();
	}

	fn EachSecondUpdate(&self)
	{
		let Some(page) = self._pageArray.get(self.getActivePage().as_str())
		else
		{
			return;
		};
		page.subevent_trigger(event_type::EACH_SECOND);
	}
}
