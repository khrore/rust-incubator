use std::cell::{Ref, RefCell};
use std::rc::Rc;

struct GlobalStack<T> {
    data: Rc<RefCell<Vec<T>>>,
}

impl<T> GlobalStack<T> {
    fn new() -> Self {
        Self {
            data: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn push(&self, item: T) {
        self.data.borrow_mut().push(item);
    }

    fn pop(&self) -> Option<T> {
        self.data.borrow_mut().pop()
    }

    fn peek(&self) -> Option<Ref<'_, T>> {
        let vec_ref = self.data.borrow();
        if vec_ref.is_empty() {
            None
        } else {
            Some(Ref::map(vec_ref, |v| &v[v.len() - 1]))
        }
    }

    fn is_empty(&self) -> bool {
        self.data.borrow().is_empty()
    }

    fn len(&self) -> usize {
        self.data.borrow().len()
    }
}

impl<T> Clone for GlobalStack<T> {
    fn clone(&self) -> Self {
        Self {
            data: Rc::clone(&self.data),
        }
    }
}

fn main() {
    println!("GlobalStack demonstration:");

    // Create initial stack
    let stack = GlobalStack::new();

    // Clone creates shared ownership - cheap operation, no data copying
    let stack2 = stack.clone();
    let stack3 = stack.clone();

    println!("\n--- Pushing elements ---");
    // All references can mutate the shared data
    stack.push(1);
    println!("Pushed 1 from stack");

    stack2.push(2);
    println!("Pushed 2 from stack2");

    stack3.push(3);
    println!("Pushed 3 from stack3");

    // All see the same shared data
    println!("\n--- Checking shared state ---");
    println!("Length from stack: {}", stack.len()); // 3
    println!("Length from stack2: {}", stack2.len()); // 3
    println!("Length from stack3: {}", stack3.len()); // 3

    // Peek operation
    println!("\n--- Peeking ---");
    if let Some(top) = stack.peek() {
        println!("Top element from stack: {}", *top); // 3
    }

    if let Some(top) = stack2.peek() {
        println!("Top element from stack2: {}", *top); // 3
    }

    // Pop affects all owners
    println!("\n--- Popping ---");
    if let Some(val) = stack2.pop() {
        println!("Popped {} from stack2", val); // 3
    }

    println!("Length after pop:");
    println!("  stack: {}", stack.len()); // 2
    println!("  stack2: {}", stack2.len()); // 2
    println!("  stack3: {}", stack3.len()); // 2

    // Push more elements
    stack.push(4);
    stack3.push(5);

    println!("\n--- Final state ---");
    println!("Stack contents (popping all):");
    while let Some(val) = stack.pop() {
        println!("  {}", val);
    }

    println!("Final length: {}", stack.len()); // 0
    println!("Is empty: {}", stack.is_empty()); // true
}
