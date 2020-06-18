use syn::parse_quote;
use win_etw_metadata::{InFlag, OutFlag};

pub struct WellKnownTypeInfo {
    pub ty: syn::Type,
    pub code: WellKnownType,
    pub in_type: InFlag,
    pub is_ref: bool,
    /// Indicates whether this type can be used in a slice, e.g. &[T].
    /// Should probably rename to `can_slice`.
    pub primitive: bool,
    pub opts: WellKnownTypeOptions,
}

#[derive(Default)]
pub struct WellKnownTypeOptions {
    pub out_type: Option<OutFlag>,
    pub in_type_expr: Option<syn::Expr>,
    pub replacement_type: Option<syn::Type>,
    #[allow(unused)]
    pub can_output_hex: bool,
}

macro_rules! well_known_types{
    (
        $(
            $t:ident: $tt:ty => {
                is_ref: $is_ref:expr,
                primitive: $primitive:expr,
                in_type: $in_type:expr,
                $( $opt_name:ident: $opt_value:expr, )*
            }
        )*
    ) => {
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub enum WellKnownType {
            $($t,)*
        }

        #[allow(non_snake_case)]
        pub struct WellKnownTypes {
            $(
                $t: WellKnownTypeInfo,
            )*
        }

        impl WellKnownTypes {
            pub fn new() -> Self {
                Self {
                    $(
                        $t: WellKnownTypeInfo {
                            ty: parse_quote!( $tt ),
                            code: WellKnownType::$t,
                            is_ref: $is_ref,
                            primitive: $primitive,
                            in_type: $in_type,
                            opts: WellKnownTypeOptions {
                                $($opt_name: $opt_value,)*
                                ..
                                WellKnownTypeOptions::default()
                            },
                        },
                    )*
                }
            }

            pub fn find(&self, ty: &syn::Type) -> Option<&WellKnownTypeInfo> {
                $(
                    if *ty == self.$t.ty {
                        return Some(&self.$t);
                    }
                )*
                None
            }
        }
    }
}

well_known_types! {
    bool: bool => {
        is_ref: false,
        primitive: true,
        in_type: InFlag::INT8,
        out_type: Some(OutFlag::BOOLEAN),
    }
    u8: u8 => { is_ref: false, primitive: true, in_type: InFlag::UINT8, can_output_hex: true, }
    u16: u16 => { is_ref: false, primitive: true, in_type: InFlag::UINT16, can_output_hex: true, }
    u32: u32 => { is_ref: false, primitive: true, in_type: InFlag::UINT32, can_output_hex: true, }
    u64: u64 => { is_ref: false, primitive: true, in_type: InFlag::UINT64, can_output_hex: true, }
    i8: i8 => { is_ref: false, primitive: true, in_type: InFlag::INT8, can_output_hex: true, }
    i16: i16 => { is_ref: false, primitive: true, in_type: InFlag::INT16, can_output_hex: true, }
    i32: i32 => { is_ref: false, primitive: true, in_type: InFlag::INT32, can_output_hex: true, }
    i64: i64 => { is_ref: false, primitive: true, in_type: InFlag::INT64, can_output_hex: true, }
    f32: f32 => { is_ref: false, primitive: true, in_type: InFlag::FLOAT, }
    f64: f64 => { is_ref: false, primitive: true, in_type: InFlag::DOUBLE, }
    usize: usize => { is_ref: false, primitive: true, in_type: InFlag::NULL,
        in_type_expr: Some(parse_quote!{
            ::win_etw_provider::metadata::InFlag::USIZE.bits()
        }),
        can_output_hex: true,
    }
    isize: isize => { is_ref: false, primitive: true, in_type: InFlag::NULL,
        in_type_expr: Some(parse_quote!{
            ::win_etw_provider::metadata::InFlag::ISIZE.bits()
        }),
        can_output_hex: true,
    }
    ref_str: &str => {
        is_ref: true,
        primitive: false,
        in_type: InFlag::COUNTED_ANSI_STRING,
        out_type: Some(OutFlag::UTF8),
    }
    u16cstr: &U16CStr => {
        is_ref: true,
        primitive: false,
        in_type: InFlag::COUNTED_UNICODE_STRING,
        replacement_type: Some(parse_quote!(&::widestring::U16CStr)),
    }
    osstr: &OsStr => {
        is_ref: true,
        primitive: false,
        in_type: InFlag::COUNTED_UNICODE_STRING,
        replacement_type: Some(parse_quote!(&::std::ffi::OsStr)),
    }
    guid: &GUID => {
        is_ref: true, primitive: false,
        in_type: InFlag::GUID,
        replacement_type: Some(parse_quote!(&::win_etw_provider::GUID)),
    }
    SocketAddrV4: &SocketAddrV4 => {
        is_ref: false,
        primitive: false,
        in_type: InFlag::BINARY,
        out_type: Some(OutFlag::SOCKETADDRESS),
        replacement_type: Some(parse_quote!(&::std::net::SocketAddrV4)),
    }
    SocketAddrV6: &SocketAddrV6 => {
        is_ref: false,
        primitive: false,
        in_type: InFlag::BINARY,
        out_type: Some(OutFlag::SOCKETADDRESS),
        replacement_type: Some(parse_quote!(&::std::net::SocketAddrV6)),
    }
    SocketAddr: &SocketAddr => {
        is_ref: false,
        primitive: false,
        in_type: InFlag::BINARY,
        out_type: Some(OutFlag::SOCKETADDRESS),
        replacement_type: Some(parse_quote!(&::std::net::SocketAddr)),
    }
    SystemTime: SystemTime => {
        is_ref: false,
        primitive: false,
        in_type: InFlag::FILETIME,
        replacement_type: Some(parse_quote!(::std::time::SystemTime)),
    }
    FILETIME: FILETIME => {
        is_ref: true,
        primitive: false,
        in_type: InFlag::FILETIME,
        replacement_type: Some(parse_quote!(::win_etw_provider::FILETIME)),
    }
    HRESULT: HRESULT => {
        is_ref: false,
        primitive: false,
        in_type: InFlag::INT32,
        replacement_type: Some(parse_quote!(i32)),
        out_type: Some(OutFlag::HRESULT),
    }
    WIN32ERROR: WIN32ERROR => {
        is_ref: false,
        primitive: false,
        in_type: InFlag::UINT32,
        replacement_type: Some(parse_quote!(u32)),
        out_type: Some(OutFlag::WIN32ERROR),
    }
    NTSTATUS: NTSTATUS => {
        is_ref: false,
        primitive: false,
        in_type: InFlag::UINT32,
        replacement_type: Some(parse_quote!(u32)),
        out_type: Some(OutFlag::NTSTATUS),
    }
}
