use crate::dot::DotString;
use std::borrow::Cow;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Color<'a> {
    RGB {
        red: u8,
        green: u8,
        blue: u8,
    },
    RGBA {
        red: u8,
        green: u8,
        blue: u8,
        alpha: u8,
    },
    // TODO: constrain?
    // Hue-Saturation-Value (HSV) 0.0 <= H,S,V <= 1.0
    HSV {
        hue: f32,
        saturation: f32,
        value: f32,
    },
    Named(&'a str),
}

impl<'a> DotString<'a> for Color<'a> {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            Color::RGB { red, green, blue } => {
                format!("#{:02x?}{:02x?}{:02x?}", red, green, blue).into()
            }
            Color::RGBA {
                red,
                green,
                blue,
                alpha,
            } => {
                format!("#{:02x?}{:02x?}{:02x?}{:02x?}", red, green, blue, alpha).into()
            }
            Color::HSV {
                hue,
                saturation,
                value,
            } => format!("{} {} {}", hue, saturation, value).into(),
            Color::Named(color) => (*color).into(),
        }
    }
}

// The sum of the optional weightings must sum to at most 1.
pub struct WeightedColor<'a> {
    pub color: Color<'a>,

    // TODO: constrain
    /// Must be in range 0 <= W <= 1.
    pub weight: Option<f32>,
}

impl<'a> DotString<'a> for WeightedColor<'a> {
    fn dot_string(&self) -> Cow<'a, str> {
        let mut dot_string = self.color.dot_string().to_string();
        if let Some(weight) = &self.weight {
            dot_string.push_str(format!(";{}", weight).as_str());
        }
        dot_string.into()
    }
}

pub struct ColorList<'a> {
    pub colors: Vec<WeightedColor<'a>>,
}

impl<'a> DotString<'a> for ColorList<'a> {
    /// A colon-separated list of weighted color values: WC(:WC)* where each WC has the form C(;F)?
    /// Ex: fillcolor=yellow;0.3:blue
    fn dot_string(&self) -> Cow<'a, str> {
        let mut dot_string = String::new();
        let mut iter = self.colors.iter();
        let first = iter.next();
        if first.is_none() {
            return dot_string.into();
        }
        dot_string.push_str(&first.unwrap().dot_string());
        for weighted_color in iter {
            dot_string.push_str(":");
            dot_string.push_str(&weighted_color.dot_string())
        }

        dot_string.into()
    }
}

/// Convert an element like `(i, j)` into a WeightedColor
pub trait IntoWeightedColor<'a> {
    fn into_weighted_color(self) -> WeightedColor<'a>;
}

impl<'a> IntoWeightedColor<'a> for &(Color<'a>, Option<f32>) {
    fn into_weighted_color(self) -> WeightedColor<'a> {
        let (s, t) = *self;
        WeightedColor {
            color: s,
            weight: t,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::attributes::{Color, ColorList, WeightedColor};
    use crate::DotString;

    #[test]
    fn colorlist_dot_string() {
        let yellow = WeightedColor {
            color: Color::Named("yellow"),
            weight: Some(0.3),
        };

        let blue = WeightedColor {
            color: Color::Named("blue"),
            weight: None,
        };

        let color_list = ColorList {
            colors: vec![yellow, blue],
        };

        let dot_string = color_list.dot_string();

        assert_eq!("yellow;0.3:blue", dot_string);
    }

    #[test]
    fn color_rbg_dot_string() {
        let color = Color::RGB {
            red: 160,
            green: 82,
            blue: 45,
        };
        assert_eq!("#a0522d", color.dot_string());
    }

    #[test]
    fn color_rbga_dot_string() {
        let color = Color::RGBA {
            red: 160,
            green: 82,
            blue: 45,
            alpha: 10,
        };
        assert_eq!("#a0522d0a", color.dot_string());
    }

    #[test]
    fn color_hsv_dot_string() {
        let color = Color::HSV {
            hue: 0.051,
            saturation: 0.718,
            value: 0.627,
        };
        assert_eq!("0.051 0.718 0.627", color.dot_string());
    }
}
