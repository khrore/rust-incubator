use std::pin::{Pin, pin};

#[derive(Default)]
struct AddrTracker(Option<usize>);

impl AddrTracker {
    // If we haven't checked the addr of self yet, store the current
    // address. If we have, confirm that the current address is the same
    // as it was last time, or else panic.
    fn check_for_move(self: Pin<&mut Self>) {
        let current_addr = &*self as *const Self as usize;
        match self.0 {
            None => {
                // SAFETY: we do not move out of self
                let self_data_mut = unsafe { self.get_unchecked_mut() };
                self_data_mut.0 = Some(current_addr);
            }
            Some(prev_addr) => assert_eq!(prev_addr, current_addr),
        }
    }
}

fn main() {
    // Create a tracker and store the initial address
    let tracker = AddrTracker::default();
    let mut ptr_to_pinned_tracker: Pin<&mut AddrTracker> = pin!(tracker);
    ptr_to_pinned_tracker.as_mut().check_for_move();
    ptr_to_pinned_tracker.as_mut().check_for_move();
}
