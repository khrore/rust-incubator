use std::borrow::Cow;
use std::collections::HashMap;
use thiserror::Error;

// ============================================================================
// Error Types - Type-Safe Error Handling (Best Practice #1)
// ============================================================================

/// Domain errors for user operations
///
/// Using enums instead of String provides:
/// - Compile-time exhaustiveness checking
/// - Pattern matching without string parsing
/// - Better performance (no allocations for known errors)
/// - Structured error information
#[derive(Debug, Error, PartialEq, Eq)]
pub enum UserError {
    #[error("User with ID {0} already exists")]
    AlreadyExists(u64),

    #[error("User with ID {0} not found")]
    NotFound(u64),

    #[error("Invalid email address: {0}")]
    InvalidEmail(String),

    #[error("Invalid user ID: {0}")]
    InvalidId(u64),
}

// ============================================================================
// Domain Types
// ============================================================================

/// User entity with unique ID, email, and activation status
///
/// In production web/blockchain apps, this would include:
/// - Timestamps (created_at, updated_at)
/// - Versioning for optimistic locking
/// - Audit fields (created_by, updated_by)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    id: u64,
    email: Cow<'static, str>,
    activated: bool,
}

impl User {
    /// Create a new user with validation
    ///
    /// Smart constructor pattern ensures invariants at construction time
    pub fn new(id: u64, email: impl Into<Cow<'static, str>>) -> Result<Self, UserError> {
        let email = email.into();

        // Validate email (production would use proper email validation crate)
        if !email.contains('@') || email.len() > 255 || email.is_empty() {
            return Err(UserError::InvalidEmail(email.to_string()));
        }

        // Validate ID (example: reject ID 0 as invalid)
        if id == 0 {
            return Err(UserError::InvalidId(id));
        }

        Ok(Self {
            id,
            email,
            activated: false,
        })
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn is_activated(&self) -> bool {
        self.activated
    }

    pub fn activate(&mut self) {
        self.activated = true;
    }
}

// ============================================================================
// Repository Trait - Infrastructure Abstraction (Best Practice #2)
// ============================================================================

/// Repository pattern for User persistence
///
/// This trait is object-safe, allowing it to be used as `dyn UserRepository`
/// Key benefits:
/// - Testability: Mock implementations without databases
/// - Flexibility: Swap PostgreSQL, Redis, in-memory implementations
/// - Hexagonal architecture: Domain logic independent of infrastructure
///
/// Object safety requirements met:
/// - No Self: Sized bound
/// - No generic methods
/// - All methods have &self or &mut self receivers
/// - No associated constants
pub trait UserRepository {
    /// Add a new user to the repository
    fn add(&mut self, user: User) -> Result<(), UserError>;

    /// Retrieve a user by ID
    fn get(&self, id: u64) -> Option<&User>;

    /// Update an existing user
    fn update(&mut self, user: User) -> Result<(), UserError>;

    /// Remove a user by ID
    fn remove(&mut self, id: u64) -> Option<User>;

    /// Check if a user exists
    fn exists(&self, id: u64) -> bool {
        self.get(id).is_some()
    }
}

// ============================================================================
// Command Pattern - CQRS/Event Sourcing (Best Practice #3)
// ============================================================================

/// Command to create a new user
///
/// In production systems, commands:
/// - Carry validation metadata (request ID, timestamp, actor)
/// - Implement idempotency tokens
/// - Support serialization for message queues (Kafka, RabbitMQ)
#[derive(Debug, Clone)]
pub struct CreateUser {
    pub id: u64,
    pub email: String,
}

impl CreateUser {
    /// Smart constructor with validation
    ///
    /// Validates at the command boundary, not in the handler
    /// Follows "parse, don't validate" principle
    pub fn new(id: u64, email: String) -> Result<Self, UserError> {
        // Validate email format
        if !email.contains('@') || email.len() > 255 || email.is_empty() {
            return Err(UserError::InvalidEmail(email));
        }

        // Validate ID
        if id == 0 {
            return Err(UserError::InvalidId(id));
        }

        Ok(Self { id, email })
    }
}

/// Marker trait for commands
pub trait Command {}
impl Command for CreateUser {}

// ============================================================================
// Command Handler Trait - The ?Sized Pattern (Best Practice #4)
// ============================================================================

/// Command handler pattern with flexible context
///
/// The `?Sized` bound on `Context` is CRITICAL here:
/// - Allows `type Context = dyn UserRepository` (trait object)
/// - Also allows `type Context = InMemoryUserRepository` (concrete type)
/// - Enables dependency injection with both static and dynamic dispatch
///
/// Without `?Sized`, we could ONLY use sized types (no trait objects)
pub trait CommandHandler<C: Command> {
    /// Associated type with ?Sized bound
    ///
    /// This is the key pattern from Task 1_7:
    /// - `?Sized` lifts the implicit `Sized` bound
    /// - Enables using `dyn Trait` as the context type
    /// - Essential for testability and plugin architectures
    type Context: ?Sized;
    type Result;

