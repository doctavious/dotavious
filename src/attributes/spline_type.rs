use crate::attributes::point::Point;
use crate::dot::DotString;
use std::borrow::Cow;

/// The number of points in the list must be equivalent to 1 mod 3; note that this is not checked.
/// TODO: should we check?
pub struct SplineType {
    pub start: Option<Point>,
    pub end: Option<Point>,
    pub spline_points: Vec<Point>,
}

impl<'a> DotString<'a> for SplineType {
    fn dot_string(&self) -> Cow<'a, str> {
        let mut dot_string = String::from("");

        if let Some(end) = &self.end {
            dot_string.push_str(format!("e,{:.1},{:.1} ", end.x, end.y).as_str());
        }

        if let Some(start) = &self.start {
            dot_string.push_str(format!("s,{:.1},{:.1} ", start.x, start.y).as_str());
        }

        let mut iter = self.spline_points.iter();
        let first = iter.next().unwrap();
        dot_string.push_str(format!("{}", first.dot_string()).as_str());
        for point in iter {
            dot_string.push_str(" ");
            dot_string.push_str(format!("{}", point.dot_string()).as_str());
        }

        dot_string.into()
    }
}

#[cfg(test)]
mod test {
    use crate::attributes::{Point, SplineType};
    use crate::DotString;

    #[test]
    fn spline_type() {
        let spline_type = SplineType {
            end: None,
            start: None,
            spline_points: vec![
                Point::new_2d(0.0, 0.0),
                Point::new_2d(1.0, 1.0),
                Point::new_2d(1.0, -1.0),
            ],
        };

        assert_eq!("0.0,0.0 1.0,1.0 1.0,-1.0", spline_type.dot_string());
    }

    #[test]
    fn spline_type_end() {
        let spline_type = SplineType {
            end: Some(Point::new_2d(2.0, 0.0)),
            start: None,
            spline_points: vec![
                Point::new_2d(0.0, 0.0),
                Point::new_2d(1.0, 1.0),
                Point::new_2d(1.0, -1.0),
            ],
        };

        assert_eq!(
            "e,2.0,0.0 0.0,0.0 1.0,1.0 1.0,-1.0",
            spline_type.dot_string()
        );
    }

    #[test]
    fn spline_type_start() {
        let spline_type = SplineType {
            end: None,
            start: Some(Point::new_2d(-1.0, 0.0)),
            spline_points: vec![
                Point::new_2d(0.0, 0.0),
                Point::new_2d(1.0, 1.0),
                Point::new_2d(1.0, -1.0),
            ],
        };

        assert_eq!(
            "s,-1.0,0.0 0.0,0.0 1.0,1.0 1.0,-1.0",
            spline_type.dot_string()
        );
    }

    #[test]
    fn spline_type_complete() {
        let spline_type = SplineType {
            end: Some(Point::new_2d(2.0, 0.0)),
            start: Some(Point::new_2d(-1.0, 0.0)),
            spline_points: vec![
                Point::new_2d(0.0, 0.0),
                Point::new_2d(1.0, 1.0),
                Point::new_2d(1.0, -1.0),
            ],
        };

        assert_eq!(
            "e,2.0,0.0 s,-1.0,0.0 0.0,0.0 1.0,1.0 1.0,-1.0",
            spline_type.dot_string()
        );
    }
}
