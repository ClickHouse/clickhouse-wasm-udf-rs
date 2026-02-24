extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

fn returns_result(output: &syn::ReturnType) -> bool {
    if let syn::ReturnType::Type(_, ty) = output {
        if let syn::Type::Path(type_path) = ty.as_ref() {
            if let Some(last_seg) = type_path.path.segments.last() {
                return last_seg.ident == "Result";
            }
        }
    }
    false
}

#[proc_macro_attribute]
pub fn clickhouse_udf(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_inputs = &input.sig.inputs;
    let fn_body = input.block;
    let fn_output = &input.sig.output;

    let internal_fn_ident = syn::Ident::new(&format!("_chudf_impl_{}", fn_name), fn_name.span());

    let arg_names: Vec<_> = fn_inputs
        .iter()
        .enumerate()
        .map(|(i, _)| syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site()))
        .collect();

    let call_expr = if returns_result(fn_output) {
        quote! {
            let result = match #internal_fn_ident(#(#arg_names),*) {
                Ok(val) => val,
                Err(err) => {
                    ::clickhouse_wasm_udf::host_api::fatal(&format!("{}", err));
                }
            };
        }
    } else {
        quote! {
            let result = #internal_fn_ident(#(#arg_names),*);
        }
    };

    let expanded = quote! {

        fn #internal_fn_ident(#fn_inputs) #fn_output {
            #fn_body
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn #fn_name(data: &::clickhouse_wasm_udf::buffer::RawBuffer, num_rows: usize) -> *const ::clickhouse_wasm_udf::buffer::RawBuffer {
            use ::clickhouse_wasm_udf::buffer::RawBuffer;

            if data.ptr.is_null() || data.len == 0 {
                ::clickhouse_wasm_udf::ch_log!("Input data is empty");
                return std::ptr::null();
            }

            let input_data = unsafe { std::slice::from_raw_parts(data.ptr, data.len) };
            let mut cin = std::io::Cursor::new(input_data);

            let mut serialized = Vec::<u8>::new();
            let mut cout = std::io::Cursor::new(&mut serialized);
            let mut serializer = ::clickhouse_wasm_udf::rmp_serde::encode::Serializer::new(&mut cout);
            let mut result_num_rows = 0;
            while cin.position() != data.len as u64 {
                let mut deser = ::clickhouse_wasm_udf::rmp_serde::decode::Deserializer::new(&mut cin);
                #(
                    let #arg_names = match ::clickhouse_wasm_udf::serde::de::Deserialize::deserialize(&mut deser) {
                        Ok(val) => val,
                        Err(err) => {
                            ::clickhouse_wasm_udf::ch_fatal!("Error deserializing argument: {:?}", err);
                        }
                    };
                )*

                #call_expr

                if let Err(write_err) = ::clickhouse_wasm_udf::serde::ser::Serialize::serialize(&result, &mut serializer) {
                    ::clickhouse_wasm_udf::ch_fatal!("Error serializing result: {:?}", write_err);
                }
                result_num_rows += 1;
            }

            if result_num_rows != num_rows {
                ::clickhouse_wasm_udf::ch_fatal!("Expected {} rows, got {}", num_rows, result_num_rows);
            }

            let out = RawBuffer::from_vec(serialized);
            Box::into_raw(Box::new(out))
        }
    };

    TokenStream::from(expanded)
}