    fn handle_command(&self, cmd: &C, ctx: &Self::Context) -> Self::Result;
}

// ============================================================================
// Command Handler Implementation (Best Practice #5)
// ============================================================================

/// Handler for CreateUser command
///
/// Note the use of `dyn UserRepository` - this is only possible because
/// we used `type Context: ?Sized` in the trait definition
impl CommandHandler<CreateUser> for User {
    type Context = dyn UserRepository; // ← Trait object! Requires ?Sized
    type Result = Result<(), UserError>;

    fn handle_command(&self, cmd: &CreateUser, ctx: &Self::Context) -> Self::Result {
        // Check if user already exists
        if ctx.exists(cmd.id) {
            return Err(UserError::AlreadyExists(cmd.id));
        }

        // Create user from command
        // Command is already validated, so we can create user
        let _user = User::new(cmd.id, cmd.email.clone())?;

        // Note: In production, we'd need &mut ctx to actually add the user
        // Real implementations would:
        // - Use interior mutability (RefCell/Mutex) in the repository
        // - Make handle_command take &mut self and &mut ctx
        // - Use async with Arc<Mutex<>> for concurrent access
        // For this demonstration, we validate but don't persist

        Ok(())
    }
}

// ============================================================================
// Concrete Repository Implementation - Production (Best Practice #6)
// ============================================================================

/// In-memory user repository
///
/// Production implementations would use:
/// - PostgreSQL: sqlx, diesel, sea-orm
/// - Redis: redis-rs with connection pooling
/// - Blockchain: substrate storage, solana accounts
#[derive(Default)]
pub struct InMemoryUserRepository {
    users: HashMap<u64, User>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            users: HashMap::with_capacity(capacity),
        }
    }

    pub fn count(&self) -> usize {
        self.users.len()
    }
}

impl UserRepository for InMemoryUserRepository {
    fn add(&mut self, user: User) -> Result<(), UserError> {
        if self.users.contains_key(&user.id) {
            return Err(UserError::AlreadyExists(user.id));
        }
        self.users.insert(user.id, user);
        Ok(())
    }

    fn get(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    fn update(&mut self, user: User) -> Result<(), UserError> {
        if !self.users.contains_key(&user.id) {
            return Err(UserError::NotFound(user.id));
        }
        self.users.insert(user.id, user);
        Ok(())
    }

    fn remove(&mut self, id: u64) -> Option<User> {
        self.users.remove(&id)
    }
}

// ============================================================================
// Mock Repository - Testing (Best Practice #7)
// ============================================================================

/// Mock repository for testing
///
/// Key advantages over real database:
/// - No Docker/external dependencies in tests
/// - Deterministic behavior
/// - Fast test execution (<1ms vs 100ms+ for DB)
/// - Can simulate error conditions easily
#[derive(Default)]
pub struct MockUserRepository {
    users: HashMap<u64, User>,
    pub call_count_add: usize,
    pub call_count_get: usize,
    pub should_fail_add: bool,
}

impl MockUserRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_users(users: Vec<User>) -> Self {
        let mut repo = Self::new();
        for user in users {
            repo.users.insert(user.id, user);
        }
        repo
    }
}

impl UserRepository for MockUserRepository {
    fn add(&mut self, user: User) -> Result<(), UserError> {
        self.call_count_add += 1;

        if self.should_fail_add {
            return Err(UserError::AlreadyExists(user.id));
        }

        if self.users.contains_key(&user.id) {
            return Err(UserError::AlreadyExists(user.id));
        }

        self.users.insert(user.id, user);
        Ok(())
    }

    fn get(&self, id: u64) -> Option<&User> {
        // Note: We can't increment counter in &self method
        // Production mocks would use RefCell<usize> for interior mutability
        self.users.get(&id)
    }

