use std::sync::{Arc, OnceLock};
use ab_glyph::FontArc;
use ahash::AHashMap;
use anyhow::anyhow;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use glyph_brush::{BrushAction, BrushError, GlyphBrush, GlyphBrushBuilder, GlyphVertex, OwnedSection, Rectangle};
use glyph_brush_layout::FontId;
use Htrace::HTrace;
use Htrace::HTracer::HTracer;
use image::{GrayImage, Rgba, RgbaImage};
use parking_lot::{RwLock, RwLockReadGuard};
use vulkano::image::sampler::{Filter, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode};
use crate::Interface::Text::{Extra, TextCacheUpdater};
use crate::Shaders::HGE_shader_2Dsimple::HGE_shader_2Dsimple;
use crate::Textures::generate::emptyTexture;
use crate::Textures::Manager::ManagerTexture;
use crate::Textures::Orders::Order_load::Order_load;
use crate::Textures::Orders::Order_partialTextureUpdate::Order_partialTextureUpdate;
use crate::Textures::Orders::Order_resize::Order_resize;
use crate::Textures::textureLoader::textureLoader_fromRaw;
use singletonThread::SingletonThread;
use crate::assetStreamReader::assetManager;
use crate::Textures::Order::Order;
use crate::Textures::Orders::Order_computeCache::Order_computeCache;

#[derive(Clone)]
pub struct ManagerFont_verticestmp
{
	pub textId: u128,
	pub vertex: Vec<HGE_shader_2Dsimple>,
	pub indices: Vec<u32>
}

impl PartialEq for ManagerFont_verticestmp
{
	fn eq(&self, other: &Self) -> bool {
		self.textId == self.textId && self.vertex.len() == other.vertex.len()
	}
}

static DEFAULTTEXTURESIZE: u32 = 32;

pub struct ManagerFont
{
	_uniqId: RwLock<u128>,
	_storeText: Arc<DashMap<u128, OwnedSection<Extra>>>,
	_storeCallBack: Arc<DashMap<u128, Arc<dyn Fn(TextCacheUpdater) + Send + Sync>>>,
	_storeFontId: Arc<DashMap<String, FontId>>,
	_fontEngine: RwLock<Option<GlyphBrush<ManagerFont_verticestmp, Extra>>>,
	_fontEngineTextureSize: ArcSwap<[u32; 2]>,
	_threadLoading: RwLock<SingletonThread>,
	_updateNeed: RwLock<bool>
}

static SINGLETON: OnceLock<ManagerFont> = OnceLock::new();


impl ManagerFont
{
	fn new() -> ManagerFont
	{
		ManagerTexture::singleton().addSampler("fontsampler", SamplerCreateInfo {
			mag_filter: Filter::Linear,
			min_filter: Filter::Linear,
			address_mode: [SamplerAddressMode::ClampToEdge; 3],
			lod: 0.0..=16.0,
			anisotropy: Some(16.0),
			mipmap_mode: SamplerMipmapMode::Linear,
			..Default::default()
		});
		
		let texture = emptyTexture(DEFAULTTEXTURESIZE, DEFAULTTEXTURESIZE);
		ManagerTexture::singleton().texture_load("font", Order_load::newPrioritize(
			textureLoader_fromRaw {
				raw: texture.to_vec(),
				width: texture.width(),
				height: texture.height(),
				canReload: false,
			}), Some("fontsampler"));
		
		let mut thread = SingletonThread::newFiltered(|| {
			HTracer::threadSetName("ManagerFont");
			ManagerFont::singleton().FontEngine_internalCacheUpdate();
		}, || -> bool {
			return *ManagerFont::singleton()._updateNeed.read();
		});
		thread.setDuration_FPS(144);
		
		return ManagerFont {
			_uniqId: RwLock::new(0),
			_storeText: Default::default(),
			_storeCallBack: Default::default(),
			_storeFontId: Default::default(),
			_fontEngine: Default::default(),
			_fontEngineTextureSize: ArcSwap::new(Arc::new([DEFAULTTEXTURESIZE, DEFAULTTEXTURESIZE])),
			_threadLoading: RwLock::new(thread),
			_updateNeed: RwLock::new(false),
		};
	}
	
	pub fn singleton() -> &'static ManagerFont
	{
		return SINGLETON.get_or_init(|| {
			ManagerFont::new()
		});
	}
	
