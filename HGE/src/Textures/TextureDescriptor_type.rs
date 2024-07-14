use std::ops::{Range, RangeBounds};
use std::collections::Bound;

#[derive(Debug)]
pub enum TextureDescriptor_type<T: RangeBounds<u16>>
{
	ALL(TextureDescriptor_process), // use all texture in ManagerTexture
	ONE(String), // only the texture is used
	ARRAY(Vec<String>, TextureDescriptor_process),
	SIZE_DEPENDENT(T, TextureDescriptor_process),
	SIZE_DEPENDENT_XY(T, T, TextureDescriptor_process),
	SIZE_MIN(T, TextureDescriptor_process)
}

impl<T: RangeBounds<u16>> TextureDescriptor_type<T>
{
	pub fn normalize(self) -> TextureDescriptor_type<Range<u16>>
	{
		match self {
			TextureDescriptor_type::ALL(x) => TextureDescriptor_type::ALL(x),
			TextureDescriptor_type::ONE(x) => TextureDescriptor_type::ONE(x),
			TextureDescriptor_type::ARRAY(x, y) => TextureDescriptor_type::ARRAY(x, y),
			TextureDescriptor_type::SIZE_DEPENDENT(t, a) => TextureDescriptor_type::SIZE_DEPENDENT(Self::converter(t), a),
			TextureDescriptor_type::SIZE_DEPENDENT_XY(x, y, a) => TextureDescriptor_type::SIZE_DEPENDENT_XY(Self::converter(x), Self::converter(y), a),
			TextureDescriptor_type::SIZE_MIN(t, a) => TextureDescriptor_type::SIZE_MIN(Self::converter(t), a)
		}
	}
	
	fn converter(base: T) -> Range<u16>
	{
		match base.start_bound() {
			Bound::Included(x) | Bound::Excluded(x) => {
				match base.end_bound() {
					Bound::Included(y) | Bound::Excluded(y) => Range{
						start: *x,
						end: *y,
					},
					Bound::Unbounded => Range{
						start: *x,
						end: u16::MAX,
					}
				}
			}
			Bound::Unbounded => {
				match base.end_bound() {
					Bound::Included(y) | Bound::Excluded(y) => Range{
						start: u16::MIN,
						end: *y,
					},
					Bound::Unbounded => Range{
						start: u16::MIN,
						end: u16::MAX,
					}
				}
			}
		}
	}
}

#[derive(Debug)]
pub enum TextureDescriptor_process
{
	RAW,
	RESIZE(u16, u16)
}

pub enum TextureDescriptor_exclude
{
	NONE,
	ARRAY(Vec<String>)
}

pub struct TextureDescriptor_adaptedTexture
{
	pub(super) x: u16,
	pub(super) y: u16,
	pub(super) content: Vec<u8>
}
