
#[derive(Debug,Clone)]
pub struct corner4<T>
{
    pub LeftTop: T,
    pub RightTop: T,
    pub LeftBottom: T,
    pub RightBottom: T,
}

impl<T> corner4<T>
{
    pub fn intoArray(self) -> [T; 4]
    {
        return [self.LeftTop, self.RightTop, self.LeftBottom, self.RightBottom];
    }
}

impl<T> corner4<T>
    where T: Copy
{
    pub fn same(val: T) -> Self
    {
        return corner4{
            LeftTop: val,
            RightTop: val,
            LeftBottom: val,
            RightBottom: val,
        };
    }
}

impl<T> corner4<T>
    where T: Clone
{
    pub fn sameCloned(val: T) -> Self
    {
        return corner4{
            LeftTop: val.clone(),
            RightTop: val.clone(),
            LeftBottom: val.clone(),
            RightBottom: val.clone(),
        };
    }
}

pub struct corner2<T>
{
    pub start: T,
    pub end: T,
}

impl<T> corner2<T>
{
    pub fn intoArray(self) -> [T; 2]
    {
        return [self.start, self.end];
    }
}

impl<T> corner2<T>
    where T: Copy
{
    pub fn same(val: T) -> Self
    {
        return corner2{
            start: val,
            end: val,
        };
    }
}

impl<T> corner2<T>
    where T: Clone
{
    pub fn sameCloned(val: T) -> Self
    {
        return corner2{
            start: val.clone(),
            end: val,
        };
    }
}
