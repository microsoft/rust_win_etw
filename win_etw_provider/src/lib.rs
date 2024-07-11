//! Enables Rust apps to report events using Event Tracing for Windows.
//!
//! See [About Event Tracing](https://docs.microsoft.com/en-us/windows/win32/etw/about-event-tracing).

#![deny(missing_docs)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![cfg_attr(not(windows), allow(unused))]

extern crate alloc;

mod guid;
mod provider;
#[cfg(all(not(feature = "windows_apps"), feature = "windows_drivers"))]
mod driver_provider;
#[cfg(all(not(feature = "windows_apps"), feature = "windows_drivers"))]
pub use driver_provider::EtwDriverProvider;

pub mod types;

#[doc(inline)]
pub use guid::GUID;

#[doc(inline)]
pub use provider::*;

#[doc(hidden)]
pub use types::*;

#[doc(inline)]
pub use types::{SocketAddrV4, SocketAddrV6, FILETIME};

#[doc(hidden)]
pub use win_etw_metadata as metadata;

mod data_descriptor;

#[doc(inline)]
pub use data_descriptor::EventDataDescriptor;

/// Errors returned by `win_etw_provider` functions.
///
/// When compiling for non-Windows platforms, this Error type becomes an uninhabited type.
#[derive(Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum Error {
    /// A Windows (Win32) error code.
    #[cfg(target_os = "windows")]
    WindowsError(u32),

    /// The operation is not supported on this platform.
    ///
    /// Most operations defined in this crate do nothing on non-Windows platforms. Those operations
    /// that return information, such as the `new_activity_id()` function, use this error value.
    NotSupported,
}

/// Allows an application to override the parameters for an event. The first parameter of each
/// generated event method is `options: Option<&EventOptions>`.
#[derive(Default)]
pub struct EventOptions {
    /// Overrides the level of the event, if present. Each event method has a default, which can be
    /// specified using (for example) `#[event(level = "warn")]`. If the event declaration does not
    /// specify a level, then the level will be `Level::VERBOSE`.
    pub level: Option<win_etw_metadata::Level>,

    /// Specifies the activity ID of this event.
    pub activity_id: Option<guid::GUID>,

    /// Specifies a related activity ID for this event. This enables an application to indicate
    /// that two sets of events are related, by associating the activity IDs of the two sets.
    /// This is sometimes known as _event correlation_.
    pub related_activity_id: Option<guid::GUID>,
}

pub use win_etw_metadata::Level;
