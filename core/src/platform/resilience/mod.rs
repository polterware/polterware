//! Port of `backend/platform/resilience`.

use std::fmt;

/// The retry class of a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Class(pub &'static str);

impl Class {
    pub const TRANSIENT_NETWORK: Class = Class("transient_network");
    pub const RESOURCE_PRESSURE: Class = Class("resource_pressure");
    pub const DEADLINE: Class = Class("deadline");
    pub const PERMANENT_FAILURE: Class = Class("permanent_failure");
    pub const VALIDATION: Class = Class("validation");
    pub const NOT_FOUND: Class = Class("not_found");
    pub const INTERNAL: Class = Class("internal");
    pub const UNAUTHENTICATED: Class = Class("unauthenticated");

    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl From<&'static str> for Class {
    fn from(value: &'static str) -> Self {
        Class(value)
    }
}

pub fn retryable(c: Class) -> bool {
    matches!(c, Class::TRANSIENT_NETWORK | Class::RESOURCE_PRESSURE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_class() {
        assert!(retryable(Class::TRANSIENT_NETWORK));
        assert!(!retryable(Class::PERMANENT_FAILURE));
        assert_eq!(Class::RESOURCE_PRESSURE.as_str(), "resource_pressure");
        assert_eq!(Class::DEADLINE.to_string(), "deadline");
    }
}
