//! A realistic HTTP client module demonstrating common real-world patterns:
//! error types, builder structs, async methods, trait implementations, and
//! associated constants.

use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

/// Maximum number of redirects the client will follow before giving up.
pub const MAX_REDIRECTS: u32 = 20;

/// Default request timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

static USER_AGENT: &str = "myapp/1.0";

/// Errors that can be returned by the HTTP client.
#[derive(Debug)]
pub enum ClientError {
    /// A network-level I/O failure.
    Io(std::io::Error),
    /// The server returned a non-2xx status code.
    Http { status: u16, body: String },
    /// The response body could not be decoded.
    Decode(String),
    /// The request timed out.
    Timeout,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Io(e) => write!(f, "io error: {e}"),
            ClientError::Http { status, body } => write!(f, "http {status}: {body}"),
            ClientError::Decode(msg) => write!(f, "decode error: {msg}"),
            ClientError::Timeout => write!(f, "request timed out"),
        }
    }
}

impl std::error::Error for ClientError {}

/// An HTTP response from the server.
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    /// Parse the body as UTF-8 text.
    pub fn text(&self) -> Result<&str, ClientError> {
        std::str::from_utf8(&self.body).map_err(|e| ClientError::Decode(e.to_string()))
    }

    /// Return true if the status code indicates success.
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }
}

/// Builder for constructing an [`HttpClient`] with custom settings.
#[derive(Default)]
pub struct ClientBuilder {
    timeout: Option<Duration>,
    max_redirects: u32,
    headers: HashMap<String, String>,
}

impl ClientBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        ClientBuilder {
            timeout: None,
            max_redirects: MAX_REDIRECTS,
            headers: HashMap::new(),
        }
    }

    /// Override the per-request timeout.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Set a default header sent with every request.
    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Consume the builder and produce an [`HttpClient`].
    pub fn build(self) -> HttpClient {
        HttpClient {
            timeout: self.timeout.unwrap_or(DEFAULT_TIMEOUT),
            max_redirects: self.max_redirects,
            default_headers: self.headers,
        }
    }
}

/// A lightweight asynchronous HTTP client.
pub struct HttpClient {
    timeout: Duration,
    max_redirects: u32,
    default_headers: HashMap<String, String>,
}

impl HttpClient {
    /// Send a GET request to `url`.
    pub async fn get(&self, url: &str) -> Result<Response, ClientError> {
        self.send("GET", url, None).await
    }

    /// Send a POST request with an optional body.
    pub async fn post(&self, url: &str, body: Option<Vec<u8>>) -> Result<Response, ClientError> {
        self.send("POST", url, body).await
    }

    /// Internal dispatch: build and execute the request.
    async fn send(
        &self,
        method: &str,
        url: &str,
        body: Option<Vec<u8>>,
    ) -> Result<Response, ClientError> {
        let _ = (method, url, body, self.timeout, self.max_redirects);
        unimplemented!("networking backend not wired up in this fixture")
    }
}

/// A retry policy that wraps an inner client.
pub trait RetryPolicy {
    /// Maximum number of retry attempts.
    fn max_attempts(&self) -> u32;

    /// Delay before the next attempt (may be zero).
    fn delay(&self, attempt: u32) -> Duration;
}

/// Exponential back-off retry policy.
pub struct ExponentialBackoff {
    pub base_delay: Duration,
    pub max_attempts: u32,
}

impl RetryPolicy for ExponentialBackoff {
    fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    fn delay(&self, attempt: u32) -> Duration {
        self.base_delay * 2u32.saturating_pow(attempt)
    }
}

/// Parse a raw `Content-Type` header value into its MIME type portion.
pub fn parse_mime_type(content_type: &str) -> &str {
    content_type.split(';').next().unwrap_or(content_type).trim()
}

/// Percent-encode a query string value.
pub fn url_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push(char::from_digit((byte >> 4) as u32, 16).unwrap_or('0'));
            out.push(char::from_digit((byte & 0xf) as u32, 16).unwrap_or('0'));
        }
    }
    out
}
