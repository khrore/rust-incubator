use std::{
    borrow::{Borrow, BorrowMut},
    fmt,
    num::NonZeroU64,
};

fn main() {
    println!("Refactored with production best practices!");

    // Demonstrate the improvements
    demonstrate_improvements();
}

fn demonstrate_improvements() {
    // Example aggregate
    #[derive(Default)]
    struct UserAggregate {
        name: String,
        email: String,
    }

    impl Aggregate for UserAggregate {
        fn aggregate_type() -> &'static str {
            "User"
        }
    }

    // Example event
    struct UserCreated {
        name: String,
        email: String,
    }

    impl Event for UserCreated {
        fn event_type(&self) -> &'static str {
            "UserCreated"
        }
    }

    impl AggregateEvent<UserAggregate> for UserCreated {
        fn apply_to(self, aggregate: &mut UserAggregate) {
            aggregate.name = self.name;
            aggregate.email = self.email;
        }
    }

    // Create and apply events
    let mut hydrated = HydratedAggregate::<UserAggregate>::default();
    let event = UserCreated {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    match hydrated.apply(event) {
        Ok(_) => println!("✅ Event applied successfully"),
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("Current version: {:?}", hydrated.version());
}

// ============================================================================
// Error Types - Production-Grade Error Handling
// ============================================================================

/// Errors that can occur during aggregate operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregateError {
    /// Version number overflow (2^64 events reached).
    VersionOverflow,

    /// Concurrency conflict detected.
    VersionMismatch { expected: Version, actual: Version },

    /// Event sequence has gaps or is out of order.
    InvalidSequence,
}

impl fmt::Display for AggregateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregateError::VersionOverflow => {
                write!(f, "Version overflow: maximum event count reached")
            }
            AggregateError::VersionMismatch { expected, actual } => {
                write!(
                    f,
                    "Version mismatch: expected {:?}, got {:?}",
                    expected, actual
                )
            }
            AggregateError::InvalidSequence => {
                write!(f, "Invalid event sequence detected")
            }
        }
    }
}

impl std::error::Error for AggregateError {}

// ============================================================================
// Core Traits - Behavior Definitions
// ============================================================================

/// A projected state built from a series of events.
///
/// This trait represents the core behavior of an event-sourced aggregate.
/// Aggregates are rebuilt by replaying events in sequence.
pub trait Aggregate: Default {
    /// A static string representing the type of the aggregate.
    ///
    /// Note: This should effectively be a constant value, and should never change.
    fn aggregate_type() -> &'static str;

    /// Consumes the event, applying its effects to the aggregate.
    fn apply<E>(&mut self, event: E)
    where
        E: AggregateEvent<Self>,
    {
        event.apply_to(self);
    }
}

/// An identifier for an aggregate.
///
/// REFACTORED: Removed `A: Aggregate` bound - IDs don't need to know about
/// aggregate behavior. This makes the trait more reusable and reduces coupling.
pub trait AggregateId {
    /// Gets the stringified aggregate identifier.
    fn as_str(&self) -> &str;
}

/// A thing that happened.
///
/// Events are immutable facts about what occurred in the system.
pub trait Event {
    /// A static description of the event.
    fn event_type(&self) -> &'static str;
}

/// An event that can be applied to an aggregate.
///
/// This trait links events to the aggregates they affect.
pub trait AggregateEvent<A: Aggregate>: Event {
    /// Consumes the event, applying its effects to the aggregate.
    fn apply_to(self, aggregate: &mut A);
}

// ============================================================================
// EventNumber - Safe Version Tracking
// ============================================================================

/// Represents an event sequence number, starting at 1.
///
/// REFACTORED: Removed derive macros to avoid adding unnecessary trait bounds
/// on inner type. All implementations are now explicit and minimal.
pub struct EventNumber(NonZeroU64);

impl EventNumber {
    /// The minimum [EventNumber].
    pub const MIN_VALUE: EventNumber = EventNumber(
        // SAFETY: One is absolutely non-zero. We use unwrap() here which is
        // now safe in const contexts.
        match NonZeroU64::new(1) {
            Some(v) => v,
            None => unreachable!(),
        },
    );

    /// Increments the event number to the next value.
    ///
    /// REFACTORED: Returns Result instead of panicking on overflow.
    /// This is critical for production systems - we must never panic on overflow.
    ///
    /// # Errors
    ///
    /// Returns `AggregateError::VersionOverflow` if incrementing would overflow.
    #[inline]
    pub fn incr(&mut self) -> Result<(), AggregateError> {
        let next = self
            .0
            .get()
            .checked_add(1)
            .ok_or(AggregateError::VersionOverflow)?;
        self.0 = NonZeroU64::new(next).ok_or(AggregateError::VersionOverflow)?;
        Ok(())
    }

