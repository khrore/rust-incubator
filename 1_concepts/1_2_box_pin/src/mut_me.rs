use std::pin::Pin;

trait MutMeSomehow {
    fn mut_me_somehow(self: Pin<&mut Self>) -> &mut Self {
        unsafe { self.get_unchecked_mut() }
    }
}

impl<T> MutMeSomehow for T {}

pub fn test() {
    let mut val = 32;
    let mut txt = Box::pin(&mut val);
    let some_txt = txt.as_mut().mut_me_somehow();
}
