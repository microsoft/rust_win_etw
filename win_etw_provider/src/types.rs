use crate::EventDataDescriptor;
use zerocopy::{AsBytes, FromBytes};

/// The value used in `SocketAddrV4::family` to identify IPv4 addresses.
pub const AF_INET: u16 = 2;

/// The value used in `SocketAddrV6::family` to identify IPv6 addresses.
pub const AF_INET6: u16 = 23;

/// This has the same in-memory representation as the Win32 SOCKADDR_IN structure.
/// https://docs.microsoft.com/en-us/windows/win32/api/ws2def/ns-ws2def-sockaddr_in
#[repr(C)]
#[derive(AsBytes, Clone)]
pub struct SocketAddrV4 {
    /// Address family identifier.
    pub family: u16,
    /// Port identifier, stored in big-endian form.
    pub port: [u8; 2],
    /// IPv4 address, stored in big-endian form.
    pub address: [u8; 4],
    /// Zero padding.
    pub zero: [u8; 8],
}

#[cfg(feature = "std")]
impl From<&std::net::SocketAddrV4> for SocketAddrV4 {
    fn from(value: &std::net::SocketAddrV4) -> Self {
        let port = value.port();
        Self {
            family: AF_INET,
            address: value.ip().octets(),
            port: port.to_be_bytes(),
            zero: [0; 8],
        }
    }
}

impl<'a> From<&'a crate::types::SocketAddrV4> for EventDataDescriptor<'a> {
    fn from(value: &'a crate::types::SocketAddrV4) -> EventDataDescriptor<'a> {
        Self::from(value.as_bytes())
    }
}

/// See `[SOCKADDR_IN6_LH](https://docs.microsoft.com/en-us/windows/win32/api/ws2ipdef/ns-ws2ipdef-sockaddr_in6_lh)`.
#[repr(C)]
#[derive(Clone, AsBytes, FromBytes)]
pub struct SocketAddrV6 {
    /// Address family identifier.
    pub family: u16,
    /// Port identifier, stored in big-endian form.
    pub port: [u8; 2],
    /// IPv6 flow info.
    pub flow_info: [u8; 4],
    /// IPv6 address.
    pub address: [u8; 16],
    /// IPv6 scope.
    pub scope_id: [u8; 4],
}

#[cfg(feature = "std")]
impl From<&std::net::SocketAddrV6> for SocketAddrV6 {
    fn from(value: &std::net::SocketAddrV6) -> Self {
        Self {
            family: AF_INET6,
            port: value.port().to_be_bytes(),
            flow_info: value.flowinfo().to_be_bytes(),
            address: value.ip().octets(),
            scope_id: value.scope_id().to_be_bytes(),
        }
    }
}

impl<'a> From<&'a crate::types::SocketAddrV6> for EventDataDescriptor<'a> {
    fn from(value: &'a crate::types::SocketAddrV6) -> EventDataDescriptor<'a> {
        Self::from(value.as_bytes())
    }
}

/// See `[FILETIME](https://docs.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-filetime)`.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct FILETIME(pub u64);

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use core::convert::TryFrom;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    /// Time elapsed between the Windows epoch and the UNIX epoch.
    const WINDOWS_EPOCH_TO_UNIX_EPOCH: Duration = Duration::from_secs(11_644_473_600);

    pub struct OutOfRangeError;

    impl TryFrom<SystemTime> for FILETIME {
        type Error = OutOfRangeError;
        fn try_from(t: SystemTime) -> Result<Self, Self::Error> {
            match t.duration_since(UNIX_EPOCH) {
                Ok(unix_elapsed) => {
                    let windows_elapsed: Duration = unix_elapsed + WINDOWS_EPOCH_TO_UNIX_EPOCH;
                    Ok(FILETIME((windows_elapsed.as_nanos() / 100) as u64))
                }
                Err(_) => Err(OutOfRangeError),
            }
        }
    }
}