    fn update(&mut self, user: User) -> Result<(), UserError> {
        if !self.users.contains_key(&user.id) {
            return Err(UserError::NotFound(user.id));
        }
        self.users.insert(user.id, user);
        Ok(())
    }

    fn remove(&mut self, id: u64) -> Option<User> {
        self.users.remove(&id)
    }
}

// ============================================================================
// Main Function - Demonstration
// ============================================================================

fn main() {
    println!("=== Task 1.7: ?Sized and Command Pattern Demo ===\n");

    // 1. Demonstrate trait object usage (requires ?Sized)
    println!("1. Using trait object (dyn UserRepository):");
    let mut repo = InMemoryUserRepository::new();

    let user = User::new(1, "alice@example.com").unwrap();
    repo.add(user.clone()).unwrap();
    println!("   Added user: {:?}", user);

    // Use trait object after mutable operations
    let repo_ref: &dyn UserRepository = &repo;
    println!("   Retrieved: {:?}\n", repo_ref.get(1));

    // TODO(human): Implement command handler demonstration
    // 2. Demonstrate command handler pattern
    println!("2. Command Handler Pattern:");
    println!("   Note: Full handler execution requires mutable context");
    println!("   See tests for complete working examples\n");
    let cmd1 = CreateUser::new(2, "some@some.some".to_owned()).unwrap();
    let handler = User::new(10, "some@some.com".to_owned()).unwrap();
    match handler.handle_command(&cmd1, repo_ref) {
        Ok(()) => println!("OK"),
        Err(e) => println!("Err: {e:?}"),
    }

    let cmd2 = CreateUser::new(1, "some@some.some".to_owned()).unwrap();
    match handler.handle_command(&cmd2, repo_ref) {
        Ok(()) => println!("OK"),
        Err(e) => println!("Err: {e:?}"),
    }

    // 3. Demonstrate validation
    println!("3. Validation at Command Boundary:");
    match CreateUser::new(0, "invalid@example.com".to_string()) {
        Ok(_) => println!("   Command created"),
        Err(e) => println!("   Validation failed: {}", e),
    }

    match CreateUser::new(2, "no-at-sign".to_string()) {
        Ok(_) => println!("   Command created"),
        Err(e) => println!("   Validation failed: {}", e),
    }

    match CreateUser::new(3, "valid@example.com".to_string()) {
        Ok(_) => println!("   ✓ Valid command created"),
        Err(e) => println!("   Validation failed: {}", e),
    }

    println!("\n=== Key Lessons ===");
    println!("✓ ?Sized enables trait object contexts");
    println!("✓ Type-safe errors with thiserror");
    println!("✓ Command pattern for CQRS architectures");
    println!("✓ Repository pattern for testability");
    println!("✓ Smart constructors enforce invariants");
}

