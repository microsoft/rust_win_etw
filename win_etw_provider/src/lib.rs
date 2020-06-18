//! Enables Rust apps to report events using Event Tracing for Windows.
//!
//! See [About Event Tracing](https://docs.microsoft.com/en-us/windows/win32/etw/about-event-tracing).

#![deny(missing_docs)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![cfg_attr(not(windows), allow(unused))]

mod guid;
mod provider;
mod types;

pub use guid::GUID;
pub use provider::*;
pub use types::*;

pub use win_etw_metadata as metadata;
mod data_descriptor;

pub use data_descriptor::EventDataDescriptor;

/// Errors returned by `win_etw_provider` functions.
///
/// When compiling for non-Windows platforms, this Error type becomes an uninhabited type.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    /// A Windows (Win32) error code.
    #[cfg(target_os = "windows")]
    WindowsError(u32),
}

/// Allows an application to override the parameters for an event. The first parameter of each
/// generated event method is `options: Option<&EventOptions>`.
#[derive(Default)]
pub struct EventOptions {
    /// Overrides the level of the event, if present. Each event method has a default, which can be
    /// specified using (for example) `#[event(level = "warn")]`. If the event declaration does not
    /// specify a level, then the level will be `Level::INFO`.
    pub level: Option<win_etw_metadata::Level>,

    /// Specifies the activity ID of this event.
    pub activity_id: Option<guid::GUID>,

    /// Specifies a related activity ID for this event. This enables an application to indicate
    /// that two sets of events are related, by associating the activity IDs of the two sets.
    /// This is sometimes known as _event correlation_.
    pub related_activity_id: Option<guid::GUID>,
}

pub use win_etw_metadata::Level;
