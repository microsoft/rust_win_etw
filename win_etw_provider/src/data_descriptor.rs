use crate::guid::GUID;
use core::marker::PhantomData;
use core::mem::size_of;
use widestring::U16CStr;
use zerocopy::AsBytes;

/// Contains a reference to the data for an event field. The type of the data is not specified in
/// this structure; instead, the type of the data is stored in the event's metadata.
/// (See `win_etw_metadata::InFlag`.)
///
/// The data that this type points to must have a well-defined (stable) byte representation. For
/// example, `u32` has a well-defined byte representation, as long as there is agreement about
/// whether the value is stored in big-endian or little-endian order. Similarly, `[u8]` and
/// `[u32]` have well-defined byte representations. However, types such as `[bool]` do not have a
/// stable byte representation, and so `EventDataDescriptor` cannot point to `&[bool]`.
///
/// This type provides implementations of `From` that can be used to point to event data.
/// All of the `EventDataDescriptor::From` implementations for types require that the types have a
/// stable, guaranteed byte representation, and that is legal (meaningful) to read that byte
/// representation.
///
/// This type is equivalent to the Win32 structure `EVENT_DATA_DESCRIPTOR`, and its representation
/// is guaranteed to be equivalent.
///
/// # Implementation warning!
///
/// This code is responsible for ensuring memory safety. Even though it contains only simple
/// primitives, these primitives are actually native pointers and pointer bounds. This data
/// structure is passed to the ETW implementation, which dereferences those pointers. The Rust
/// type checker cannot "see" these dereferences, since they occur in non-Rust code, so the borrow
/// checker does not know that `EventDataDescriptor` deals with memory safety. This is why the
/// `phantom_ref` field exists, and it is _crucial_ that this code be used and encapsulated
/// correctly.
///
/// For type safety to be conserved, the following invariants *must* be maintained:
///
/// * The `'a` lifetime parameter of `EventDataDescriptor<'a>` must be correctly associated with
///   the lifetime of any reference that is used to construct an instance of `EventDataDescriptor`.
///
/// * The fields of `EventDataDescriptor` must remain private. Arbitrary user code cannot be
///   permitted to construct instances of `EventDataDescriptor` with arbitrary values for these
///   fields.
///
/// * `EventDataDescriptor` should only be used to pass to ETW functions.
#[repr(C)]
#[derive(Clone)]
pub struct EventDataDescriptor<'a> {
    // descriptor: evntprov::EVENT_DATA_DESCRIPTOR,
    ptr: u64,
    size: u32,

    /// In the Windows SDK, this field is marked "reserved" and is a union. However, it is clear
    /// from usage within `traceloggingprovider.h` that this field is used to identify event data,
    /// provider metadata, and event metadata.
    kind: u32,

    /// Represents the lifetime of the pointed-to data.
    phantom_ref: PhantomData<&'a ()>,
}

impl EventDataDescriptor<'static> {
    /// Returns an empty data descriptor.
    pub fn empty() -> Self {
        Self {
            ptr: 0,
            size: 0,
            kind: 0,
            phantom_ref: PhantomData,
        }
    }

    /// Creates an `EventDataDescriptor` for provider metadata.
    /// Provider metadata is required to have `'static` lifetime.
    pub fn for_provider_metadata(s: &'static [u8]) -> Self {
        Self {
            ptr: s.as_ptr() as usize as u64,
            size: s.len() as u32,
            kind: EVENT_DATA_DESCRIPTOR_TYPE_PROVIDER_METADATA,
            phantom_ref: PhantomData,
        }
    }

    /// Creates an `EventDataDescriptor` for the metadata that describes a single event.
    /// Event metadata is required to have `'static` lifetime.
    pub fn for_event_metadata(s: &'static [u8]) -> Self {
        Self {
            ptr: s.as_ptr() as usize as u64,
            size: s.len() as u32,
            kind: EVENT_DATA_DESCRIPTOR_TYPE_EVENT_METADATA,
            phantom_ref: PhantomData,
        }
    }
}

const EVENT_DATA_DESCRIPTOR_TYPE_PROVIDER_METADATA: u32 = 2;
const EVENT_DATA_DESCRIPTOR_TYPE_EVENT_METADATA: u32 = 1;

impl<'a> EventDataDescriptor<'a> {
    /// Creates a `EventDataDescriptor for a slice of bytes.
    pub fn for_bytes(s: &'a [u8]) -> Self {
        Self {
            ptr: s.as_ptr() as usize as u64,
            size: s.len() as u32,
            kind: 0,
            phantom_ref: PhantomData,
        }
    }
}

macro_rules! well_known_types {
    (
        $(
            $t:ident ;
        )*
    ) => {
        $(
            impl<'a> From<&'a $t> for EventDataDescriptor<'a> {
                fn from(value: &'a $t) -> EventDataDescriptor<'a> {
                    EventDataDescriptor::for_bytes(value.as_bytes())
                }
            }

            impl<'a> From<&'a [$t]> for EventDataDescriptor<'a> {
                fn from(value: &'a [$t]) -> EventDataDescriptor<'a> {
                    EventDataDescriptor::for_bytes(value.as_bytes())
                }
            }
        )*
    }
}

well_known_types! {
    u8; u16; u32; u64;
    i8; i16; i32; i64;
    f32; f64;
    usize; isize;
}

impl<'a> From<&'a str> for EventDataDescriptor<'a> {
    fn from(value: &'a str) -> EventDataDescriptor<'a> {
        let bytes: &'a [u8] = value.as_bytes();
        EventDataDescriptor::for_bytes(bytes)
    }
}

impl<'a> From<&'a U16CStr> for EventDataDescriptor<'a> {
    fn from(value: &'a U16CStr) -> EventDataDescriptor<'a> {
        Self {
            ptr: value.as_ptr() as usize as u64,
            size: (value.len() * 2 + 1) as u32,
            kind: 0,
            phantom_ref: PhantomData,
        }
    }
}

impl<'a> From<&'a GUID> for EventDataDescriptor<'a> {
    fn from(value: &'a GUID) -> EventDataDescriptor<'a> {
        Self {
            ptr: value as *const GUID as usize as u64,
            size: size_of::<GUID>() as u32,
            kind: 0,
            phantom_ref: PhantomData,
        }
    }
}
