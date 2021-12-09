use std::cell::Ref;

pub struct RefIter<'a, T> {
    inner: Option<Ref<'a, [T]>>,
}

impl<'a, T> RefIter<'a, T> {
    pub fn new(inner: Ref<'a, [T]>) -> Self {
        Self { inner: Some(inner) }
    }
}

impl<'a, T> Iterator for RefIter<'a, T> {
    type Item = Ref<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.take() {
            Some(borrow) => match *borrow {
                [] => None,
                [_, ..] => {
                    let (head, tail) = Ref::map_split(borrow, |slice| (&slice[0], &slice[1..]));
                    self.inner.replace(tail);
                    Some(head)
                }
            },
            None => None,
        }
    }
}
