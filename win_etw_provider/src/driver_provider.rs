//! Provides an ETW provider that uses Windows kernel-mode APIs and types to implement ETW tracing.  
//! The `windows-driver` feature must be activated to compile.
//! This provider is designed to match the behavior of the standard user-mode ETW provider and is
//! largely copied from that module, with changes to use wdk_sys equivalent functions and values.

use crate::guid::GUID;
use crate::EventDescriptor;
use crate::Level;
use crate::Provider;
use crate::{Error, EventDataDescriptor};
use alloc::boxed::Box;
use core::convert::TryFrom;
use core::pin::Pin;
use core::ptr::null;
use core::sync::atomic::{AtomicU8, Ordering::SeqCst};
use wdk_sys::NT_SUCCESS;

use win_support::*;

/// Generates a new activity ID.
///
/// This function is only implemented on Windows.
pub fn new_activity_id() -> Result<GUID, Error> {
    win_support::new_activity_id()
}

/// Implements `Provider` by registering with ETW.
pub struct EtwDriverProvider {
    handle: wdk_sys::REGHANDLE,

    stable: Pin<Box<StableProviderData>>,
}

impl Provider for EtwDriverProvider {
    #[inline(always)]
    fn write(
        &self,
        options: Option<&crate::EventOptions>,
        descriptor: &EventDescriptor,
        data: &[EventDataDescriptor<'_>],
    ) {
        unsafe {
            let mut activity_id_ptr = null();
            let mut related_activity_id_ptr = null();

            let mut event_descriptor = wdk_sys::EVENT_DESCRIPTOR {
                Id: descriptor.id,
                Version: descriptor.version,
                Channel: descriptor.channel,
                Level: descriptor.level.0,
                Opcode: descriptor.opcode,
                Task: descriptor.task,
                Keyword: descriptor.keyword,
            };

            if let Some(options) = options {
                if let Some(id) = options.activity_id.as_ref() {
                    activity_id_ptr = id as *const GUID as *const wdk_sys::GUID;
                }
                if let Some(id) = options.related_activity_id.as_ref() {
                    related_activity_id_ptr = id as *const GUID as *const wdk_sys::GUID;
                }
                if let Some(level) = options.level {
                    event_descriptor.Level = level.0;
                }
            }

            let error = wdk_sys::ntddk::EtwWriteEx(
                self.handle,
                &event_descriptor as *const wdk_sys::_EVENT_DESCRIPTOR,
                0,                       // filter
                0,                       // flags
                activity_id_ptr,         // activity id
                related_activity_id_ptr, // related activity id
                data.len() as u32,
                data.as_ptr() as *mut wdk_sys::_EVENT_DATA_DESCRIPTOR,
            );
            if !NT_SUCCESS(error) {
                write_failed(error as u32)
            }
        }
    }

    // write_ex
    // write_transfer

    fn is_enabled(&self, level: u8, keyword: u64) -> bool {
        unsafe { wdk_sys::ntddk::EtwProviderEnabled(self.handle, level, keyword) != 0 }
    }

    fn is_event_enabled(&self, event_descriptor: &EventDescriptor) -> bool {
        if false {
            unsafe {
                wdk_sys::ntddk::EtwEventEnabled(
                    self.handle,
                    event_descriptor as *const _ as *const wdk_sys::EVENT_DESCRIPTOR,
                ) != 0
            }
        } else {
            let max_level = self.stable.as_ref().max_level.load(SeqCst);
            event_descriptor.level.0 <= max_level
        }
    }
}

#[inline(never)]
fn write_failed(_error: u32) {
    #[cfg(feature = "dev")]
    {
        eprintln!("EventWrite failed: {}", _error);
    }
}

mod win_support {

    use super::*;
    pub use winapi::shared::evntrace;

    /// This data is stored in a Box, so that it has a stable address.
    /// It is used to coordinate with ETW; ETW runs callbacks that need a stable pointer.
    /// See `EventRegister` and the "enable callback".
    pub(crate) struct StableProviderData {
        pub(crate) max_level: AtomicU8,
    }

    /// See [ETWENABLECALLBACK](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nc-wdm-etwenablecallback).
    pub(crate) unsafe extern "C" fn enable_callback(
        _source_id: *const wdk_sys::GUID,
        is_enabled_code: u32,
        level: u8,
        _match_any_keyword: u64,
        _match_all_keyword: u64,
        _filter_data: *mut wdk_sys::EVENT_FILTER_DESCRIPTOR,
        context: wdk_sys::PVOID,
    ) {
        // This should never happen.
        if context.is_null() {
            return;
        }
        let stable_data: &StableProviderData = &*(context as *const _ as *const StableProviderData);

        let _source_id: GUID = if _source_id.is_null() {
            GUID::default()
        } else {
            (*(_source_id as *const GUID)).clone()
        };
        #[cfg(feature = "dev")]
        {
            eprintln!(
                "enable_callback: source_id {} is_enabled {}, level {}, any {:#x} all {:#x} filter? {:?}",
                _source_id, is_enabled_code, level, _match_any_keyword, _match_all_keyword,
                !_filter_data.is_null()
            );
        }

        match is_enabled_code {
            evntrace::EVENT_CONTROL_CODE_ENABLE_PROVIDER => {
                #[cfg(feature = "dev")]
                {
                    eprintln!("ETW is ENABLING this provider.  setting level: {}", level);
                }
                stable_data.max_level.store(level, SeqCst);
            }
            evntrace::EVENT_CONTROL_CODE_DISABLE_PROVIDER => {
                #[cfg(feature = "dev")]
                {
                    eprintln!("ETW is DISABLING this provider.  setting level: {}", level);
                }
                stable_data.max_level.store(level, SeqCst);
            }
            evntrace::EVENT_CONTROL_CODE_CAPTURE_STATE => {
                // ETW is requesting that the provider log its state information. The meaning of this
                // is provider-dependent. Currently, this functionality is not exposed to Rust apps.
                #[cfg(feature = "dev")]
                {
                    eprintln!("EVENT_CONTROL_CODE_CAPTURE_STATE");
                }
            }
            _ => {
                // The control code is unrecognized.
                #[cfg(feature = "dev")]
                {
                    eprintln!(
                        "enable_callback: control code {} is not recognized",
                        is_enabled_code
                    );
                }
            }
        }
    }

