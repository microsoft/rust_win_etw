use super::*;

struct CompileErrors {
    errors: Vec<String>,
}

use proc_macro2::TokenTree;
use syn::buffer::Cursor;

impl syn::parse::Parse for CompileErrors {
    fn parse(s: syn::parse::ParseStream) -> syn::Result<Self> {
        s.step(|c| {
            let mut c: Cursor = *c;

            let mut errors = Vec::new();

            while !c.eof() {
                if let Some((i, next)) = c.ident() {
                    if i == "compile_error" {
                        if let Some((p, next)) = next.punct() {
                            if p.as_char() == '!' {
                                if let Some((TokenTree::Group(args), next)) = next.token_tree() {
                                    // println!("found compile_error!(...): {:?}", args);
                                    let real_args: syn::LitStr = syn::parse2(args.stream())?;
                                    // println!("real_args: {:?}", real_args);
                                    errors.push(real_args.value());
                                    // errors.push(args);
                                    c = next;
                                    continue;
                                }
                            }
                        }
                    }
                }
                // Didn't recognize it.
                if let Some((_ignored, next)) = c.token_tree() {
                    c = next;
                } else {
                    println!("cursor is positioned on something that is not a token tree!");
                    break;
                }
            }

            Ok((Self { errors }, Cursor::empty()))
        })
    }
}

