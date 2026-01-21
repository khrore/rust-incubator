use std::marker::PhantomData;

// ===== Newtype Pattern: Domain Types =====

mod post {
    /// Post identifier - wraps u64 to prevent mixing with user IDs
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Id(u64);

    impl Id {
        pub fn new(id: u64) -> Self {
            Self(id)
        }

        pub fn get(&self) -> u64 {
            self.0
        }
    }

    /// Post title - wraps String to prevent mixing with body content
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Title(String);

    impl Title {
        pub fn new(title: String) -> Self {
            Self(title)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<String> for Title {
        fn from(s: String) -> Self {
            Self::new(s)
        }
    }

    impl From<&str> for Title {
        fn from(s: &str) -> Self {
            Self::new(s.to_string())
        }
    }

    /// Post body content - wraps String to prevent mixing with title
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Body(String);

    impl Body {
        pub fn new(body: String) -> Self {
            Self(body)
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<String> for Body {
        fn from(s: String) -> Self {
            Self::new(s)
        }
    }

    impl From<&str> for Body {
        fn from(s: &str) -> Self {
            Self::new(s.to_string())
        }
    }
}

mod user {
    /// User identifier - wraps u64 to prevent mixing with post IDs
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Id(u64);

    impl Id {
        pub fn new(id: u64) -> Self {
            Self(id)
        }

        pub fn get(&self) -> u64 {
            self.0
        }
    }
}

// ===== Typestate Pattern: State Markers =====

/// Draft state - post is being created
struct New;

/// Awaiting moderation state - post has been published but not yet approved
struct Unmoderated;

/// Live state - post has been approved and is visible to users
struct Published;

/// Removed state - post has been deleted (terminal state)
struct Deleted;

// ===== Post Entity with Typestate =====

/// Blog post with compile-time state guarantees
///
/// State transitions:
/// ```
/// New --publish()--> Unmoderated --allow()--> Published
///                         |                       |
///                      deny()                 delete()
///                         |                       |
///                         +-------> Deleted <-----+
/// ```
struct Post<State> {
    id: post::Id,
    user_id: user::Id,
    title: post::Title,
    body: post::Body,
    _state: PhantomData<State>,
}

// ===== State-Specific Implementations =====

/// Methods available only for New posts
impl Post<New> {
    /// Creates a new draft post
    pub fn new(id: post::Id, user_id: user::Id, title: post::Title, body: post::Body) -> Self {
        Post {
            id,
            user_id,
            title,
            body,
            _state: PhantomData,
        }
    }

    /// Publishes the post for moderation
    ///
    /// Transitions: New → Unmoderated
    pub fn publish(self) -> Post<Unmoderated> {
        Post {
            id: self.id,
            user_id: self.user_id,
            title: self.title,
            body: self.body,
            _state: PhantomData,
        }
    }
}

/// Methods available only for Unmoderated posts
impl Post<Unmoderated> {
    /// Approves the post and makes it visible to users
    ///
    /// Transitions: Unmoderated → Published
    pub fn allow(self) -> Post<Published> {
        Post {
            id: self.id,
            user_id: self.user_id,
            title: self.title,
            body: self.body,
            _state: PhantomData,
        }
    }

    /// Rejects the post during moderation
    ///
    /// Transitions: Unmoderated → Deleted
    pub fn deny(self) -> Post<Deleted> {
        Post {
            id: self.id,
            user_id: self.user_id,
            title: self.title,
            body: self.body,
            _state: PhantomData,
        }
    }
}

/// Methods available only for Published posts
impl Post<Published> {
    /// Removes a published post
    ///
    /// Transitions: Published → Deleted
    pub fn delete(self) -> Post<Deleted> {
        Post {
            id: self.id,
            user_id: self.user_id,
            title: self.title,
            body: self.body,
            _state: PhantomData,
        }
    }
}

/// Deleted is a terminal state - no transition methods available
impl Post<Deleted> {
    // No state transitions from Deleted state
}

// ===== Shared Methods (Available in All States) =====

impl<State> Post<State> {
    /// Returns the post ID
    pub fn id(&self) -> post::Id {
        self.id
    }

    /// Returns the user ID of the post author
    pub fn user_id(&self) -> user::Id {
        self.user_id
    }

    /// Returns the post title as a string slice
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns the post body as a string slice
    pub fn body(&self) -> &str {
        self.body.as_str()
    }
}

fn main() {
    println!("=== Typestate Pattern Demonstration ===\n");

    // Create a new draft post
    let post = Post::<New>::new(
        post::Id::new(1),
        user::Id::new(42),
        post::Title::from("Understanding Rust Typestates"),
        post::Body::from("Typestates encode state machines in the type system..."),
    );

    println!("✓ Created draft post: '{}'", post.title());
    println!("  Author ID: {}", post.user_id().get());
    println!("  Post ID: {}\n", post.id().get());

    // Valid transition: New → Unmoderated
    let unmoderated = post.publish();
    println!("✓ Published post for moderation\n");

    // Demonstrate the deny path
    let denied_post = Post::<New>::new(
        post::Id::new(2),
        user::Id::new(99),
        post::Title::from("Spam Post"),
        post::Body::from("Buy now!"),
    );
    let denied_moderated = denied_post.publish();
    let _denied_deleted = denied_moderated.deny();
    println!("✓ Denied spam post during moderation\n");

    // Valid transition: Unmoderated → Published
    let published = unmoderated.allow();
    println!("✓ Approved and published post\n");

    // Valid transition: Published → Deleted
    let _deleted = published.delete();
    println!("✓ Deleted published post\n");

    println!("=== Compile-Time Safety Guarantees ===\n");
    println!("The following code will NOT compile:");
    println!("  let draft = Post::<New>::new(...);");
    println!("  draft.delete();  // ❌ Error: no method `delete` for Post<New>");
    println!("  draft.allow();   // ❌ Error: no method `allow` for Post<New>\n");

    println!("  let deleted = published.delete();");
    println!("  deleted.deny();  // ❌ Error: no method `deny` for Post<Deleted>\n");

    println!("  let published = unmoderated.allow();");
    println!("  published.publish();  // ❌ Error: no method `publish` for Post<Published>\n");

    // Demonstrate newtype safety
    println!("=== Newtype Pattern Safety ===\n");
    println!("This code will NOT compile:");
    println!("  let post_id = post::Id::new(1);");
    println!("  let user_id = user::Id::new(42);");
    println!("  let post = Post::new(user_id, post_id, ...);");
    println!("  // ❌ Error: mismatched types - expected post::Id, found user::Id\n");

    // This won't compile - demonstrates type safety:
    // let wrong_ids = Post::<New>::new(
    //     user::Id::new(42),  // ❌ Expected post::Id, found user::Id
    //     post::Id::new(1),   // ❌ Expected user::Id, found post::Id
    //     post::Title::from("Wrong"),
    //     post::Body::from("Types prevent this!"),
    // );
}
