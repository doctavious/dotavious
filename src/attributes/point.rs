use crate::dot::DotString;
use std::borrow::Cow;

pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: Option<f32>,

    /// specify that the node position should not change.
    pub force_pos: bool,
}

impl Point {
    pub fn new_2d(x: f32, y: f32) -> Self {
        Self::new(x, y, None, false)
    }

    pub fn new_3d(x: f32, y: f32, z: f32) -> Self {
        Self::new(x, y, Some(z), false)
    }

    pub fn new(x: f32, y: f32, z: Option<f32>, force_pos: bool) -> Self {
        Self { x, y, z, force_pos }
    }
}

impl<'a> DotString<'a> for Point {
    fn dot_string(&self) -> Cow<'a, str> {
        let mut slice = format!("{:.1},{:.1}", self.x, self.y);
        if self.z.is_some() {
            slice.push_str(format!(",{:.1}", self.z.unwrap()).as_str());
        }
        if self.force_pos {
            slice.push_str("!")
        }
        slice.into()
    }
}

#[cfg(test)]
mod test {
    use crate::attributes::Point;
    use crate::DotString;

    #[test]
    fn dot_string() {
        assert_eq!("1.0,2.0", Point::new_2d(1.0, 2.0).dot_string());
        assert_eq!("1.0,2.0,0.0", Point::new_3d(1.0, 2.0, 0.0).dot_string());
        assert_eq!("1.0,2.0!", Point::new(1.0, 2.0, None, true).dot_string());
        assert_eq!(
            "1.0,2.0,0.0!",
            Point::new(1.0, 2.0, Some(0.0), true).dot_string()
        );
    }
}
