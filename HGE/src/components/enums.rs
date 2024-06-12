use derive_more::Display;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Display)]
pub enum ALIGN_V
{
	TOP,
	CENTER,
	BOTTOM
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Display)]
pub enum ALIGN_H
{
	LEFT,
	CENTER,
	RIGHT
}
