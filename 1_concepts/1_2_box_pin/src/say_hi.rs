use std::fmt;
use std::{pin::Pin, rc::Rc};

trait SayHi: fmt::Debug {
    fn say_hi(self: Pin<&Self>) {
        println!("Hi from {:?}", self)
    }
}

impl<T: fmt::Debug> SayHi for T {}

pub fn test() {
    let str = Box::pin("String".to_owned());
    str.as_ref().say_hi();

    let array = Box::pin([1, 2, 3]);
    array.as_ref().say_hi();

    let vec = Box::pin(vec![1, 2, 3]);
    vec.as_ref().say_hi();

    let rc = Rc::pin("Rc");
    rc.as_ref().say_hi();

    let t = Box::pin(42);
    t.as_ref().say_hi();
}
