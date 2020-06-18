//! Definitions for metadata used by
//! [Event Tracing for Windows](https://docs.microsoft.com/en-us/windows/win32/etw/event-tracing-portal).
//!
//! These definitions are used by the `win_etw_macros` crate to describe the schema of events.
//! Most applications will not need to use these definitions directly.

#![no_std]
#![deny(missing_docs)]

use bitflags::bitflags;

/// This structure describes the start of the ETW metadata section. A single static instance of
/// this structure is placed in PE/COFF modules, and it identifies the start of the ETW metadata
/// section. In this implementation, that single instance is `ETW_TRACE_LOGGING_METADATA`.
#[repr(C)]
pub struct TraceLoggingMetadata {
    signature: u32, // = _tlg_MetadataSignature = "ETW0"
    size: u16,      // = sizeof(_TraceLoggingMetadata_t)
    version: u8,    // = _tlg_MetadataVersion
    flags: u8,      // = _tlg_MetadataFlags
    magic: u64,     // = _tlg_MetadataMagic
}

/// The value stored in `TraceLoggingMetadata::signature`.
/// In little-endian ASCII, this is "ETW0".
pub const METADATA_SIGNATURE: u32 = 0x30_57_54_45;

/// The value stored in `TraceLoggingMetadata::magic`.
pub const METADATA_MAGIC: u64 = 0xBB8A_052B_8804_0E86;

/// The version of the metadata emitted. Currently, there is only one version.
pub const METADATA_VERSION: u8 = 0;

/// The bit flag which indicates the size of pointers on the target architecture.
/// The value of this constant depends on the target architecture.
#[cfg(target_pointer_width = "64")]
pub const METADATA_FLAGS_POINTER_WIDTH: u8 = 1;

/// The bit flag which indicates the size of pointers on the target architecture.
/// The value of this constant depends on the target architecture.
#[cfg(not(target_pointer_width = "64"))]
pub const METADATA_FLAGS_POINTER_WIDTH: u8 = 0;

#[cfg(feature = "metadata_headers")]
#[link_section = ".rdata$etw0"]
#[used]
#[no_mangle]
static ETW_TRACE_LOGGING_METADATA: TraceLoggingMetadata = TraceLoggingMetadata {
    signature: METADATA_SIGNATURE,
    size: core::mem::size_of::<TraceLoggingMetadata>() as u16,
    version: METADATA_VERSION,
    flags: METADATA_FLAGS_POINTER_WIDTH,
    magic: METADATA_MAGIC,
};

/// Predefined event tracing levels
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Level(pub u8);

impl Level {
    /// Tracing is not on
    pub const NONE: Level = Level(0);
    /// Abnormal exit or termination
    pub const CRITICAL: Level = Level(1);
    /// Deprecated name for Abnormal exit or termination
    pub const FATAL: Level = Level(1);
    /// Severe errors that need logging
    pub const ERROR: Level = Level(2);
    /// Warnings such as allocation failure
    pub const WARN: Level = Level(3);
    /// Includes non-error cases(e.g.,Entry-Exit)
    pub const INFO: Level = Level(4);
    /// Detailed traces from intermediate steps
    pub const VERBOSE: Level = Level(5);
}