fn test_worker(attrs: TokenStream, input: TokenStream, expected_errors: &[&'static str]) {
    let output = trace_logging_events_core::<false>(attrs, input);

    // Set WIN_ETW_SHOW_OUTPUT=1 (or = anything at all) to see the output of
    // the trace_logging_provider macro for unit tests. This is useful during
    // development.
    if std::env::var("WIN_ETW_SHOW_OUTPUT").is_ok() {
        let output_str = format!("{}", output);
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut rustfmt_cmd = Command::new("rustfmt")
            .stdin(Stdio::piped())
            .spawn()
            .expect("rustfmt failed");
        let mut child_stdin = rustfmt_cmd.stdin.take().unwrap();
        child_stdin.write_all(output_str.as_bytes()).unwrap();
        drop(child_stdin);
        rustfmt_cmd.wait().unwrap();
    }

    // Scan 'output' for errors.
    let errors: CompileErrors = syn::parse2(output).unwrap();
    if expected_errors.is_empty() {
        assert!(
            errors.errors.is_empty(),
            "Macro produced errors:\n{:#?}",
            errors.errors
        );
    } else {
        // For each of the errors in expected_errors, scan the list of actual errors.
        // Do a simple substring search.
        for &expected_error in expected_errors.iter() {
            if errors.errors.iter().any(|e| {
                // println!("checking in {:?}", e);
                e.contains(expected_error)
            }) {
                // println!("found expected error {:?}", expected_error);
            } else {
                panic!(
                    "Did not find expected error {:?} in list:\n{:#?}",
                    expected_error, errors.errors
                );
            }
        }
    }

    // Make sure all errors found in compilation is expected.
    for error in errors.errors {
        if expected_errors.iter().any(|e| {
            // println!("checking in {:?}", e);
            let s = String::from(*e);
            error.contains(&s)
        }) {
            // println!("found expected error {:?}", expected_error);
        } else {
            panic!(
                "Did not match error {:?} with expected error in list:\n{:#?}",
                error, expected_errors
            );
        }
    }
}

macro_rules! test_case {
    (
        #[test]
        fn $test_case_name:ident();

        input: {
            #[trace_logging_provider ( $( $attrs:tt )* )]
            $( $input:tt )*
        }

        expected_errors: [
            $( $error:expr, )*
        ]

    ) => {
        #[test]
        fn $test_case_name() {
            let attrs = quote!{ $( $attrs )* };

            let input = quote!{ $( $input )* };
            let expected_errors = [ $( $error, )* ];
            test_worker(attrs, input, &expected_errors);

        }
    }
}

test_case! {
    #[test]
    fn test_many_types();
    input: {
        #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
        trait Events {
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
            fn arg_u16cstr(arg: &U16CStr);
            fn arg_guid(arg: &GUID);
            fn arg_system_time(a: SystemTime);
            fn arg_filetime(a: FILETIME);

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
    }
    expected_errors: []
}

test_case! {
    #[test]
    fn test_unsupported_field_types();
    input: {
        #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
        trait Events {
            fn event(a: ());
        }
    }
    expected_errors: [
        "This type is not supported for event parameters.",
    ]
}

test_case! {
    #[test]
    fn test_event_return_type();
    input: {
        #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
        trait Events {
            fn event() -> String;
        }
    }
    expected_errors: [
        "Event methods must not return data.",
    ]
}

test_case! {
    #[test]
    fn test_event_default_implementation();
    input: {
        #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
        trait Events {
            fn event() { }
        }
    }
    expected_errors: [
        "Event methods must not contain an implementation.",
    ]
}

test_case! {
    #[test]
    fn test_event_generic();
    input: {
        #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
        trait Events {
            fn event<T>();
        }
    }
    expected_errors: [
        "Generic event methods are not supported.",
    ]
}

test_case! {
    #[test]
    fn test_event_generic_lifetime();
    input: {
        #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
        trait Events {
            fn event<'a>();
        }
    }
    expected_errors: [
        "Generic event methods are not supported.",
    ]
}

test_case! {
    #[test]
    fn test_wrong_self_ref();
    input: {
        #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
        trait Events {
            fn event(&self);
        }
    }
    expected_errors: [
        "Event methods should not provide any receiver arguments",
    ]
}

test_case! {
    #[test]
    fn test_wrong_self_mut();
    input: {
        #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
        trait Events {
            fn event(&mut self);
        }
    }
    expected_errors: [
        "Event methods should not provide any receiver arguments",
    ]
}

test_case! {
    #[test]
    fn test_wrong_self_move();
    input: {
        #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
        trait Events {
            fn event(self);
        }
    }
    expected_errors: [
        "Event methods should not provide any receiver arguments",
    ]
}

test_case! {
    #[test]
    fn test_bad_guid_literal();
    input: {
        #[trace_logging_provider(guid = 0)]
        trait Events {}
    }
    expected_errors: [
        "The attribute value is required to be a GUID in string form.",
    ]
}

test_case! {
    #[test]
    fn test_all_event_ids();
    input: {
        #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
        trait Events {
            #[event(id = 1)]
            fn event_one();

            #[event(id = 2)]
            fn event_two();
        }
    }
    expected_errors: [
    ]
}

test_case! {
    #[test]
    fn test_mixed_event_ids_fail();
    input: {
        #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
        trait Events {
            #[event(id = 1)]
            fn event_one();

            fn event_two();
        }
    }

    expected_errors: [
        "Event ids must be set for all events, or for none.",
    ]
}

test_case! {
    #[test]
    fn test_duplicate_event_ids_fail();
    input: {
        #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
        trait Events {
            #[event(id = 1, id = 2)]
            fn event_one();

            #[event(id = 3)]
            fn event_two();
        }
    }

    expected_errors: [
        "Event id has already been defined.",
    ]
}

test_case! {
        #[test]
        fn test_single_repeated_event_ids_fail();
        input: {
            #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
            trait Events {
                #[event(id = 1)]
                fn event_one();

                #[event(id = 1)]
                fn event_two();
            }
        }

    expected_errors: [
        "Event id 1 has already been defined on event_one.",
    ]
}

test_case! {
        #[test]
        fn test_multiple_repeated_event_ids_fail();
        input: {
            #[trace_logging_provider(guid = "610259b8-9270-46f2-ad94-2f805721b287")]
            trait Events {
                #[event(id = 1)]
                fn event_one();

                #[event(id = 2)]
                fn event_two();

                #[event(id = 1)]
                fn event_three();

                #[event(id = 2)]
                fn event_four();
            }
        }

    expected_errors: [
        "Event id 1 has already been defined on event_one.",
        "Event id 2 has already been defined on event_two.",
    ]
}

test_case! {
    #[test]
    fn test_bad_multiple_errors();
    input: {
        #[trace_logging_provider(guid = "bad guid")]
        trait Events {
            fn bad_arg(a: ());
        }
    }
    expected_errors: [
        "The attribute value is required to be a valid GUID.",
        "This type is not supported for event parameters.",
    ]
}

test_case! {
    #[test]
    fn test_invalid_event_attributes();
    input: {
        #[trace_logging_provider(guid = "00000000-0000-0000-0000-000000000001")]
        trait Events {
            #[event(bad_name = "bad_value")]
            fn event();
        }
    }
    expected_errors: [
        "Unrecognized attribute.",
    ]
}

test_case! {
    #[test]
    fn test_event_attributes_others_forbidden();
    input: {
        #[trace_logging_provider(guid = "00000000-0000-0000-0000-000000000001")]
        trait Events {
            #[some_other_attribute]
            fn event();
        }
    }
    expected_errors: [
        "The only attributes allowed on event methods are #[doc] and #[event(...)] attributes.",
    ]
}

test_case! {
    #[test]
    fn wrong_item_kind();
    input: {
        #[trace_logging_provider()]
        fn wrong_item_kind() {}
    }
    expected_errors: [
        "The #[trace_logging_provider] attribute cannot be used with this kind of item.",
    ]
}

test_case! {
    #[test]
    fn multiple_errors_detected();
    input: {
        #[trace_logging_provider()]
        trait Events
        {
            #[some_other_attribute]
            fn event_one();

            #[event(id = 1)]
            fn event_two();

            #[event(id = 2)]
            fn event_three();

            #[event(id = 1)]
            fn event_four();
        }

    }
    expected_errors: [
        "The only attributes allowed on event methods are #[doc] and #[event(...)] attributes.",
        "Event ids must be set for all events, or for none.",
        "Event id 1 has already been defined on event_two.",
    ]
}

test_case! {
    #[test]
    fn provider_groups();
    input: {
        #[trace_logging_provider(
            guid = "00000000-0000-0000-0000-000000000001",
            provider_group_guid = "00000000-0000-0000-0000-000000000002",
        )]
        trait Events
        {
            fn foo();
        }
    }
    expected_errors: [
    ]
}

