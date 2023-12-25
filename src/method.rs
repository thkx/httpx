
use std::fmt::{Display, Result, Formatter};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    // PATCH,
    // OPTIONS,
    // HEAD,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Method::GET => write!(f, "{}", "GET"),
            Method::POST => write!(f, "{}", "POST"),
            Method::PUT => write!(f, "{}", "PUT"),
            Method::DELETE => write!(f, "{}", "DELETE"),
            // Method::PATCH => write!(f, "{}", "PATCH"),
            // Method::OPTIONS => write!(f, "{}", "OPTIONS"),
            // Method::HEAD => write!(f, "{}", "HEAD"),
        }
    }
}

impl From<&str> for Method {
    fn from(method: &str) -> Method {
        match method {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            // "PATCH" => Method::PATCH,
            // "OPTIONS" => Method::OPTIONS,
            // "HEAD" => Method::HEAD,
            _ => Method::GET,
        }
    }
}
