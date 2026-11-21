//! Shared application identity used by the daemon and desktop client.

#[cfg(debug_assertions)]
pub const APP_NAME: &str = "Rapid Pi Debug";
#[cfg(not(debug_assertions))]
pub const APP_NAME: &str = "Rapid Pi";

#[cfg(debug_assertions)]
pub const APP_ID: &str = "sh.rapidpi.dev";
#[cfg(not(debug_assertions))]
pub const APP_ID: &str = "sh.rapidpi";

#[cfg(debug_assertions)]
pub const DATA_DIRECTORY_NAME: &str = "Rapid Pi Debug";
#[cfg(not(debug_assertions))]
pub const DATA_DIRECTORY_NAME: &str = "Rapid Pi";
