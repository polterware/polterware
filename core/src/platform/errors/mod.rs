//! Port of `backend/platform/errors`.

use std::fmt;

/// A stable error code shared by the core and its clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Code(pub &'static str);

impl Code {
    pub const UNAUTHENTICATED: Code = Code("unauthenticated");
    pub const PERMISSION_DENIED: Code = Code("permission_denied");
    pub const INVALID_ARGUMENT: Code = Code("invalid_argument");
    pub const NOT_FOUND: Code = Code("not_found");
    pub const RESOURCE_EXHAUSTED: Code = Code("resource_exhausted");
    pub const DEADLINE_EXCEEDED: Code = Code("deadline_exceeded");
    pub const INTERNAL: Code = Code("internal");
    pub const FAILED_PRECONDITION: Code = Code("failed_precondition");

    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl From<&'static str> for Code {
    fn from(value: &'static str) -> Self {
        Code(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detail {
    pub code: Code,
    pub message: String,
    pub retryable: bool,
    pub correlation_id: String,
}

pub fn map_retryable(c: Code) -> bool {
    matches!(c, Code::RESOURCE_EXHAUSTED | Code::INTERNAL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable() {
        assert!(map_retryable(Code::RESOURCE_EXHAUSTED));
        assert!(!map_retryable(Code::PERMISSION_DENIED));
        assert_eq!(Code::FAILED_PRECONDITION.as_str(), "failed_precondition");
        assert_eq!(
            Detail {
                code: Code::NOT_FOUND,
                message: "missing".to_string(),
                retryable: false,
                correlation_id: "abc".to_string(),
            },
            Detail {
                code: Code::from("not_found"),
                message: "missing".to_string(),
                retryable: false,
                correlation_id: "abc".to_string(),
            }
        );
    }
}
