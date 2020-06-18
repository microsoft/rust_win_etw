use zerocopy::{AsBytes, FromBytes};

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
#[derive(Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, AsBytes, FromBytes)]
pub struct GUID {
    /// Contains bytes 0-3 (inclusive) of the GUID, in big-endian form.
    pub data1: u32,
    /// Contains bytes 4-5 (inclusive) of the GUID, in big-endian form.
    pub data2: u16,
    /// Contains bytes 6-7 (inclusive) of the GUID, in big-endian form.
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
impl From<winapi::shared::guiddef::GUID> for GUID {
    fn from(value: winapi::shared::guiddef::GUID) -> Self {
        Self {
            data1: value.Data1,
            data2: value.Data2,
            data3: value.Data3,
            data4: value.Data4,
        }
    }
}
