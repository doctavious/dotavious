use crate::attributes::point::Point;
use crate::dot::DotString;
use std::borrow::Cow;

pub struct Rectangle {
    lower_left: Point,
    upper_right: Point,
}

impl<'a> DotString<'a> for Rectangle {
    fn dot_string(&self) -> Cow<'a, str> {
        format!(
            "{:.1},{:.1},{:.1},{:.1}",
            self.lower_left.x, self.lower_left.y, self.upper_right.x, self.upper_right.y
        )
        .into()
    }
}

#[cfg(test)]
mod test {
    use crate::attributes::{Rectangle, Point};
    use crate::DotString;

    #[test]
    fn dot_string() {
        assert_eq!("0.0,0.0,1.0,1.0", Rectangle {
            lower_left: Point::new_2d(0.0, 0.0),
            upper_right: Point::new_2d(1.0, 1.0)
        }.dot_string());
    }

}