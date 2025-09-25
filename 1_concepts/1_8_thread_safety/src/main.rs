use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};

struct OnlySync<'a> {
    a: MutexGuard<'a, i32>,
}

struct OnlySend {
    a: Cell<i32>,
}

struct IsSyncNotSend {
    a: Arc<i32>,
}

struct SyncAndSend {
    a: Rc<i32>,
}

fn main() {
    println!("Implement me!");
}