    /// Gets the raw u64 value.
    #[inline]
    pub fn get(&self) -> u64 {
        self.0.get()
    }
}

// Manual trait implementations to avoid unnecessary bounds

impl Clone for EventNumber {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for EventNumber {}

impl fmt::Debug for EventNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EventNumber").field(&self.0.get()).finish()
    }
}

impl PartialEq for EventNumber {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for EventNumber {}

impl PartialOrd for EventNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EventNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl std::hash::Hash for EventNumber {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

// ============================================================================
// Version - Aggregate Version Tracking
// ============================================================================

/// An aggregate version.
///
/// Tracks whether an aggregate is in its initial state or has had events applied.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Version {
    /// The version of an aggregate that has not had any events applied to it.
    Initial,
    /// The version of the last event applied to the aggregate.
    Number(EventNumber),
}

impl Default for Version {
    #[inline]
    fn default() -> Self {
        Version::Initial
    }
}

impl Version {
    /// Creates a new `Version` from a number.
    ///
    /// The number `0` gets interpreted as being `Version::Initial`, while any
    /// other number is interpreted as the latest event number applied.
    #[inline]
    pub fn new(number: u64) -> Self {
        NonZeroU64::new(number)
            .map(EventNumber)
            .map(Version::Number)
            .unwrap_or(Version::Initial)
    }

    /// Increments the version number to the next in sequence.
    ///
    /// REFACTORED: Returns Result to propagate overflow errors.
    ///
    /// # Errors
    ///
    /// Returns `AggregateError::VersionOverflow` if incrementing would overflow.
    #[inline]
    pub fn incr(&mut self) -> Result<(), AggregateError> {
        match *self {
            Version::Initial => {
                *self = Version::Number(EventNumber::MIN_VALUE);
                Ok(())
            }
            Version::Number(ref mut en) => en.incr(),
        }
    }

    /// Validates that this version is the expected next version after `previous`.
    ///
    /// This is critical for detecting concurrent modifications in distributed systems.
    ///
    /// # Errors
    ///
    /// Returns `AggregateError::InvalidSequence` if the version sequence is invalid.
    pub fn validate_sequence(&self, previous: Version) -> Result<(), AggregateError> {
        match (previous, *self) {
            (Version::Initial, Version::Number(n)) if n == EventNumber::MIN_VALUE => Ok(()),
            (Version::Number(prev), Version::Number(curr)) if curr.get() == prev.get() + 1 => {
                Ok(())
            }
            _ => Err(AggregateError::InvalidSequence),
        }
    }
}

// ============================================================================
// HydratedAggregate - Event-Sourced Aggregate with Version Tracking
// ============================================================================

/// An aggregate that has been loaded from a source, which keeps track of the
/// version of its last snapshot and the current version of the aggregate.
///
/// REFACTORED: Removed derive macros and trait bounds from type definition.
/// This allows the type to be used in more contexts and prevents trait bound
/// pollution in impl blocks.
pub struct HydratedAggregate<A> {
    version: Version,
    snapshot_version: Option<Version>,
    state: A,
}

// Manual implementations without unnecessary bounds on A

impl<A> Clone for HydratedAggregate<A>
where
    A: Clone,
{
    fn clone(&self) -> Self {
        HydratedAggregate {
            version: self.version,
            snapshot_version: self.snapshot_version,
            state: self.state.clone(),
        }
    }
}

impl<A> Copy for HydratedAggregate<A> where A: Copy {}

impl<A> fmt::Debug for HydratedAggregate<A>
where
    A: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HydratedAggregate")
            .field("version", &self.version)
            .field("snapshot_version", &self.snapshot_version)
            .field("state", &self.state)
            .finish()
    }
}

impl<A> Default for HydratedAggregate<A>
where
    A: Default,
{
    fn default() -> Self {
        HydratedAggregate {
            version: Version::default(),
            snapshot_version: None,
            state: A::default(),
        }
    }
}

impl<A> PartialEq for HydratedAggregate<A>
where
    A: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.snapshot_version == other.snapshot_version
            && self.state == other.state
    }
}

impl<A> Eq for HydratedAggregate<A> where A: Eq {}

impl<A> std::hash::Hash for HydratedAggregate<A>
where
    A: std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.version.hash(state);
        self.snapshot_version.hash(state);
        self.state.hash(state);
    }
}

