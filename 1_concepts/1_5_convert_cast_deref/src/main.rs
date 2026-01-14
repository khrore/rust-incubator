use rand::Rng;
use std::borrow::Borrow;
use std::cell::Cell;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

// ============================================================================
// EmailString: A validated email address type
// ============================================================================

/// A type that guarantees valid email address format.
/// Can only be constructed through validation.
#[derive(Clone)]
pub struct EmailString(String);

/// Error type for invalid email addresses
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidEmailError {
    invalid_input: String,
    reason: &'static str,
}

impl fmt::Display for InvalidEmailError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid email '{}': {}", self.invalid_input, self.reason)
    }
}

impl Error for InvalidEmailError {}

impl EmailString {
    /// Validates an email string (simple validation for demonstration)
    fn validate(email: &str) -> Result<(), &'static str> {
        if email.is_empty() {
            return Err("email cannot be empty");
        }
        if !email.contains('@') {
            return Err("email must contain '@'");
        }
        if !email.contains('.') {
            return Err("email must contain '.'");
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err("email must have exactly one '@'");
        }
        if parts[0].is_empty() {
            return Err("local part cannot be empty");
        }
        if parts[1].is_empty() {
            return Err("domain part cannot be empty");
        }

        Ok(())
    }
}

// Fallible conversion from String (ownership-consuming)
impl TryFrom<String> for EmailString {
    type Error = InvalidEmailError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate(&value).map_err(|reason| InvalidEmailError {
            invalid_input: value.clone(),
            reason,
        })?;
        Ok(EmailString(value))
    }
}

// Fallible conversion from &str (allocates)
impl TryFrom<&str> for EmailString {
    type Error = InvalidEmailError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        EmailString::try_from(value.to_string())
    }
}

// ✅ CORRECT: AsRef for cheap reference conversion
impl AsRef<str> for EmailString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for EmailString {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

// Borrow trait for semantic equivalence with str
// This allows EmailString to be used in HashMap lookups with &str keys
impl Borrow<str> for EmailString {
    fn borrow(&self) -> &str {
        &self.0
    }
}

// Extract the inner String (consuming)
impl From<EmailString> for String {
    fn from(email: EmailString) -> String {
        email.0
    }
}

// Display for user-facing output
impl fmt::Display for EmailString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Debug for development
impl fmt::Debug for EmailString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("EmailString").field(&self.0).finish()
    }
}

// These trait implementations ensure Hash/Eq/Ord are equivalent to String
// This is required for proper Borrow semantics
impl Hash for EmailString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl PartialEq for EmailString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for EmailString {}

impl PartialOrd for EmailString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EmailString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

// ============================================================================
// Random<T>: A smart pointer that randomly selects from 3 values
// ============================================================================

/// A smart pointer that contains 3 values and returns a random one on each
/// dereference. This is a legitimate use of Deref because Random<T> IS a
/// smart pointer - it owns and provides access to T values.
pub struct Random<T> {
    values: [T; 3],
    // Cell allows interior mutability for tracking last index without &mut
    last_index: Cell<usize>,
}

impl<T> Random<T> {
    /// Creates a new Random smart pointer with three values
    pub fn new(v1: T, v2: T, v3: T) -> Self {
        Random {
            values: [v1, v2, v3],
            last_index: Cell::new(0),
        }
    }

    /// Returns a random index (0, 1, or 2)
    fn random_index(&self) -> usize {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..3)
    }

    /// Returns the index that was used in the last dereference
    pub fn last_index(&self) -> usize {
        self.last_index.get()
    }
}

// Construction from array
impl<T> From<[T; 3]> for Random<T> {
    fn from(values: [T; 3]) -> Self {
        Random {
            values,
            last_index: Cell::new(0),
        }
    }
}

// ✅ CORRECT: Deref for smart pointers
// Random<T> is a smart pointer - it owns T and provides access to it
impl<T> Deref for Random<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let idx = self.random_index();
        self.last_index.set(idx);
        &self.values[idx]
    }
}

// DerefMut allows mutable access to randomly selected value
impl<T> DerefMut for Random<T> {
    fn deref_mut(&mut self) -> &mut T {
        let idx = self.random_index();
        self.last_index.set(idx);
        &mut self.values[idx]
    }
}

// Debug for development
impl<T: fmt::Debug> fmt::Debug for Random<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Random")
            .field("values", &self.values)
            .field("last_index", &self.last_index.get())
            .finish()
    }
}

// Clone if T is Clone
impl<T: Clone> Clone for Random<T> {
    fn clone(&self) -> Self {
        Random {
            values: self.values.clone(),
            last_index: Cell::new(self.last_index.get()),
        }
    }
}

// ============================================================================
// Main: Demonstration of usage
// ============================================================================

fn main() {
    println!("=== EmailString Examples ===\n");
    demo_email_string();

    println!("\n=== Random<T> Examples ===\n");
    demo_random();
}

fn demo_email_string() {
    // Valid email construction
    match EmailString::try_from("user@example.com") {
        Ok(email) => {
            println!("✓ Created valid email: {}", email);
            println!("  Debug: {:?}", email);

            // Using AsRef<str> - the idiomatic way
            print_str_ref(email.as_ref());

            // Using in generic contexts
            process_as_ref(&email);

            // Borrow trait allows using in collections
            use std::collections::HashMap;
            let mut map = HashMap::new();
            map.insert(email.clone(), "John Doe");

            // Can look up with &str thanks to Borrow trait
            if let Some(name) = map.get("user@example.com") {
                println!("  Found in map: {}", name);
            }

            // Extract back to String
            let string: String = email.into();
            println!("  Extracted String: {}", string);
        }
        Err(e) => println!("✗ Error: {}", e),
    }

    println!();

    // Invalid email examples
    let invalid_emails = vec![
        "",
        "notanemail",
        "missing-at-sign.com",
        "@nodomain",
        "nolocal@",
        "double@@example.com",
    ];

    for invalid in invalid_emails {
        match EmailString::try_from(invalid) {
            Ok(_) => println!("✗ Should have failed: {}", invalid),
            Err(e) => println!("✓ Correctly rejected: {}", e),
        }
    }
}

