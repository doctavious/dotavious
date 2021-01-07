use crate::dot::DotString;
use std::borrow::Cow;

pub enum Shape {
    Box,
    Polygon,
    Ellipse,
    Oval,
    Circle,
    Point,
    Egg,
    Triangle,
    Plaintext,
    Plain,
    Diamond,
    Trapezium,
    Parallelogram,
    House,
    Pentagon,
    Hexagon,
    Septagon,
    Octagon,
    DoubleCircle,
    DoubleOctagon,
    TripleOctagon,
    Invtriangle,
    Invtrapezium,
    Invhouse,
    Mdiamond,
    Msquare,
    Mcircle,
    Record,
    Rect,
    Rectangle,
    Square,
    Star,
    None,
    Underline,
    Cylinder,
    Note,
    Tab,
    Folder,
    Box3D,
    Component,
    Promoter,
    Cds,
    Terminator,
    Utr,
    Primersite,
    Restrictionsite,
    FivePoverHang,
    ThreePoverHang,
    NoverHang,
    Assemply,
    Signature,
    Insulator,
    Ribosite,
    Rnastab,
    Proteasesite,
    Proteinstab,
    Rpromotor,
    Rarrow,
    Larrow,
    Lpromotor,
}

impl<'a> DotString<'a> for Shape {
    fn dot_string(&self) -> Cow<'a, str> {
        match self {
            Shape::Box => "box".into(),
            Shape::Polygon => "polygon".into(),
            Shape::Ellipse => "ellipse".into(),
            Shape::Oval => "oval".into(),
            Shape::Circle => "circle".into(),
            Shape::Point => "point".into(),
            Shape::Egg => "egg".into(),
            Shape::Triangle => "triangle".into(),
            Shape::Plaintext => "plaintext".into(),
            Shape::Plain => "plain".into(),
            Shape::Diamond => "diamond".into(),
            Shape::Trapezium => "trapezium".into(),
            Shape::Parallelogram => "parallelogram".into(),
            Shape::House => "house".into(),
            Shape::Pentagon => "pentagon".into(),
            Shape::Hexagon => "hexagon".into(),
            Shape::Septagon => "septagon".into(),
            Shape::Octagon => "octagon".into(),
            Shape::DoubleCircle => "doublecircle".into(),
            Shape::DoubleOctagon => "doubleoctagon".into(),
            Shape::TripleOctagon => "tripleocctagon".into(),
            Shape::Invtriangle => "invtriangle".into(),
            Shape::Invtrapezium => "invtrapezium".into(),
            Shape::Invhouse => "invhouse".into(),
            Shape::Mdiamond => "mdiamond".into(),
            Shape::Msquare => "msquare".into(),
            Shape::Mcircle => "mcircle".into(),
            Shape::Record => "record".into(),
            Shape::Rect => "rect".into(),
            Shape::Rectangle => "rectangle".into(),
            Shape::Square => "square".into(),
            Shape::Star => "star".into(),
            Shape::None => "none".into(),
            Shape::Underline => "underline".into(),
            Shape::Cylinder => "cylinder".into(),
            Shape::Note => "note".into(),
            Shape::Tab => "tab".into(),
            Shape::Folder => "folder".into(),
            Shape::Box3D => "box3d".into(),
            Shape::Component => "component".into(),
            Shape::Promoter => "promoter".into(),
            Shape::Cds => "cds".into(),
            Shape::Terminator => "terminator".into(),
            Shape::Utr => "utr".into(),
            Shape::Primersite => "primersite".into(),
            Shape::Restrictionsite => "restrictionsite".into(),
            Shape::FivePoverHang => "fivepoverhang".into(),
            Shape::ThreePoverHang => "threepoverhang".into(),
            Shape::NoverHang => "noverhang".into(),
            Shape::Assemply => "assemply".into(),
            Shape::Signature => "signature".into(),
            Shape::Insulator => "insulator".into(),
            Shape::Ribosite => "ribosite".into(),
            Shape::Rnastab => "rnastab".into(),
            Shape::Proteasesite => "proteasesite".into(),
            Shape::Proteinstab => "proteinstab".into(),
            Shape::Rpromotor => "rpromotor".into(),
            Shape::Rarrow => "rarrow".into(),
            Shape::Larrow => "larrow".into(),
            Shape::Lpromotor => "lpromotor".into(),
        }
    }
}