    pub fn new_activity_id() -> Result<GUID, Error> {
        unsafe {
            let mut guid: wdk_sys::GUID = core::mem::zeroed();
            let error = wdk_sys::ntddk::EtwActivityIdControl(
                wdk_sys::EVENT_ACTIVITY_CTRL_CREATE_ID,
                &mut guid,
            );
            if error == 0 {
                Ok(guid.into())
            } else {
                Err(Error::WindowsError(error as u32))
            }
        }
    }
}

impl EtwDriverProvider {
    /// Registers an event provider with ETW.
    ///
    /// The implementation uses `[EtwRegister](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-etwregister)`.
    pub fn new(provider_id: &GUID) -> Result<EtwDriverProvider, Error> {
        unsafe {
            let mut stable = Box::pin(StableProviderData {
                max_level: AtomicU8::new(0),
            });
            let mut handle: wdk_sys::REGHANDLE = 0;
            let stable_ptr: &mut StableProviderData = &mut stable;
            let error = wdk_sys::ntddk::EtwRegister(
                provider_id as *const _ as *const wdk_sys::GUID,
                Some(enable_callback),
                stable_ptr as *mut StableProviderData as wdk_sys::PVOID,
                &mut handle,
            );
            if error != 0 {
                Err(Error::WindowsError(error as u32))
            } else {
                Ok(EtwDriverProvider { handle, stable })
            }
        }
    }

    /// See TraceLoggingRegisterEx in traceloggingprovider.h.
    /// This registers provider metadata.
    pub fn register_provider_metadata(&mut self, provider_metadata: &[u8]) -> Result<(), Error> {
        unsafe {
            let error = wdk_sys::ntddk::EtwSetInformation(
                self.handle,
                2,
                provider_metadata.as_ptr() as wdk_sys::PVOID,
                u32::try_from(provider_metadata.len()).unwrap(),
            );
            if error != 0 {
                Err(Error::WindowsError(error as u32))
            } else {
                #[cfg(feature = "dev")]
                {
                    eprintln!("register_provider_metadata: succeeded");
                }
                Ok(())
            }
        }
    }

    /// Registers provider traits for a provider.
    ///
    /// ETW providers should not call this function directly. It is automatically
    /// called by the provider code that is generated by `win_etw_macros`.
    ///
    /// See [Provider Traits](https://docs.microsoft.com/en-us/windows/win32/etw/provider-traits).
    pub fn set_provider_traits(&mut self, provider_traits: &[u8]) -> Result<(), Error> {
        unsafe {
            let error = wdk_sys::ntddk::EtwSetInformation(
                self.handle,
                wdk_sys::_EVENT_INFO_CLASS::EventProviderSetTraits,
                provider_traits.as_ptr() as *mut u8 as wdk_sys::PVOID,
                u32::try_from(provider_traits.len()).unwrap(),
            );
            if error != 0 {
                #[cfg(feature = "dev")]
                {
                    eprintln!("EventSetInformation failed for provider traits");
                }
                return Err(Error::WindowsError(error as u32));
            }
        }
        Ok(())
    }
}

impl Drop for EtwDriverProvider {
    fn drop(&mut self) {
        unsafe {
            // Nothing we can do if this fails.
            let _ = wdk_sys::ntddk::EtwUnregister(self.handle);
        }
    }
}

unsafe impl Send for EtwDriverProvider {}
unsafe impl Sync for EtwDriverProvider {}

/// Allows an application to enter a nested activity scope. This creates a new activity ID,
/// sets this activity ID as the current activity ID of the current thread, and then runs the
/// provided function. After the function finishes, it restores the activity ID of the calling
/// thread (even if a panic occurs).
///
/// See `[EtwActivityIdControl](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-etwactivityidcontrol)`.
#[inline(always)]
pub fn with_activity<F: FnOnce() -> R, R>(f: F) -> R {
    let mut previous_activity_id: GUID = Default::default();

    let mut restore = RestoreActivityHolder {
        previous_activity_id: None,
    };

    unsafe {
        let result = wdk_sys::ntddk::EtwActivityIdControl(
            wdk_sys::EVENT_ACTIVITY_CTRL_CREATE_SET_ID,
            &mut previous_activity_id as *mut _ as *mut wdk_sys::GUID,
        );
        if NT_SUCCESS(result) {
            restore.previous_activity_id = Some(previous_activity_id);
        } else {
            // Failed to create/replace the activity ID. There is not much we can do about this.
        }
    }

    let result = f();
    // RestoreActivityHolder::drop() will run, even if f() panics, and will restore the
    // activity ID of the current thread.
    drop(restore);
    result
}

struct RestoreActivityHolder {
    previous_activity_id: Option<GUID>,
}

impl Drop for RestoreActivityHolder {
    fn drop(&mut self) {
        unsafe {
            if let Some(previous_activity_id) = self.previous_activity_id.as_ref() {
                let _ = wdk_sys::ntddk::EtwActivityIdControl(
                    wdk_sys::EVENT_ACTIVITY_CTRL_SET_ID,
                    previous_activity_id as *const GUID as *mut wdk_sys::GUID,
                );
            }
        }
    }
}
