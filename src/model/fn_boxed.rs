use crate::views::{Point, Tuple};
use gtk::glib;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, glib::Boxed)]
#[boxed_type(name = "FnBoxedTuple")]
#[allow(clippy::type_complexity)]
pub struct FnBoxedTuple(pub Rc<RefCell<Option<Box<dyn Fn(&Tuple) -> String>>>>);

impl FnBoxedTuple {
    pub fn new(func: Option<Box<dyn Fn(&Tuple) -> String>>) -> Self {
        Self(Rc::new(RefCell::new(func)))
    }
}

#[derive(Clone, glib::Boxed)]
#[boxed_type(name = "FnBoxedPoint")]
#[allow(clippy::type_complexity)]
pub struct FnBoxedPoint(pub Rc<RefCell<Option<Box<dyn Fn(&Point) -> String>>>>);

impl FnBoxedPoint {
    pub fn new(func: Option<Box<dyn Fn(&Point) -> String>>) -> Self {
        Self(Rc::new(RefCell::new(func)))
    }
}

#[cfg(test)]
mod test {
    use super::FnBoxedPoint;

    #[test]
    fn new() {
        FnBoxedPoint::new(None);
    }
}
