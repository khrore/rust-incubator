use std::borrow::Cow;
use std::env;
use std::net::{IpAddr, SocketAddr};

fn main() {
    println!("Refactored!");

    // Demonstrate flexible input types for error creation
    let _err = Error::new(ErrorCode::from_static("NO_USER"))
        .with_status(HttpStatus(404))
        .with_message(ErrorMessage::from_static("User not found"));

    // Also accepts String
    let _err2 = Error::new(ErrorCode::from_string("PERMISSION_DENIED".to_string()))
        .with_status(HttpStatus(403))
        .with_message(ErrorMessage::from_static("Access forbidden"));

    // Demonstrate flexible input types for server
    let _server1 = Server::bind("127.0.0.1".parse::<IpAddr>().unwrap(), 8080);
    let _server2 = Server::bind_addr("[::1]:9911".parse::<SocketAddr>().unwrap());
}

/// Wrapper for error codes to ensure type safety
#[derive(Debug, Clone)]
pub struct ErrorCode<'a>(Cow<'a, str>);

impl<'a> ErrorCode<'a> {
    pub fn from_static(s: &'static str) -> Self {
        ErrorCode(Cow::Borrowed(s))
    }

    pub fn from_string(s: String) -> Self {
        ErrorCode(Cow::Owned(s))
    }
}

impl<'a> From<&'static str> for ErrorCode<'a> {
    fn from(s: &'static str) -> Self {
        ErrorCode::from_static(s)
    }
}

impl<'a> From<String> for ErrorCode<'a> {
    fn from(s: String) -> Self {
        ErrorCode::from_string(s)
    }
}

impl<'a> AsRef<str> for ErrorCode<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// Wrapper for HTTP status codes
#[derive(Debug, Clone, Copy)]
pub struct HttpStatus(pub u16);

impl From<u16> for HttpStatus {
    fn from(status: u16) -> Self {
        HttpStatus(status)
    }
}

/// Wrapper for error messages to ensure they're safe for client exposure
#[derive(Debug, Clone)]
pub struct ErrorMessage<'a>(Cow<'a, str>);

impl<'a> ErrorMessage<'a> {
    pub fn from_static(s: &'static str) -> Self {
        ErrorMessage(Cow::Borrowed(s))
    }

    pub fn from_string(s: String) -> Self {
        ErrorMessage(Cow::Owned(s))
    }
}

impl<'a> From<&'static str> for ErrorMessage<'a> {
    fn from(s: &'static str) -> Self {
        ErrorMessage::from_static(s)
    }
}

impl<'a> From<String> for ErrorMessage<'a> {
    fn from(s: String) -> Self {
        ErrorMessage::from_string(s)
    }
}

impl<'a> AsRef<str> for ErrorMessage<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// Secure Error type for high-load backend systems
#[derive(Debug, Clone)]
pub struct Error<'a> {
    /// Internal error code (internal use only)
    code: ErrorCode<'a>,
    /// HTTP status code
    status: HttpStatus,
    /// Public-facing message (safe for client exposure)
    message: ErrorMessage<'a>,
    /// Optional internal debug information (not exposed to clients)
    debug_info: Option<Cow<'a, str>>,
}

impl<'a> Error<'a> {
    /// Creates a new error with an error code
    /// Accepts any type that can be converted to an ErrorCode
    pub fn new<S: Into<ErrorCode<'a>>>(code: S) -> Self {
        Self {
            code: code.into(),
            status: HttpStatus(500),
            message: ErrorMessage::from_static("An unknown error occurred."),
            debug_info: None,
        }
    }

    /// Sets the HTTP status code
    pub fn with_status(mut self, status: HttpStatus) -> Self {
        self.status = status;
        self
    }

    /// Sets the public-facing error message
    /// Accepts any type that can be converted to an ErrorMessage
    pub fn with_message<S: Into<ErrorMessage<'a>>>(mut self, message: S) -> Self {
        self.message = message.into();
        self
    }

    /// Sets internal debug information that isn't exposed to clients
    pub fn with_debug_info<S: Into<Cow<'a, str>>>(mut self, info: S) -> Self {
        self.debug_info = Some(info.into());
        self
    }

    /// Gets the error code (for internal logging only)
    pub fn code(&self) -> &str {
        self.code.as_ref()
    }

    /// Gets the HTTP status code
    pub fn status(&self) -> u16 {
        self.status.0
    }

    /// Gets the public-facing message
    pub fn message(&self) -> &str {
        self.message.as_ref()
    }

    /// Gets internal debug information (should not be sent to clients)
    pub fn debug_info(&self) -> Option<&str> {
        self.debug_info.as_ref().map(|s| s.as_ref())
    }
}

/// Wrapper for worker count to enforce positive values
#[derive(Debug, Clone, Copy)]
pub struct WorkerCount(usize);

