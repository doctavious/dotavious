use crate::attributes::Point;
use crate::DotString;
use std::borrow::Cow;

pub struct ViewPort {
    pub width: f32,
    pub height: f32,
    pub zoom: f32,
    pub focus: Option<FocusType>
}

impl ViewPort {
    pub fn new(width: f32, height: f32, zoom: Option<f32>, focus: Option<FocusType>) -> Self {
        Self {
            width,
            height,
            zoom: zoom.unwrap_or(1 as f32),
            focus
        }
    }

    pub fn new_point(width: f32, height: f32, zoom: Option<f32>, x: f32, y: f32) -> Self {
        Self {
            width,
            height,
            zoom: zoom.unwrap_or(1 as f32),
            focus: Some(FocusType::Point(Point::new_2d(x, y)))
        }
    }

    pub fn new_node(width: f32, height: f32, zoom: Option<f32>, node: String) -> Self {
        Self {
            width,
            height,
            zoom: zoom.unwrap_or(1 as f32),
            focus: Some(FocusType::Node(node))
        }
    }
}

impl<'a> DotString<'a> for ViewPort {
    fn dot_string(&self) -> Cow<'a, str> {
        let mut dot_string = String::from("");
        dot_string.push_str(
            format!("{:.1},{:.1},{:.1}",
                    self.width, self.height, self.zoom
        ).as_str());

        if let Some(focus) = &self.focus {
            match focus {
                FocusType::Point(p) => {
                    dot_string.push_str(format!(",{}", p.dot_string()).as_str());
                },
                FocusType::Node(n) => {
                    dot_string.push_str(format!(",'{}'", n).as_str());
                },
            }
        }

        dot_string.into()
    }
}

pub enum FocusType {
    Point(Point),
    Node(String)
}


#[cfg(test)]
mod test {
    use crate::attributes::{ViewPort};
    use crate::DotString;

    #[test]
    fn viewport_dot_string() {
        assert_eq!(
            "1.0,2.0,1.0",
            ViewPort::new(1.0, 2.0, None, None).dot_string()
        );
    }

    #[test]
    fn viewport_zoom_dot_string() {
        assert_eq!(
            "1.0,2.0,3.0",
            ViewPort::new(1.0, 2.0, Some(3.0), None).dot_string()
        );
    }

    #[test]
    fn viewport_point_focus_dot_string() {
        assert_eq!(
            "1.0,2.0,3.0,5.0,10.0",
            ViewPort::new_point(1.0, 2.0, Some(3.0), 5.0, 10.0).dot_string()
        );
    }

    #[test]
    fn viewport_node_focus_dot_string() {
        assert_eq!(
            "1.0,2.0,3.0,'2.8 BSD'",
            ViewPort::new_node(
                1.0, 2.0, Some(3.0), String::from("2.8 BSD")
            ).dot_string()
        );
    }
}