fn demo_random() {
    // Random with numbers
    let rand_num = Random::new(10, 20, 30);
    println!("Random number examples:");
    for i in 1..=5 {
        let value = *rand_num; // Deref happens here
        println!(
            "  Attempt {}: {} (index: {})",
            i,
            value,
            rand_num.last_index()
        );
    }

    println!();

    // Random with strings - demonstrates Deref coercion
    let rand_string = Random::new("Hello".to_string(), "World".to_string(), "Rust".to_string());
    println!("Random string examples:");
    for _ in 1..=5 {
        // Deref coercion: Random<String> -> &String -> &str
        print_str_ref(&rand_string);
        println!("  (index: {})", rand_string.last_index());
    }

    println!();

    // Construction from array
    let rand_from_array = Random::from([100, 200, 300]);
    println!("Random from array:");
    for i in 1..=3 {
        println!("  Value {}: {}", i, *rand_from_array);
    }

    println!();

    // Mutable dereferencing
    let mut rand_mut = Random::new(1, 2, 3);
    println!("Mutable deref example:");
    println!("  Before: {:?}", rand_mut);
    *rand_mut *= 10; // DerefMut allows mutation
    println!("  After multiplying by 10:");
    for _ in 1..=3 {
        println!("    Value: {}", *rand_mut);
    }
}

// Helper functions to demonstrate conversions

fn print_str_ref(s: &str) {
    println!("  String reference: '{}'", s);
}

fn process_as_ref<S: AsRef<str>>(s: S) {
    println!("  Generic AsRef: '{}'", s.as_ref());
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // EmailString tests
    #[test]
    fn email_string_valid() {
        assert!(EmailString::try_from("user@example.com").is_ok());
        assert!(EmailString::try_from("test.user@sub.domain.com").is_ok());
        assert!(EmailString::try_from("a@b.c").is_ok());
    }

    #[test]
    fn email_string_invalid() {
        assert!(EmailString::try_from("").is_err());
        assert!(EmailString::try_from("notanemail").is_err());
        assert!(EmailString::try_from("missing@domain").is_err());
        assert!(EmailString::try_from("@nodomain.com").is_err());
        assert!(EmailString::try_from("noat.com").is_err());
    }

    #[test]
    fn email_string_as_ref() {
        let email = EmailString::try_from("test@test.com").unwrap();
        let s: &str = email.as_ref();
        assert_eq!(s, "test@test.com");

        let bytes: &[u8] = email.as_ref();
        assert_eq!(bytes, b"test@test.com");
    }

    #[test]
    fn email_string_borrow() {
        let email = EmailString::try_from("test@test.com").unwrap();
        let borrowed: &str = email.borrow();
        assert_eq!(borrowed, "test@test.com");
    }

    #[test]
    fn email_string_display() {
        let email = EmailString::try_from("display@test.com").unwrap();
        assert_eq!(format!("{}", email), "display@test.com");
    }

    #[test]
    fn email_string_into_string() {
        let email = EmailString::try_from("convert@test.com").unwrap();
        let string: String = email.into();
        assert_eq!(string, "convert@test.com");
    }

    #[test]
    fn email_string_equality() {
        let email1 = EmailString::try_from("same@test.com").unwrap();
        let email2 = EmailString::try_from("same@test.com").unwrap();
        let email3 = EmailString::try_from("different@test.com").unwrap();

        assert_eq!(email1, email2);
        assert_ne!(email1, email3);
    }

    #[test]
    fn email_string_hash_map() {
        use std::collections::HashMap;

        let email = EmailString::try_from("key@test.com").unwrap();
        let mut map = HashMap::new();
        map.insert(email, "value");

        // Can look up with &str thanks to Borrow
        assert_eq!(map.get("key@test.com"), Some(&"value"));
    }

    // Random<T> tests
    #[test]
    fn random_returns_valid_values() {
        let rand = Random::new(1, 2, 3);
        for _ in 0..20 {
            let val = *rand;
            assert!(val == 1 || val == 2 || val == 3);
        }
    }

    #[test]
    fn random_deref_works() {
        let rand = Random::new("a".to_string(), "b".to_string(), "c".to_string());
        // Deref to &String, then to &str
        let s: &str = &rand;
        assert!(s == "a" || s == "b" || s == "c");
    }

    #[test]
    fn random_from_array() {
        let rand = Random::from([10, 20, 30]);
        let val = *rand;
        assert!(val == 10 || val == 20 || val == 30);
    }

    #[test]
    fn random_deref_mut() {
        let mut rand = Random::new(1, 2, 3);
        *rand = 100; // This modifies one of the three values

        // At least one value should now be 100
        let mut found_modified = false;
        for _ in 0..20 {
            if *rand == 100 {
                found_modified = true;
                break;
            }
        }
        assert!(found_modified);
    }

    #[test]
    fn random_last_index_tracking() {
        let rand = Random::new(10, 20, 30);
        let _ = *rand;
        let idx = rand.last_index();
        assert!(idx < 3);
    }

    #[test]
    fn random_clone() {
        let rand1 = Random::new(1, 2, 3);
        let rand2 = rand1.clone();

        // Both should return valid values
        let val1 = *rand1;
        let val2 = *rand2;
        assert!(val1 >= 1 && val1 <= 3);
        assert!(val2 >= 1 && val2 <= 3);
    }
}