test_case! {
    #[test]
    fn provider_attributes_multiple_provider_groups();
    input: {
        #[trace_logging_provider(
            guid = "00000000-0000-0000-0000-000000000001",
            provider_group_guid = "00000000-0000-0000-0000-000000000002",
            provider_group_guid = "00000000-0000-0000-0000-000000000003",
        )]
        trait Events
        {
            fn foo();
        }
    }
    expected_errors: [
        "The 'provider_group_guid' attribute key cannot be specified more than once.",
    ]
}

test_case! {
    #[test]
    fn event_with_keyword();
    input: {
        #[trace_logging_provider(guid = "00000000-0000-0000-0000-000000000001")]
        trait Events
        {
            #[event(keyword = 100)]
            fn foo();
        }
    }
    expected_errors: []
}

test_case! {
    #[test]
    fn error_multiple_keyword();
    input: {
        #[trace_logging_provider(guid = "00000000-0000-0000-0000-000000000001")]
        trait Events
        {
            #[event(keyword = 100)]
            fn foo();
        }
    }
    expected_errors: []
}

use quote::quote;

fn test_provider_attributes_error(input: TokenStream, expected_errors: &[&str]) {
    match syn::parse2::<ProviderAttributes>(input) {
        Ok(parsed) => {
            panic!("Expected parsing of input to fail.  Output: {:?}", parsed);
        }
        Err(combined_error) => {
            check_errors(&combined_error, expected_errors);
        }
    }
}

#[test]
fn provider_attributes_invalid_meta() {
    // We do not check the error details for this, because they are not under our control.
    // This is a failure to parse the comma-separated syn::Meta list.
    let result = syn::parse2::<ProviderAttributes>(quote! { bad bad bad });
    assert!(result.is_err());
}