bitflags! {
    /// Defines the input type of a field.
    /// In traceloggingprovider.h, this is the 'TlgIn_t` enumerated type.
    #[repr(transparent)]
    pub struct InFlag: u8 {
        /// No value at all
        const NULL = 0;
        /// A wide string (UTF-16), corresponding to `PCWSTR` in Win32.
        const UNICODE_STRING = 1;
        /// An ANSI string, corresponding to `PCSTR` in Win32.
        /// The character set can be specified as UTF-8 by using `OutFlag::UTF8`.
        const ANSI_STRING = 2;
        /// `i8`
        const INT8 = 3;
        /// `u8`
        const UINT8 = 4;
        /// `i16`, stored in little-endian form.
        const INT16 = 5;
        /// `u16`, stored in little-endian form.
        const UINT16 = 6;
        /// `i16`, stored in little-endian form.
        const INT32 = 7;
        /// `u32`, stored in little-endian form.
        const UINT32 = 8;
        /// `i64`, stored in little-endian form.
        const INT64 = 9;
        /// `u64`, stored in little-endian form.
        const UINT64 = 10;
        /// `f32`, stored in little-endian form.
        const FLOAT = 11;
        /// `f64`, stored in little-endian form.
        const DOUBLE = 12;
        /// A Win32 'BOOL' value, which is `i32`, stored in little-endian form.
        const BOOL32 = 13;
        /// An array of bytes, stored in little-endian form.
        const BINARY = 14;
        /// A `GUID`, stored in canonical byte-oriented representation. The fields within the `GUID`
        /// are stored in big-endian form.
        const GUID = 15;
        // POINTER (16) is not supported
        /// A Win32 [`FILETIME`](https://docs.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-filetime)
        /// value. `FILETIME` values are `u64` values, stored in little-endian form, counting 100ns
        /// intervals from the `FILETIME` epoch.
        const FILETIME = 17;
        /// A Win32 [`SYSTEMTIME`](https://docs.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-systemtime)
        /// value, with fields encoded in little-endian form.
        const SYSTEMTIME = 18;
        /// A Win32 [`SID`](https://docs.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-sid).
        const SID = 19;
        /// An `i32` value, encoded in little-endian form, displayed in hexadecimal.
        const HEXINT32 = 20;
        /// An `i64` value, encoded in little-endian form, displayed in hexadecimal.
        const HEXINT64 = 21;
        /// A counted wide string (UTF-16), corresponding to `UNICODE_STRING` in Win32.
        /// This type uses two data descriptor slots. The first is a `u16` value, giving the
        /// length of the string data in WCHAR units (not bytes). The second points to the
        /// character data.
        const COUNTED_UNICODE_STRING = 22;
        /// A counted ANSI string, corresponding to `STRING` in Win32.
        /// The character set can be specified as UTF-8 by using `OutFlag::UTF8`.
        /// This type uses two data descriptor slots. The first is a `u16` value, giving the
        /// length of the string data in WCHAR units (not bytes). The second points to the
        /// character data.
        const COUNTED_ANSI_STRING = 23;
        /// A flag which indicates that this field is an array of constant length.
        /// If this field is present, then the metadata contains an additional `u16` field, which
        /// is the constant length.
        const CCOUNT_FLAG = 0x20;
        /// A flag which indicates that this field has a dynamic length. The field uses two
        /// data descriptors. The first is a `u16` field specifying the length of the array.
        /// The second points to the array data.
        const VCOUNT_FLAG = 0x40;
        /// A flag which indicates that this field metadata also includes an `OutFlag`. The
        /// `OutFlag` byte immediately follows the `InFlag` byte.
        const CHAIN_FLAG = 0b1000_0000;
        /// A flag which indicates that the field uses a custom serializer.
        const CUSTOM_FLAG = 0b0110_0000;
        /// A mask of the field type flags.
        const TYPE_MASK = 0b0001_1111;
        /// A mask of the field length flags (`VCOUNT_FLAG`, `CCOUNT_FLAG`, `CUSTOM_FLAG`).
        const COUNT_MASK = 0b0110_0000;
        /// A mask over all of the flags (all bits excluding the type bits).
        const FLAG_MASK = 0b1110_0000;
    }
}

impl InFlag {
    /// An alias for the architecture-dependent `USIZE` (pointer-sized word) `InFlag`.
    #[cfg(target_pointer_width = "32")]
    pub const USIZE: InFlag = InFlag::UINT32;

    /// An alias for the architecture-dependent `ISIZE` (pointer-sized word) `InFlag`.
    #[cfg(target_pointer_width = "32")]
    pub const ISIZE: InFlag = InFlag::INT32;

    /// An alias for the architecture-dependent `USIZE` (pointer-sized word) `InFlag`.
    #[cfg(target_pointer_width = "64")]
    pub const USIZE: InFlag = InFlag::UINT64;

    /// An alias for the architecture-dependent `ISIZE` (pointer-sized word) `InFlag`.
    #[cfg(target_pointer_width = "64")]
    pub const ISIZE: InFlag = InFlag::INT64;
}

bitflags! {
    /// Specifies how a field should be interpreted or displayed.
    #[repr(transparent)]
    pub struct OutFlag: u8 {
        /// No display at all.
        const NULL = 0;
        /// No display at all.
        const NOPRINT = 1;
        /// Contains text.
        const STRING = 2;
        /// Is a boolean. This can only be used with `InFlag::INT8`.
        const BOOLEAN = 3;
        /// Display in hexadecimal. Can be used with any integer type.
        const HEX = 4;
        /// The field is a Win32 process ID.
        const PID = 5;
        /// The field is a Win32 thread ID.
        const TID = 6;
        /// The field is a TCP/IP or UDP/IP port, in big-endian encoding.
        const PORT =  7;
        /// The field is an IP v4 address, in big-endian encoding.
        const IPV4 = 8;
        /// The field is an IP v6 address, in big-endian encoding.
        const IPV6 = 9;
        /// The field is a `SOCKADDR`. See `[SOCKADDR](https://docs.microsoft.com/en-us/windows/win32/api/winsock/ns-winsock-sockaddr)`.
        const SOCKETADDRESS = 10;
        /// The field is text, and should be interpreted as XML.
        const XML = 11;
        /// The field is text, and should be interpreted as JSON.
        const JSON = 12;
        /// The field is a `Win32` error code. The field type should be `InFlag::UINT32`.
        const WIN32ERROR = 13;
        /// The field is an `NTSTATUS` error code. The field type should be `InFlag::UINT32`.
        const NTSTATUS = 14;
        /// The field is an `HRESULT` error code. The field type should be `InFlag::INT32`.
        const HRESULT = 15;
        /// The field is a Win32 `FILETIME` value.
        const FILETIME = 16;
        /// Display field as signed.
        const SIGNED = 17;
        /// Display field as unsigned.
        const UNSIGNED = 18;
        /// For strings, indicates that the string encoding is UTF-8.
        const UTF8 = 35;
        /// Used with `InFlag::BINARY`.
        const PKCS7_WITH_TYPE_INFO = 36;
        // const CODE_POINTER = 37;

        /// Indicates that the timezone for a time value is UTC.
        /// This can be used with `InFlag::FILETIME` or `InFlag::SYSTEMTIME`.
        const DATETIME_UTC = 38;
    }
}
