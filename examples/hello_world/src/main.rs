#![allow(clippy::unreadable_literal)]
#![forbid(unsafe_code)]
use win_etw_macros::trace_logging_provider;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::time::{Duration, SystemTime};
use widestring::{U16CString, U16String};
use win_etw_provider::{guid, FILETIME, GUID};

// {861A3948-3B6B-4DDF-B862-B2CB361E238E}
// DEFINE_GUID(my_provider_guid, 0x861a3948, 0x3b6b, 0x4ddf, 0xb8, 0x62, 0xb2, 0xcb, 0x36, 0x1e, 0x23, 0x8e);
const EXAMPLE_GUID: GUID =
    guid!(0x861a3948, 0x3b6b, 0x4ddf, 0xb8, 0x62, 0xb2, 0xcb, 0x36, 0x1e, 0x23, 0x8e);

fn main() {
    let hello_provider = HelloWorldProvider::new();

    hello_provider.arg_str(None, "Hello, world!");
    hello_provider.arg_slice_u8(None, &[44, 55, 66]);
    hello_provider.arg_slice_i32(None, &[10001, 20002, 30003]);
    hello_provider.arg_f32(None, core::f32::consts::PI);
    hello_provider.arg_guid(None, &EXAMPLE_GUID);

    let client_addr_v4: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 23, 42), 6667);
    hello_provider.client_connected_v4(None, &client_addr_v4);

    let client_addr_v6 = "[2001:db8::1]:8080".parse::<SocketAddrV6>().unwrap();
    hello_provider.client_connected_v6(None, &client_addr_v6);

    hello_provider.client_connected(None, &SocketAddr::V4(client_addr_v4));
    hello_provider.client_connected(None, &SocketAddr::V6(client_addr_v6));

    hello_provider.something_bad_happened(None, "uh oh!");

    hello_provider.file_created(None, SystemTime::now());
    hello_provider.file_created_filetime(
        None,
        FILETIME((11644473600 + (3 * 365 + 31 + 28 + 31 + 30 + 31 + 15) * 86400) * 10_000_000),
    );

    hello_provider.arg_u32_hex(None, 0xcafef00d);

    use std::ffi::OsString;

    hello_provider.arg_u16str(None, &U16String::from_str("this is a u16str"));
    hello_provider.arg_u16cstr(None, &U16CString::from_str("this is a u16cstr").unwrap());
    hello_provider.arg_osstr(None, &OsString::from("hello!"));

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::{STATUS_ABIOS_INVALID_COMMAND, ERROR_SXS_UNTRANSLATABLE_HRESULT, ERROR_OUT_OF_PAPER};

        hello_provider.arg_hresult(None, ERROR_SXS_UNTRANSLATABLE_HRESULT as i32);
        hello_provider.arg_ntstatus(None, STATUS_ABIOS_INVALID_COMMAND as u32);
        hello_provider.arg_win32error(None, ERROR_OUT_OF_PAPER);
    }

    let args = std::env::args().collect::<Vec<String>>();
    if args.len() >= 2 && args[1] == "loop" {
        eprintln!("looping");
        loop {
            std::thread::sleep(Duration::from_millis(3000));
            hello_provider.hello(None, "Looping...  (from Rust)");
            hello_provider.message_at_critical(None, "something super exciting happened!");
            hello_provider.message_at_error(None, "something pretty bad happened!");
            hello_provider.message_at_warn(None, "something warning-worthy happened");
            hello_provider.message_at_info(None, "something normal happened");
            hello_provider.message_at_verbose(None, "noisy noisy noisy");
            hello_provider.message_at_level_8(None, "incredibly detailed level 8 tracing");
        }
    }
}

/// Hello, World, from ETW
#[trace_logging_provider(
    guid = "861A3948-3B6B-4DDF-B862-B2CB361E238E",
    provider_group_guid = "6aeb6059-444a-4606-a3ea-06fd00fe3378"
)]
trait HelloWorldProvider {
    fn hello(a: &str);
    fn arg_i32(a: i32);
    fn arg_u8(a: u8);

    /// Log a floating point value.
    #[event(level = "info")]
    fn arg_f32(a: f32);

    fn arg_slice_u8(arg: &[u8]);
    fn arg_slice_i32(arg: &[i32]);
    fn arg_str(arg: &str);

    fn arg_guid(arg: &GUID);

    #[event(level = "error")]
    fn something_bad_happened(message: &str);

    #[event(task = 42, opcode = 99)]
    fn client_connected_v4(client_addr: &SocketAddrV4);

    #[event(task = 42, opcode = 99)]
    fn client_connected_v6(client_addr: &SocketAddrV6);

    #[event(task = 42, opcode = 99)]
    fn client_connected(client_addr: &SocketAddr);

    fn file_created(create_time: SystemTime);

    fn file_created_filetime(t: FILETIME);

    fn arg_bool(a: bool);

    fn arg_usize(a: usize);
    fn arg_isize(a: isize);