// ============================================================================
// Tests - Demonstrating Trait Object Usage (Best Practice #8)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test trait object usage with concrete implementation
    #[test]
    fn test_trait_object_with_concrete_repo() {
        let mut repo = InMemoryUserRepository::new();
        let user = User::new(1, "test@example.com").unwrap();

        repo.add(user.clone()).unwrap();

        // Use as trait object
        let repo_ref: &dyn UserRepository = &repo;
        assert_eq!(repo_ref.get(1), Some(&user));
    }

    // Test trait object usage with mock
    #[test]
    fn test_trait_object_with_mock() {
        let mut mock = MockUserRepository::new();
        let user = User::new(1, "test@example.com").unwrap();

        mock.add(user.clone()).unwrap();

        // Use as trait object
        let mock_ref: &dyn UserRepository = &mock;
        assert_eq!(mock_ref.get(1), Some(&user));
    }

    // Test heterogeneous collection (only possible with trait objects)
    #[test]
    fn test_heterogeneous_repository_collection() {
        let mut concrete = InMemoryUserRepository::new();
        let mut mock = MockUserRepository::new();

        let user1 = User::new(1, "user1@example.com").unwrap();
        let user2 = User::new(2, "user2@example.com").unwrap();

        concrete.add(user1.clone()).unwrap();
        mock.add(user2.clone()).unwrap();

        // Store different implementations in same collection
        let repos: Vec<&dyn UserRepository> = vec![&concrete, &mock];

        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].get(1), Some(&user1));
        assert_eq!(repos[1].get(2), Some(&user2));
    }

    // Test command validation
    mod command_validation {
        use super::*;

        #[test]
        fn test_valid_command() {
            let cmd = CreateUser::new(1, "valid@example.com".to_string());
            assert!(cmd.is_ok());
        }

        #[test]
        fn test_invalid_email_no_at() {
            let result = CreateUser::new(1, "invalid.com".to_string());
            assert_eq!(
                result.unwrap_err(),
                UserError::InvalidEmail("invalid.com".to_string())
            );
        }

        #[test]
        fn test_invalid_email_empty() {
            let result = CreateUser::new(1, "".to_string());
            assert!(matches!(result.unwrap_err(), UserError::InvalidEmail(_)));
        }

        #[test]
        fn test_invalid_id_zero() {
            let result = CreateUser::new(0, "valid@example.com".to_string());
            assert_eq!(result.unwrap_err(), UserError::InvalidId(0));
        }
    }

    // Test repository operations with mock
    mod repository_operations {
        use super::*;

        #[test]
        fn test_add_and_get() {
            let mut mock = MockUserRepository::new();
            let user = User::new(1, "test@example.com").unwrap();

            mock.add(user.clone()).unwrap();

            assert_eq!(mock.call_count_add, 1);
            assert_eq!(mock.get(1), Some(&user));
        }

        #[test]
        fn test_add_duplicate_fails() {
            let mut mock = MockUserRepository::new();
            let user = User::new(1, "test@example.com").unwrap();

            mock.add(user.clone()).unwrap();
            let result = mock.add(user);

            assert_eq!(result.unwrap_err(), UserError::AlreadyExists(1));
            assert_eq!(mock.call_count_add, 2);
        }

        #[test]
        fn test_update_existing_user() {
            let mut mock = MockUserRepository::new();
            let mut user = User::new(1, "test@example.com").unwrap();

            mock.add(user.clone()).unwrap();
            user.activate();
            mock.update(user.clone()).unwrap();

            let retrieved = mock.get(1).unwrap();
            assert!(retrieved.is_activated());
        }

        #[test]
        fn test_update_nonexistent_fails() {
            let mut mock = MockUserRepository::new();
            let user = User::new(999, "test@example.com").unwrap();

            let result = mock.update(user);
            assert_eq!(result.unwrap_err(), UserError::NotFound(999));
        }

        #[test]
        fn test_remove_user() {
            let mut mock = MockUserRepository::new();
            let user = User::new(1, "test@example.com").unwrap();

            mock.add(user.clone()).unwrap();
            let removed = mock.remove(1);

            assert_eq!(removed, Some(user));
            assert_eq!(mock.get(1), None);
        }

        #[test]
        fn test_exists() {
            let mut mock = MockUserRepository::new();
            let user = User::new(1, "test@example.com").unwrap();

            assert!(!mock.exists(1));
            mock.add(user).unwrap();
            assert!(mock.exists(1));
        }
    }

    // Test mock behavior control
    mod mock_behavior {
        use super::*;

        #[test]
        fn test_mock_can_simulate_failures() {
            let mut mock = MockUserRepository::new();
            mock.should_fail_add = true;

            let user = User::new(1, "test@example.com").unwrap();
            let result = mock.add(user);

            assert!(result.is_err());
            assert_eq!(mock.call_count_add, 1);
        }

        #[test]
        fn test_mock_with_prepopulated_data() {
            let users = vec![
                User::new(1, "user1@example.com").unwrap(),
                User::new(2, "user2@example.com").unwrap(),
            ];

            let mock = MockUserRepository::with_users(users.clone());

            assert_eq!(mock.get(1), Some(&users[0]));
            assert_eq!(mock.get(2), Some(&users[1]));
        }
    }

    // Test user entity validation
    mod user_validation {
        use super::*;

        #[test]
        fn test_valid_user_creation() {
            let user = User::new(1, "valid@example.com");
            assert!(user.is_ok());
        }

        #[test]
        fn test_invalid_email() {
            let result = User::new(1, "no-at-sign");
            assert!(matches!(result.unwrap_err(), UserError::InvalidEmail(_)));
        }

        #[test]
        fn test_invalid_id() {
            let result = User::new(0, "valid@example.com");
            assert_eq!(result.unwrap_err(), UserError::InvalidId(0));
        }

        #[test]
        fn test_user_activation() {
            let mut user = User::new(1, "test@example.com").unwrap();
            assert!(!user.is_activated());

            user.activate();
            assert!(user.is_activated());
        }
    }
}