	pub fn Text_add(&self, newtext: OwnedSection<Extra>, callback: impl Fn(TextCacheUpdater) + Send + Sync + 'static, id: u128)
	{
		self._storeText.insert(id, newtext);
		self._storeCallBack.insert(id, Arc::new(callback));
		*self._updateNeed.write() = true;
	}
	
	pub fn Text_remove(&self, id: u128)
	{
		self._storeText.remove(&id);
		self._storeCallBack.remove(&id);
	}
	
	
	pub fn FontLoad(&self, lang: impl Into<String>) -> anyhow::Result<()>
	{
		let lang = lang.into();
		let fontUser = self.loadFond(lang.clone(), "NotoSans-SemiBold")?;
		let fontUser = FontArc::try_from_vec(fontUser)?;
		let fontUniversel = self.loadFond("world", "NotoSans-SemiBold")?;
		let fontUniversel = FontArc::try_from_vec(fontUniversel)?;
		let fontBold = self.loadFond("world", "NotoSans-Black")?;
		let fontBold = FontArc::try_from_vec(fontBold)?;
		
		let mut glyph_brush = GlyphBrushBuilder::using_fonts([fontUser,fontUniversel,fontBold].into())
			.draw_cache_position_tolerance(1.0)
			.draw_cache_scale_tolerance(1.0)
			.build();
		self._storeFontId.insert("user".to_string(),FontId(0));
		self._storeFontId.insert("normal".to_string(),FontId(1));
		self._storeFontId.insert("bold".to_string(),FontId(2));
		let tmp = *self._fontEngineTextureSize.load_full();
		glyph_brush.resize_texture(tmp[0], tmp[1]);
		
		*self._fontEngine.write() = Some(glyph_brush);
		
		return Ok(());
	}
	
	pub fn FontIdGet(&self, name: &str) -> Option<FontId>
	{
		return match self._storeFontId.get(name) {
			None => None,
			Some(font) => {
				Some(font.value().clone())
			}
		};
	}
	
	pub fn FontEngineGet(&self) -> RwLockReadGuard<'_, Option<GlyphBrush<ManagerFont_verticestmp, Extra>>>
	{
		self._fontEngine.read()
	}
	
	pub fn FontEngine_CacheUpdate(&self)
	{
		self._threadLoading.write().thread_launch();
	}
	
	pub fn getUniqId(&self) -> u128
	{
		let mut binding = self._uniqId.write();
		let returned = *binding;
		*binding += 1;
		return returned;
	}
	
	//////////// PRIVATE //////////////
	
	fn loadFond(&self, lang: impl Into<String>, file: impl Into<String>) -> anyhow::Result<Vec<u8>>
	{
		let lang = lang.into();
		let file = file.into();
		let fullpath = format!("fonts/{}/{}.ttf", lang, file);
		let returning = match assetManager::singleton().readFile(fullpath.clone())
		{
			None => { Err(anyhow!(format!("Cannot read font file {}", fullpath))) },
			Some(result) => {
				Ok(result.into_inner())
			}
		};
		
		return returning;
	}
	
	
	fn FontEngine_internalCacheUpdate(&self)
	{
		HTracer::threadSetName("FontEngine");
		if let Some(FontEngine) = self._fontEngine.write().as_mut()
		{
			let fontidtexture = ManagerTexture::singleton().getTextureToId("font");
			if fontidtexture.is_none()
			{
				return;
			}
			let fontidtexture = fontidtexture.unwrap();
			let _TextureSize = *self._fontEngineTextureSize.load_full();
			
			self._storeText.iter().for_each(|item| {
				let tmp = item.value().clone();
				FontEngine.queue(tmp.to_borrowed());
			});
			
			{
				//FontEngine.resize_texture(1, 1); // need to reset cache to clear desync cache bug (?!)
				//FontEngine.resize_texture(TextureSize[0], TextureSize[1]);
			}
			
			let mut textureUpdate: Vec<Box<dyn Order + Send + Sync>> = Vec::new();
			
			let result = FontEngine.process_queued(
				|rect, tex_data| {
					textureUpdate.push(Box::new(self.processInternal_textureUpdate(rect, tex_data)));
				},
				|vertex_data| {
					self.processInternal_vertexConvert(vertex_data, fontidtexture)
				},
			);
			
			match result
			{
				Ok(BrushAction::Draw(mut vertices)) =>
					{
						if (vertices.len() > 0)
						{
							let mut storage = AHashMap::new();
							//println!("ManagerFont : text vertices to update : {}", vertices.len());
							vertices.iter_mut().for_each(|x|
								{
									if (!storage.contains_key(&x.textId))
									{
										storage.insert(x.textId, vec![]);
									}
									
									storage.get_mut(&x.textId).unwrap().push(TextCacheUpdater {
										vertex: x.vertex.drain(0..).collect(),
										indices: x.indices.drain(0..).collect(),
										isUpdated: true,
									});
								});
							
							// storage converter
							let mut storageToCache = AHashMap::new();
							storage.into_iter().for_each(|(textid, cache)| {
								let mut finalCache = TextCacheUpdater { vertex: vec![], indices: vec![], isUpdated: true };
								
								cache.into_iter().for_each(|mut x| {
									let oldMaxIndice = finalCache.vertex.len() as u32;
									finalCache.vertex.append(&mut x.vertex);
									finalCache.indices.append(&mut x.indices.iter().map(|x| { x + oldMaxIndice }).collect());
								});
								
								if let Some(callback) = ManagerFont::singleton()._storeCallBack.get(&textid)
								{
									let func = callback.value().clone();
									storageToCache.insert(textid, move ||{
										func(finalCache);
									});
								}
							});
							
							
							textureUpdate.push(Box::new(Order_computeCache::newPrioritize()));
							ManagerTexture::singleton().texture_update("font", textureUpdate);
							storageToCache.clone().into_iter().for_each(|(_, func)|
							{
								func();
							});
							*ManagerFont::singleton()._updateNeed.write() = false;
							/*ManagerTexture::singleton().texture_setCallback("font", move |_| {
							});*/
						}
					}
				Err(BrushError::TextureTooSmall { suggested }) =>
					{
						HTrace!("Resizing font texture {:?}", suggested);
						FontEngine.resize_texture(suggested.0, suggested.1);
						self._fontEngineTextureSize.swap(Arc::new([suggested.0, suggested.1]));
						
						ManagerTexture::singleton().texture_update("font", vec![Box::new(Order_resize {
							newWidth: suggested.0,
							newHeight: suggested.1,
							sameThread: true,
						})]);
					}
				_ => {}
			}
		}
	}
	
	fn processInternal_textureUpdate(&self, rect: Rectangle<u32>, tex_data: &[u8]) -> Order_partialTextureUpdate
	{
		let gray = GrayImage::from_raw(rect.width(), rect.height(), tex_data.to_vec()).unwrap();
		let mut finalchar = RgbaImage::from_pixel(rect.width(), rect.height(), Rgba([255, 255, 255, 0])); // Rgba([255,255,255,0]
		
		for (x, y, pixel) in finalchar.enumerate_pixels_mut()
		{
			let graypixel = gray.get_pixel(x, y);
			pixel.0[3] = graypixel.0[0];
		}
		
		Order_partialTextureUpdate {
			raw: finalchar,
			offset: [rect.min[0], rect.min[1]],
			sameThread: true,
		}
	}
	
	fn processInternal_vertexConvert(&self, vertex_data: GlyphVertex<Extra>, fontid: u32) -> ManagerFont_verticestmp
	{
		let mut gl_rect = ab_glyph::Rect {
			min: ab_glyph::point(vertex_data.pixel_coords.min.x, vertex_data.pixel_coords.min.y),
			max: ab_glyph::point(vertex_data.pixel_coords.max.x, vertex_data.pixel_coords.max.y),
		};
		let mut tex_coords = ab_glyph::Rect {
			min: ab_glyph::point(vertex_data.tex_coords.min.x, vertex_data.tex_coords.min.y),
			max: ab_glyph::point(vertex_data.tex_coords.max.x, vertex_data.tex_coords.max.y),
		};
		
		// handle overlapping bounds, modify uv_rect to preserve texture aspect
		if gl_rect.max.x > vertex_data.bounds.max.x {
			let old_width = gl_rect.width();
			gl_rect.max.x = vertex_data.bounds.max.x;
			tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
		}
		if gl_rect.min.x < vertex_data.bounds.min.x {
			let old_width = gl_rect.width();
			gl_rect.min.x = vertex_data.bounds.min.x;
			tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
		}
		if gl_rect.max.y > vertex_data.bounds.max.y {
			let old_height = gl_rect.height();
			gl_rect.max.y = vertex_data.bounds.max.y;
			tex_coords.max.y = tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
		}
		if gl_rect.min.y < vertex_data.bounds.min.y {
			let old_height = gl_rect.height();
			gl_rect.min.y = vertex_data.bounds.min.y;
			tex_coords.min.y = tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
		}
		
		let vertex = vec![HGE_shader_2Dsimple {
			position: [gl_rect.min.x, gl_rect.min.y, 0.0],
			ispixel: 1,
			texture: fontid,
			uvcoord: [tex_coords.min.x, tex_coords.min.y],
			color: vertex_data.extra.color,
			color_blend_type: 0,
		}, HGE_shader_2Dsimple {
			position: [gl_rect.max.x, gl_rect.min.y, 0.0],
			ispixel: 1,
			texture: fontid,
			uvcoord: [tex_coords.max.x, tex_coords.min.y],
			color: vertex_data.extra.color,
			color_blend_type: 0,
		}, HGE_shader_2Dsimple {
			position: [gl_rect.min.x, gl_rect.max.y, 0.0],
			ispixel: 1,
			texture: fontid,
			uvcoord: [tex_coords.min.x, tex_coords.max.y],
			color: vertex_data.extra.color,
			color_blend_type: 0,
		}, HGE_shader_2Dsimple {
			position: [gl_rect.max.x, gl_rect.max.y, 0.0],
			ispixel: 1,
			texture: fontid,
			uvcoord: [tex_coords.max.x, tex_coords.max.y],
			color: vertex_data.extra.color,
			color_blend_type: 0,
		}];
		
		//let mut tmp = StructAllCache::new();
		//tmp.set(vertex, [0, 1, 2, 1, 3, 2].to_vec());
		
		return ManagerFont_verticestmp {
			textId: vertex_data.extra.textId,
			vertex: vertex,
			indices: [0, 1, 2, 1, 3, 2].to_vec(),
		};
	}
}