    fn arg_u32_hex(#[event(output = "hex")] a: u32);

    fn arg_hresult(a: HRESULT);
    fn arg_ntstatus(a: NTSTATUS);
    fn arg_win32error(a: WIN32ERROR);

    fn arg_u16str(a: &U16Str);
    fn arg_u16cstr(a: &U16CStr);
    fn arg_osstr(a: &OsStr);

    #[event(level = "critical")]
    fn message_at_critical(msg: &str);

    #[event(level = "info")]
    fn message_at_info(msg: &str);

    #[event(level = "warn")]
    fn message_at_warn(msg: &str);

    #[event(level = "error")]
    fn message_at_error(msg: &str);

    #[event(level = "verbose")]
    fn message_at_verbose(msg: &str);

    #[event(level = 8)]
    fn message_at_level_8(msg: &str);
}

#[trace_logging_provider(guid = "76d66486-d11a-47a8-af05-88942b6edb55")]
trait AnotherFineProvider {
    fn arg_str(arg: &str);
}

#[trace_logging_provider(guid = "b9978f10-b3e0-4bbe-a4f2-160a2e7148d6")]
trait TestManyEvents {
    fn arg_none();
    fn arg_bool(a: bool);
    fn arg_u8(a: u8);
    fn arg_u16(a: u16);
    fn arg_u32(a: u32);
    fn arg_u64(a: u64);
    fn arg_i8(a: i8);
    fn arg_i16(a: i16);
    fn arg_i32(a: i32);
    fn arg_i64(a: i64);
    fn arg_f32(a: f32);
    fn arg_f64(a: f64);
    fn arg_usize(a: usize);
    fn arg_isize(a: isize);

    fn arg_slice_bool(a: &[bool]);
    fn arg_slice_u8(a: &[u8]);
    fn arg_slice_u16(a: &[u16]);
    fn arg_slice_u32(a: &[u32]);
    fn arg_slice_u64(a: &[u64]);
    fn arg_slice_i8(a: &[i8]);
    fn arg_slice_i16(a: &[i16]);
    fn arg_slice_i32(a: &[i32]);
    fn arg_slice_i64(a: &[i64]);
    fn arg_slice_f32(a: &[f32]);
    fn arg_slice_f64(a: &[f64]);
    fn arg_slice_usize(a: &[usize]);
    fn arg_slice_isize(a: &[isize]);

    fn arg_str(arg: &str);
    fn arg_guid(arg: &GUID);
    fn arg_system_time(a: SystemTime);
    fn arg_filetime(a: FILETIME);

    #[event(level = "critical")]
    fn arg_u8_at_critical(a: u8);

    #[event(level = "info")]
    fn arg_u8_at_info(a: u8);

    #[event(level = "warn")]
    fn arg_u8_at_warn(a: u8);

    #[event(level = "error")]
    fn arg_u8_at_error(a: u8);

    #[event(level = "verbose")]
    fn arg_u8_at_verbose(a: u8);

    #[event(level = 8)]
    fn arg_u8_at_level_8(a: u8);

    #[event(task = 100)]
    fn arg_with_task(a: u8);

    #[event(opcode = 10)]
    fn arg_with_opcode(a: u8);

    fn arg_u32_hex(#[event(output = "hex")] a: u32);

    fn arg_hresult(a: HRESULT);
    fn arg_ntstatus(a: NTSTATUS);
    fn arg_win32error(a: WIN32ERROR);
}

#[trace_logging_provider]
trait ProviderWithAutogeneratedGuid {
    fn arg_str(arg: &str);
}

#[trace_logging_provider(name = "Your.Provider.Name")]
trait ProviderWithAutogeneratedGuidAndCustomProviderName {
    fn arg_str(arg: &str);
}

#[trace_logging_provider(
    name = "Another.Provider.Name",
    guid = "86fe5bee-dec7-40f2-b39b-a46c20633042"
)]
trait ProviderWithCustomGuidAndCustomProviderName {
    fn arg_str(arg: &str);
}

#[cfg(test)]
mod tests {
    use super::{
        ProviderWithAutogeneratedGuid, ProviderWithAutogeneratedGuidAndCustomProviderName,
        ProviderWithCustomGuidAndCustomProviderName,
    };
    use win_etw_provider::guid;

    #[test]
    fn provider_with_autogenerated_guid() {
        assert_eq!(
            "ProviderWithAutogeneratedGuid",
            ProviderWithAutogeneratedGuid::PROVIDER_NAME
        );

        assert_eq!(
            guid!(
                0x0d31f5cc,
                0xfb84,
                0x50db,
                [0xa6, 0x02, 0x8c, 0x7b, 0xed, 0x9c, 0x5b, 0x8b]
            ),
            ProviderWithAutogeneratedGuid::PROVIDER_GUID,
        );
    }

    #[test]
    fn provider_with_autogenerated_guid_and_custom_provider_name() {
        assert_eq!(
            "Your.Provider.Name",
            ProviderWithAutogeneratedGuidAndCustomProviderName::PROVIDER_NAME
        );

        assert_eq!(
            guid!(
                0x0cec4c9d,
                0xcaa7,
                0x5d85,
                [0xb3, 0xbd, 0xc7, 0x35, 0x77, 0xf0, 0x3f, 0xd8]
            ),
            ProviderWithAutogeneratedGuidAndCustomProviderName::PROVIDER_GUID
        );
    }

    #[test]
    fn provider_with_custom_guid_and_custom_provider_name() {
        assert_eq!(
            "Another.Provider.Name",
            ProviderWithCustomGuidAndCustomProviderName::PROVIDER_NAME
        );

        assert_eq!(
            guid!(
                0x86fe5bee,
                0xdec7,
                0x40f2,
                [0xb3, 0x9b, 0xa4, 0x6c, 0x20, 0x63, 0x30, 0x42]
            ),
            ProviderWithCustomGuidAndCustomProviderName::PROVIDER_GUID
        );
    }
}