impl WorkerCount {
    pub fn new(count: usize) -> Option<Self> {
        if count > 0 {
            Some(WorkerCount(count))
        } else {
            None
        }
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

impl From<usize> for WorkerCount {
    fn from(count: usize) -> Self {
        WorkerCount(count)
    }
}

/// Wrapper for max connections to enforce positive values
#[derive(Debug, Clone, Copy)]
pub struct MaxConnections(usize);

impl MaxConnections {
    pub fn new(count: usize) -> Option<Self> {
        if count > 0 {
            Some(MaxConnections(count))
        } else {
            None
        }
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

impl From<usize> for MaxConnections {
    fn from(count: usize) -> Self {
        MaxConnections(count)
    }
}

/// Thread-safe Server configuration for high-load systems
#[derive(Debug, Clone)]
pub struct Server {
    address: Option<SocketAddr>,
    workers: WorkerCount,  // Number of worker threads for high concurrency
    max_connections: MaxConnections,  // Connection pool limits
}

impl Default for Server {
    #[inline]
    fn default() -> Self {
        Self {
            address: None,
            workers: WorkerCount::from(num_cpus_estimate()),
            max_connections: MaxConnections::from(10_000),   // Reasonable default for high-load
        }
    }
}

impl Server {
    /// Creates a server bound to a specific IP and port
    /// Accepts generic IP address type for flexibility
    pub fn bind<I: Into<IpAddr>>(ip: I, port: u16) -> Self {
        let mut server = Self::default();
        server.address = Some(SocketAddr::new(ip.into(), port));
        server
    }

    /// Creates a server bound to a specific socket address
    /// Accepts any type that can be converted to a SocketAddr
    pub fn bind_addr<A: Into<SocketAddr>>(addr: A) -> Self {
        let mut server = Self::default();
        server.address = Some(addr.into());
        server
    }

    /// Configures the number of worker threads
    pub fn with_workers(mut self, count: WorkerCount) -> Self {
        self.workers = count;
        self
    }

    /// Configures the maximum number of connections
    pub fn with_max_connections(mut self, max: MaxConnections) -> Self {
        self.max_connections = max;
        self
    }

    /// Gets the server's listening address
    pub fn address(&self) -> Option<SocketAddr> {
        self.address
    }

    /// Gets the number of configured worker threads
    pub fn workers(&self) -> usize {
        self.workers.get()
    }

    /// Gets the maximum connection limit
    pub fn max_connections(&self) -> usize {
        self.max_connections.get()
    }

    /// Validates the server configuration before startup
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.address.is_none() {
            return Err(ValidationError::MissingAddress);
        }

        if self.workers.get() == 0 {
            return Err(ValidationError::InvalidWorkers);
        }

        if self.max_connections.get() == 0 {
            return Err(ValidationError::InvalidMaxConnections);
        }

        Ok(())
    }
}

/// Validation errors for server configuration
#[derive(Debug, PartialEq)]
pub enum ValidationError {
    MissingAddress,
    InvalidWorkers,
    InvalidMaxConnections,
}

/// Helper function to estimate available CPU cores for high-performance scenarios
/// Uses environment variable override if available, otherwise defaults to 4
fn num_cpus_estimate() -> usize {
    env::var("RAYON_NUM_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            // Conservative default that works well in most environments
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
        })
}

#[cfg(test)]
mod server_spec {
    use super::*;

    mod bind {
        use std::net::{Ipv4Addr, Ipv6Addr};

        use super::*;

        #[test]
        fn sets_provided_address_to_server() {
            // Test IPv4 binding with string parsing
            let server1 = Server::bind("127.0.0.1".parse::<IpAddr>().unwrap(), 8080);
            assert_eq!(server1.address().unwrap().to_string(), "127.0.0.1:8080");

            // Test IPv6 binding
            let server2 = Server::bind_addr("[::1]:9911".parse::<SocketAddr>().unwrap());
            assert_eq!(server2.address().unwrap().to_string(), "[::1]:9911");

            // Test with typed IP address
            let server3 = Server::bind(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 3000);
            assert_eq!(server3.address().unwrap().to_string(), "192.168.1.1:3000");

            let server4 = Server::bind(IpAddr::V6(Ipv6Addr::LOCALHOST), 4000);
            assert_eq!(server4.address().unwrap().to_string(), "[::1]:4000");
        }

        #[test]
        fn configures_workers_and_max_connections() {
            let server = Server::bind("127.0.0.1".parse::<IpAddr>().unwrap(), 8080)
                .with_workers(WorkerCount::from(8))
                .with_max_connections(MaxConnections::from(20_000));

            assert_eq!(server.workers(), 8);
            assert_eq!(server.max_connections(), 20_000);
        }

        #[test]
        fn validates_configuration() {
            let valid_server = Server::bind("127.0.0.1".parse::<IpAddr>().unwrap(), 8080);
            assert!(valid_server.validate().is_ok());

            let invalid_server = Server::default();
            assert_eq!(invalid_server.validate().unwrap_err(), ValidationError::MissingAddress);
        }
    }

    mod error {
        use super::*;

        #[test]
        fn creates_error_with_generic_input_types() {
            // Test with &str
            let err1 = Error::new(ErrorCode::from_static("NOT_FOUND"))
                .with_message(ErrorMessage::from_static("Item not found"));
            assert_eq!(err1.code(), "NOT_FOUND");
            assert_eq!(err1.message(), "Item not found");

            // Test with String
            let err2 = Error::new(ErrorCode::from_string("TIMEOUT".to_string()))
                .with_message(ErrorMessage::from_string("Request timed out".to_string()));
            assert_eq!(err2.code(), "TIMEOUT");
            assert_eq!(err2.message(), "Request timed out");

            // Test with From conversion
            let err3 = Error::new("UNAUTHORIZED")
                .with_message("Not authorized");
            assert_eq!(err3.code(), "UNAUTHORIZED");
            assert_eq!(err3.message(), "Not authorized");
        }

        #[test]
        fn handles_debug_info_properly() {
            let err = Error::new(ErrorCode::from_static("INTERNAL_ERROR"))
                .with_message(ErrorMessage::from_static("Something went wrong"))
                .with_debug_info("Database connection failed");

            assert_eq!(err.code(), "INTERNAL_ERROR");
            assert_eq!(err.message(), "Something went wrong");
            assert_eq!(err.debug_info(), Some("Database connection failed"));
        }

        #[test]
        fn handles_status_codes_properly() {
            let err = Error::new(ErrorCode::from_static("BAD_REQUEST"))
                .with_status(HttpStatus(400))
                .with_message(ErrorMessage::from_static("Bad request"));

            assert_eq!(err.status(), 400);
        }
    }
}