// Behavior-specific implementations - only bound where needed

impl<A> HydratedAggregate<A>
where
    A: Aggregate,
{
    /// Applies a sequence of events to the internal aggregate.
    ///
    /// REFACTORED: Returns Result to propagate errors instead of panicking.
    ///
    /// # Errors
    ///
    /// Returns `AggregateError` if version overflow occurs or events are invalid.
    pub fn apply_events<E, I>(&mut self, events: I) -> Result<(), AggregateError>
    where
        E: AggregateEvent<A>,
        I: IntoIterator<Item = E>,
    {
        for event in events {
            self.apply(event)?;
        }
        Ok(())
    }

    /// Applies a single event to the aggregate, keeping track of the new aggregate version.
    ///
    /// REFACTORED: Returns Result to propagate version overflow errors.
    ///
    /// # Errors
    ///
    /// Returns `AggregateError::VersionOverflow` if applying would overflow the version.
    pub fn apply<E>(&mut self, event: E) -> Result<(), AggregateError>
    where
        E: AggregateEvent<A>,
    {
        self.state.apply(event);
        self.version.incr()?;
        Ok(())
    }

    /// Applies an event with optimistic locking.
    ///
    /// This is critical for concurrent event sourcing systems. It ensures that
    /// events are only applied if the aggregate is at the expected version,
    /// preventing the "lost update" problem.
    ///
    /// # Errors
    ///
    /// Returns `AggregateError::VersionMismatch` if the current version doesn't
    /// match the expected version, indicating a concurrent modification.
    pub fn apply_with_concurrency_check<E>(
        &mut self,
        event: E,
        expected_version: Version,
    ) -> Result<(), AggregateError>
    where
        E: AggregateEvent<A>,
    {
        if self.version != expected_version {
            return Err(AggregateError::VersionMismatch {
                expected: expected_version,
                actual: self.version,
            });
        }
        self.apply(event)
    }
}

// Methods that don't require Aggregate bound

impl<A> HydratedAggregate<A> {
    /// Creates a new hydrated aggregate from a state.
    pub fn new(state: A) -> Self {
        HydratedAggregate {
            version: Version::Initial,
            snapshot_version: None,
            state,
        }
    }

    /// The current version of the aggregate.
    pub fn version(&self) -> Version {
        self.version
    }

    /// The version of the snapshot from which the aggregate was loaded.
    pub fn snapshot_version(&self) -> Option<Version> {
        self.snapshot_version
    }

    /// Updates the snapshot version. Generally used to indicate that a snapshot was taken.
    pub fn set_snapshot_version(&mut self, new_snapshot_version: Version) {
        self.snapshot_version = Some(new_snapshot_version);
    }

    /// The actual aggregate.
    pub fn state(&self) -> &A {
        &self.state
    }

    /// Mutable access to the state.
    ///
    /// # Safety
    ///
    /// Direct mutation bypasses event sourcing. Use only for snapshots or
    /// internal operations.
    pub fn state_mut(&mut self) -> &mut A {
        &mut self.state
    }
}

// AsRef/Borrow implementations - no Aggregate bound needed!

impl<A> AsRef<A> for HydratedAggregate<A> {
    fn as_ref(&self) -> &A {
        &self.state
    }
}

impl<A> Borrow<A> for HydratedAggregate<A> {
    fn borrow(&self) -> &A {
        &self.state
    }
}

// ============================================================================
// Entity - Identified Aggregate Instance
// ============================================================================

/// An identified, specific instance of a hydrated aggregate.
///
/// REFACTORED: Removed trait bound from type definition. The bound is only
/// needed in specific impl blocks that use Aggregate behavior.
pub struct Entity<I, A> {
    id: I,
    aggregate: HydratedAggregate<A>,
}

// Manual implementations without unnecessary bounds

impl<I, A> Clone for Entity<I, A>
where
    I: Clone,
    A: Clone,
{
    fn clone(&self) -> Self {
        Entity {
            id: self.id.clone(),
            aggregate: self.aggregate.clone(),
        }
    }
}

impl<I, A> Copy for Entity<I, A>
where
    I: Copy,
    A: Copy,
{
}

impl<I, A> fmt::Debug for Entity<I, A>
where
    I: fmt::Debug,
    A: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entity")
            .field("id", &self.id)
            .field("aggregate", &self.aggregate)
            .finish()
    }
}