#[test]
fn provider_attributes_unrecognized_key() {
    test_provider_attributes_error(
        quote!(bad_name = "bad_value"),
        &["Unrecognized attribute key."],
    );
}

#[test]
fn provider_attributes_nil_guid() {
    test_provider_attributes_error(
        quote!(guid = "00000000-0000-0000-0000-000000000000"),
        &["The GUID cannot be the NIL (all-zeroes) GUID."],
    );
}

#[test]
fn provider_attributes_invalid_guid() {
    test_provider_attributes_error(
        quote!(guid = "xxx"),
        &["The attribute value is required to be a valid GUID."],
    );
}

#[test]
fn provider_attributes_dup_guid() {
    test_provider_attributes_error(
        quote!(
            guid = "610259b8-9270-46f2-ad94-2f805721b287",
            guid = "610259b8-9270-46f2-ad94-2f805721b287"
        ),
        &["The 'guid' attribute key cannot be specified more than once."],
    );
}

#[test]
fn provider_attributes_valid() {
    let result = syn::parse2::<ProviderAttributes>(quote! {
        guid = "610259b8-9270-46f2-ad94-2f805721b287"
    });
    assert!(result.is_ok(), "Result: {:?}", result);
}

#[test]
fn provider_attributes_valid_static() {
    let result = syn::parse2::<ProviderAttributes>(quote! {
        guid = "610259b8-9270-46f2-ad94-2f805721b287", static_mode
    });
    assert!(result.is_ok(), "Result: {:?}", result);
}

fn check_errors(error: &Error, expected_errors: &[&str]) {
    let mut error_strings: Vec<String> = error.into_iter().map(|e| format!("{}", e)).collect();

    // Go through all the expected errors and ensure we find them in the output.
    for expected_error in expected_errors.iter() {
        let position = error_strings
            .iter()
            .position(|e| e.contains(expected_error));
        if let Some(idx) = position {
            error_strings.remove(idx);
            // good
        } else {
            eprintln!("\nDid not find this error in list: {:?}", expected_error);
            eprintln!("Actual errors:");
            for e in error_strings.iter() {
                eprintln!("    {:?}", e);
            }
            panic!("Error strings did not match.");
        }
    }

    for error_string in error_strings.iter() {
        panic!("Unexpected error: {}", error_string);
    }
}

#[test]
fn test_etw_name_based_guid() {
    // the expected values below were generated by passing the provider name to
    // `[System.Diagnostics.Tracing.EventSource]::new("YourProviderName").Guid`
    use uuid::uuid;
    assert_eq!(
        uuid!("5fefebda-b28e-5a81-d371-cebf3d3ddb41"),
        etw_event_source_guid("YourProviderName")
    );
    assert_eq!(
        uuid!("6f8eac67-f87f-598a-71a0-67e48d8c468d"),
        etw_event_source_guid("YourProviderNameYourProviderName")
    );
    assert_eq!(
        uuid!("5b1bddda-c110-53c9-b030-17aea3bfabd9"),
        etw_event_source_guid("YourProviderNameYourProviderNameYourProviderNameYourProviderName")
    );
    assert_eq!(
        uuid!("0cec4c9d-caa7-5d85-b3bd-c73577f03fd8"),
        etw_event_source_guid("Your.Provider.Name")
    );
    assert_eq!(
        uuid!("32805bf5-e0cc-522a-5f56-20c3e359ed93"),
        etw_event_source_guid("Your.Provider.Name.Your.Provider.Name")
    );
    assert_eq!(
        uuid!("ce5fa4ea-ab00-5402-8b76-9f76ac858fb5"),
        etw_event_source_guid("MyCompany.MyComponent")
    );
    assert_eq!(
        uuid!("0d31f5cc-fb84-50db-a602-8c7bed9c5b8b"),
        etw_event_source_guid("ProviderWithAutogeneratedGuid")
    );
}
