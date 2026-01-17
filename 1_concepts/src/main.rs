use std::fmt::Debug;
use std::sync::{Arc, Mutex, Weak};
use std::thread;

/// Internal node with bidirectional links
struct Node<T> {
    data: T,
    next: Option<Arc<Mutex<Node<T>>>>,
    prev: Option<Weak<Mutex<Node<T>>>>, // Weak to prevent cycles!
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            next: None,
            prev: None,
        }
    }
}

/// Thread-safe doubly linked list with interior mutability.
/// Cloning creates a new handle to the SAME list, not a copy.
#[derive(Clone)]
pub struct DoublyLinkedList<T> {
    head: Option<Arc<Mutex<Node<T>>>>,
    tail: Option<Arc<Mutex<Node<T>>>>,
    len: Arc<Mutex<usize>>, // Shared counter
}

impl<T: Debug> DoublyLinkedList<T>
where
    T: Send + Sync,
{
    /// Create a new empty list
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: Arc::new(Mutex::new(0)),
        }
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Get the current length
    pub fn len(&self) -> usize {
        *self.len.lock().unwrap()
    }

    /// Add an element to the back of the list
    pub fn push_back(&self, data: T) {
        // TODO(human): Implement the push_back logic here.
        // This is the core of the data structure where you need to:
        // 1. Create a new node wrapped in Arc<Mutex<>>
        // 2. Handle two cases:
        //    a) Empty list: Set both head and tail to the new node
        //    b) Non-empty list: Link the new node to the current tail,
        //       update tail.next, set new node's prev to weak ref of old tail,
        //       update self.tail to point to the new node
        // 3. Increment the length counter
        //
        // Critical considerations:
        // - You'll need to lock nodes when modifying their next/prev pointers
        // - Use Arc::downgrade() to create Weak references
        // - Keep locking scopes minimal to avoid deadlocks
        // - Remember: multiple threads might call this simultaneously!

        todo!("Implement push_back")
    }

    /// Add an element to the front of the list
    pub fn push_front(&self, data: T) {
        let new_node = Arc::new(Mutex::new(Node::new(data)));

        match &self.head {
            None => {
                // Empty list: new node is both head and tail
                self.head = Some(new_node.clone());
                self.tail = Some(new_node);
            }
            Some(old_head) => {
                // Link new node to old head
                new_node.lock().unwrap().next = Some(old_head.clone());
                old_head.lock().unwrap().prev = Some(Arc::downgrade(&new_node));

                // Update head to new node
                self.head = Some(new_node);
            }
        }

        *self.len.lock().unwrap() += 1;
    }

    /// Print all elements from head to tail
    pub fn print_forward(&self)
    where
        T: Debug,
    {
        print!("Forward: [");
        let mut current = self.head.clone();
        let mut first = true;

        while let Some(node_arc) = current {
            let node = node_arc.lock().unwrap();
            if !first {
                print!(", ");
            }
            print!("{:?}", node.data);
            first = false;
            current = node.next.clone();
        }
        println!("]");
    }

    /// Print all elements from tail to head
    pub fn print_backward(&self)
    where
        T: Debug,
    {
        print!("Backward: [");
        let mut current = self.tail.clone();
        let mut first = true;

        while let Some(node_arc) = current {
            let node = node_arc.lock().unwrap();
            if !first {
                print!(", ");
            }
            print!("{:?}", node.data);
            first = false;
            current = node.prev.as_ref().and_then(|weak| weak.upgrade());
        }
        println!("]");
    }
}

impl<T: Send + Sync + Debug> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Safety: DoublyLinkedList is Send because all internal components are Send
unsafe impl<T: Send> Send for DoublyLinkedList<T> {}
// Safety: DoublyLinkedList is Sync because all mutations go through Mutex
unsafe impl<T: Send> Sync for DoublyLinkedList<T> {}

fn main() {
    let list = DoublyLinkedList::new();

    // Single-threaded usage
    println!("=== Single-threaded test ===");
    list.push_back(1);
    list.push_back(2);
    list.push_front(0);
    list.print_forward();
    list.print_backward();
    println!("Length: {}\n", list.len());

    // Multi-threaded usage
    println!("=== Multi-threaded test ===");
    let list2 = DoublyLinkedList::new();
    let list2_clone = list2.clone(); // Same list, different handle

    let handle = thread::spawn(move || {
        for i in 0..5 {
            list2_clone.push_back(i);
            println!("Thread 1 pushed: {}", i);
        }
    });

    for i in 10..15 {
        list2.push_back(i);
        println!("Thread 2 pushed: {}", i);
    }

    handle.join().unwrap();

    println!("\nFinal state:");
    list2.print_forward();
    println!("Length: {}", list2.len());
}