impl<I, A> Default for Entity<I, A>
where
    I: Default,
    A: Default,
{
    fn default() -> Self {
        Entity {
            id: I::default(),
            aggregate: HydratedAggregate::default(),
        }
    }
}

impl<I, A> PartialEq for Entity<I, A>
where
    I: PartialEq,
    A: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.aggregate == other.aggregate
    }
}

impl<I, A> Eq for Entity<I, A>
where
    I: Eq,
    A: Eq,
{
}

impl<I, A> std::hash::Hash for Entity<I, A>
where
    I: std::hash::Hash,
    A: std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.aggregate.hash(state);
    }
}

// Behavior implementations - no bounds needed for data access!

impl<I, A> Entity<I, A> {
    /// Creates a new entity from an identifier and an associated hydrated aggregate.
    pub fn new(id: I, aggregate: HydratedAggregate<A>) -> Self {
        Entity { id, aggregate }
    }

    /// The entity's identifier.
    pub fn id(&self) -> &I {
        &self.id
    }

    /// An immutable reference to the underlying aggregate.
    pub fn aggregate(&self) -> &HydratedAggregate<A> {
        &self.aggregate
    }

    /// A mutable reference to the underlying aggregate.
    pub fn aggregate_mut(&mut self) -> &mut HydratedAggregate<A> {
        &mut self.aggregate
    }
}

// Aggregate-specific behavior - bound only where needed

impl<I, A> Entity<I, A>
where
    A: Aggregate,
{
    /// Applies an event to the entity's aggregate.
    ///
    /// # Errors
    ///
    /// Returns `AggregateError` if the event cannot be applied.
    pub fn apply_event<E>(&mut self, event: E) -> Result<(), AggregateError>
    where
        E: AggregateEvent<A>,
    {
        self.aggregate.apply(event)
    }
}

// Conversion traits - minimal bounds

impl<I, A> From<Entity<I, A>> for HydratedAggregate<A> {
    fn from(entity: Entity<I, A>) -> Self {
        entity.aggregate
    }
}

impl<I, A> AsRef<HydratedAggregate<A>> for Entity<I, A> {
    fn as_ref(&self) -> &HydratedAggregate<A> {
        &self.aggregate
    }
}

impl<I, A> AsMut<HydratedAggregate<A>> for Entity<I, A> {
    fn as_mut(&mut self) -> &mut HydratedAggregate<A> {
        &mut self.aggregate
    }
}

impl<I, A> Borrow<HydratedAggregate<A>> for Entity<I, A> {
    fn borrow(&self) -> &HydratedAggregate<A> {
        &self.aggregate
    }
}

impl<I, A> Borrow<A> for Entity<I, A> {
    fn borrow(&self) -> &A {
        self.aggregate.borrow()
    }
}

impl<I, A> BorrowMut<HydratedAggregate<A>> for Entity<I, A> {
    fn borrow_mut(&mut self) -> &mut HydratedAggregate<A> {
        &mut self.aggregate
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug, PartialEq)]
    struct TestAggregate {
        counter: u32,
    }

    impl Aggregate for TestAggregate {
        fn aggregate_type() -> &'static str {
            "Test"
        }
    }

    struct Increment;

    impl Event for Increment {
        fn event_type(&self) -> &'static str {
            "Increment"
        }
    }

    impl AggregateEvent<TestAggregate> for Increment {
        fn apply_to(self, aggregate: &mut TestAggregate) {
            aggregate.counter += 1;
        }
    }

    #[test]
    fn test_event_application() {
        let mut hydrated = HydratedAggregate::<TestAggregate>::default();
        assert_eq!(hydrated.version(), Version::Initial);

        hydrated.apply(Increment).unwrap();
        assert_eq!(hydrated.version(), Version::Number(EventNumber::MIN_VALUE));
        assert_eq!(hydrated.state().counter, 1);
    }

    #[test]
    fn test_concurrency_check() {
        let mut hydrated = HydratedAggregate::<TestAggregate>::default();

        // Should succeed with correct version
        assert!(
            hydrated
                .apply_with_concurrency_check(Increment, Version::Initial)
                .is_ok()
        );

        // Should fail with wrong version
        assert!(matches!(
            hydrated.apply_with_concurrency_check(Increment, Version::Initial),
            Err(AggregateError::VersionMismatch { .. })
        ));
    }

    #[test]
    fn test_version_overflow() {
        let mut version = Version::Number(EventNumber(NonZeroU64::new(u64::MAX).unwrap()));
        assert!(matches!(
            version.incr(),
            Err(AggregateError::VersionOverflow)
        ));
    }
}
