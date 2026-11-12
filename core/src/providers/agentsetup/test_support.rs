//! Shared helpers for the agent setup tests. HOME and PATH are process-wide and
//! tests run in parallel, so they are overridden per thread instead.

use std::ffi::OsStr;
use std::sync::{Mutex, MutexGuard};

use crate::providers::agentsetup::command::overrides;

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Keeps the tests that install agents in their own order.
pub(crate) fn lock_env() -> MutexGuard<'static, ()> {
    ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Overrides a variable for the test and clears the override on drop.
pub(crate) struct EnvGuard {
    key: &'static str,
}

impl EnvGuard {
    pub(crate) fn set(key: &'static str, value: impl AsRef<OsStr>) -> EnvGuard {
        overrides::set(key, value.as_ref().to_string_lossy().into_owned());
        EnvGuard { key }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        overrides::remove(self.key);
    }
}
