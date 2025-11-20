use uuid::Uuid;
use zerocopy::{FromBytes, IntoBytes};

/// Initializes a `GUID` from literal values.
#[macro_export]
macro_rules! guid {
    (
        $a:expr,
        $b:expr,
        $c:expr,
        $d:expr
    ) => {
        $crate::GUID {
            data1: $a,
            data2: $b,
            data3: $c,
            data4: $d,
        }
    };

    (
        $a:expr,
        $b:expr,
        $c:expr,
        $d0:expr,
        $d1:expr,
        $d2:expr,
        $d3:expr,
        $d4:expr,
        $d5:expr,
        $d6:expr,
        $d7:expr
    ) => {
        $crate::GUID {
            data1: $a,
            data2: $b,
            data3: $c,
            data4: [$d0, $d1, $d2, $d3, $d4, $d5, $d6, $d7],
        }
    };
}

/// The Windows [`GUID`](https://docs.microsoft.com/en-us/windows/win32/api/guiddef/ns-guiddef-guid)
/// type.
///
/// `win_etw_provider` defines this type, rather than directly referencing (or re-exporting)
/// an equivalent type from other crates in order to minimize its dependencies. `GUID` has a well-
/// defined byte representation, so converting between different implementations of `GUID` is
/// not a problem.
#[repr(C)]
#[derive(Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, IntoBytes, FromBytes)]
pub struct GUID {
    /// Contains bytes 0-3 (inclusive) of the GUID.
    pub data1: u32,
    /// Contains bytes 4-5 (inclusive) of the GUID.
    pub data2: u16,
    /// Contains bytes 6-7 (inclusive) of the GUID.
    pub data3: u16,
    /// Contains bytes 8-15 (inclusive) of the GUID.
    pub data4: [u8; 8],
}

impl core::fmt::Display for GUID {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            fmt,
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.data1,
            self.data2,
            self.data3,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7]
        )
    }
}

impl core::fmt::Debug for GUID {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, fmt)
    }
}

#[cfg(target_os = "windows")]
impl From<windows_sys::core::GUID> for GUID {
    fn from(value: windows_sys::core::GUID) -> Self {
        Self {
            data1: value.data1,
            data2: value.data2,
            data3: value.data3,
            data4: value.data4,
        }
    }
}

#[cfg(target_os = "windows")]
impl From<GUID> for windows_sys::core::GUID {
    fn from(value: GUID) -> Self {
        Self {
            data1: value.data1,
            data2: value.data2,
            data3: value.data3,
            data4: value.data4,
        }
    }
}

#[cfg(feature = "uuid")]
impl From<uuid::Uuid> for GUID {
    fn from(value: uuid::Uuid) -> Self {
        let fields = value.as_fields();
        Self {
            data1: fields.0,
            data2: fields.1,
            data3: fields.2,
            data4: *fields.3,
        }
    }
}

impl From<&str> for GUID {
    fn from(value: &str) -> Self {
        let uuid = uuid::Uuid::parse_str(value).expect("Invalid string");
        uuid.into()
    }
}

#[cfg(feature = "uuid")]
#[cfg(test)]
mod test {
    use crate::guid::GUID;
    use uuid::Uuid;
    #[test]
    fn test_uuid() {
        let uuid = Uuid::parse_str("1a1a1a1a-2b2b-3c3c-4142-434546474849").unwrap();
        let guid: GUID = uuid.into();
        assert_eq!(guid.data1, 0x1a1a_1a1a);
        assert_eq!(guid.data2, 0x2b2b);
        assert_eq!(guid.data3, 0x3c3c);
        assert_eq!(guid.data4, [0x41, 0x42, 0x43, 0x45, 0x46, 0x47, 0x48, 0x49]);
    }

    #[test]
    fn test_from_str() {
        let guid_str = "1a1a1a1a-2b2b-3c3c-4142-434546474849";
        let guid: GUID = GUID::from(guid_str);
        assert_eq!(guid.data1, 0x1a1a_1a1a);
        assert_eq!(guid.data2, 0x2b2b);
        assert_eq!(guid.data3, 0x3c3c);
        assert_eq!(guid.data4, [0x41, 0x42, 0x43, 0x45, 0x46, 0x47, 0x48, 0x49]);
    }
}
