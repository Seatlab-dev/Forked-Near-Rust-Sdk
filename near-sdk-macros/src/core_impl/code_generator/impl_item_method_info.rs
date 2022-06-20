use crate::core_impl::info_extractor::{
    AttrSigInfo, ImplItemMethodInfo, InputStructType, MethodType, SerializerType,
};
use crate::core_impl::utils;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{ReturnType, Signature};

impl ImplItemMethodInfo {
    /// Generate wrapper method for the given method of the contract.
    pub fn method_wrapper(&self) -> TokenStream2 {
        let ImplItemMethodInfo { attr_signature_info, struct_type, .. } = self;
        // Args provided by `env::input()`.
        let has_input_args = attr_signature_info.input_args().next().is_some();

        let panic_hook = quote! {
            near_sdk::env::setup_panic_hook();
        };

        let input_struct = attr_signature_info.input_struct(InputStructType::Deserialization);

        let input_struct2 =
            attr_signature_info.input_struct2(&self.struct_type, &attr_signature_info.method_type);

        let arg_parsing = if has_input_args {
            let decomposition = attr_signature_info.decomposition_pattern();
            let serializer_invocation = match attr_signature_info.input_serializer {
                SerializerType::JSON => quote! {
                    near_sdk::serde_json::from_slice(
                        &near_sdk::env::input().expect("Expected input since method has arguments.")
                    ).expect("Failed to deserialize input from JSON.")
                },
                SerializerType::Borsh => quote! {
                    near_sdk::borsh::BorshDeserialize::try_from_slice(
                        &near_sdk::env::input().expect("Expected input since method has arguments.")
                    ).expect("Failed to deserialize input from Borsh.")
                },
            };
            quote! {
                let #decomposition : Input = #serializer_invocation ;
            }
        } else {
            TokenStream2::new()
        };

        let callback_deser = attr_signature_info.callback_deserialization();
        let callback_vec_deser = attr_signature_info.callback_vec_deserialization();

        let arg_list = attr_signature_info.arg_list();
        let AttrSigInfo {
            non_bindgen_attrs,
            ident,
            receiver,
            returns,
            result_serializer,
            method_type,
            is_payable,
            is_private,
            is_handles_result,
            ..
        } = attr_signature_info;

        let no_args = attr_signature_info.args.is_empty();
        let no_return = matches!(&self.attr_signature_info.returns, ReturnType::Default);
        let output_type = match &self.attr_signature_info.returns {
            ReturnType::Default => quote!(()),
            ReturnType::Type(_token, type_) => {
                if matches!(method_type, &MethodType::Init) {
                    quote!(())
                } else {
                    quote!(#type_)
                }
            }
        };
        let ident_str = ident.to_string();
        let near_method = match attr_signature_info.method_type {
            MethodType::Regular => quote!(near_sdk::utils::openapi::NearMethod::Regular),
            MethodType::View => quote!(near_sdk::utils::openapi::NearMethod::View),
            MethodType::Init => quote!(near_sdk::utils::openapi::NearMethod::Init),
            MethodType::InitIgnoreState => {
                quote!(near_sdk::utils::openapi::NearMethod::InitIgnoreState)
            }
        };

        let attr_to_string = |attr: &syn::Attribute| {
            let meta = attr.parse_meta().ok()?;
            if let syn::Meta::NameValue(syn::MetaNameValue { lit: syn::Lit::Str(s), .. }) = meta {
                return Some(s.value());
            }

            None
        };

        let doc_attrs = attr_signature_info
            .doc_attrs
            .iter()
            .filter_map(attr_to_string)
            .map(|s| s + "\n")
            .collect::<String>();

        let mut doc_args_attr = String::new();
        let mut has_any_parameter_doc = false;
        for arg in &self.attr_signature_info.args {
            let attrs = &arg.doc_attrs;
            if !attrs.is_empty() {
                has_any_parameter_doc = true;
            }

            let arg_attr =
                attrs.iter().filter_map(attr_to_string).map(|s| s + "\n").collect::<String>();
            doc_args_attr = doc_args_attr
                + &if arg_attr.is_empty() {
                    format!("- `{header}`", header = arg.ident)
                } else {
                    format!("- `{header}` - {arg_attr}", header = arg.ident, arg_attr = arg_attr)
                };
        }
        let doc_args_attr = if has_any_parameter_doc {
            format!("\n\n#### Parameters\n\n{}", doc_args_attr)
        } else {
            "".into()
        };

        let mut properties = vec![];
        match method_type {
            crate::core_impl::MethodType::Regular => {}
            crate::core_impl::MethodType::View => {}
            crate::core_impl::MethodType::Init => {
                properties.push(("init".to_string(), "✓".to_string()));
            }
            crate::core_impl::MethodType::InitIgnoreState => {
                properties.push(("init".to_string(), "✓ (ignore state)".to_string()));
            }
        };
        if !matches!(method_type, crate::core_impl::MethodType::View) {
            properties.push((
                "payable".to_string(),
                if *is_payable { "✓".to_string() } else { "✕".to_string() },
            ));
        }

        if *is_private {
            properties.push(("private".to_string(), "✓".to_string()));
        }

        let properties = if properties.is_empty() {
            String::new()
        } else {
            format!(
                "\n\n#### Properties\n\n| | |\n| -: | :- |\n{}",
                properties
                    .into_iter()
                    .map(|(k, v)| format!("| {} | {} |\n", k, v))
                    .collect::<String>()
            )
        };

        let doc_attrs = format!("{}{}{}", doc_attrs, properties, doc_args_attr);
        let response_description = format!("{}", &output_type);

        let method = quote! {
            pub const NAME: &'static str = #ident_str;
            pub const NEAR_METHOD: near_sdk::utils::openapi::NearMethod = #near_method;
            const DESCRIPTION: &'static str = #doc_attrs;
            const RESPONSE_DESCRIPTION: &'static str = #response_description;
            const NO_ARGS: bool = #no_args;
            const NO_RETURN: bool = #no_return;
            pub type Output = #output_type;
            impl near_sdk::utils::Method for Input {
                const NAME: &'static str = NAME;
                const NEAR_METHOD: near_sdk::utils::openapi::NearMethod = NEAR_METHOD;
                const DESCRIPTION: &'static str = DESCRIPTION;
                const RESPONSE_DESCRIPTION: &'static str = RESPONSE_DESCRIPTION;
                const NO_ARGS: bool = NO_ARGS;
                const NO_RETURN: bool = NO_RETURN;
                type Input = Self;
                type Output = Output;
            }
        };

        let deposit_check = if *is_payable || matches!(method_type, &MethodType::View) {
            // No check if the method is payable or a view method
            quote! {}
        } else {
            // If method is not payable, do a check to make sure that it doesn't consume deposit
            let error = format!("Method {} doesn't accept deposit", ident);
            quote! {
                if near_sdk::env::attached_deposit() != 0 {
                    near_sdk::env::panic_str(#error);
                }
            }
        };
        let is_private_check = if *is_private {
            let error = format!("Method {} is private", ident);
            quote! {
                if near_sdk::env::current_account_id() != near_sdk::env::predecessor_account_id() {
                    near_sdk::env::panic_str(#error);
                }
            }
        } else {
            quote! {}
        };
        let body = if matches!(method_type, &MethodType::Init) {
            match init_method_wrapper(self, true) {
                Ok(wrapper) => wrapper,
                Err(err) => return err.to_compile_error(),
            }
        } else if matches!(method_type, &MethodType::InitIgnoreState) {
            match init_method_wrapper(self, false) {
                Ok(wrapper) => wrapper,
                Err(err) => return err.to_compile_error(),
            }
        } else {
            let contract_deser;
            let method_invocation;
            let contract_ser;
            if let Some(receiver) = receiver {
                let mutability = &receiver.mutability;
                contract_deser = quote! {
                    let #mutability contract: #struct_type = near_sdk::env::state_read().unwrap_or_default();
                };
                method_invocation = quote! {
                    contract.#ident(#arg_list)
                };
                if matches!(method_type, &MethodType::Regular) {
                    contract_ser = quote! {
                        near_sdk::env::state_write(&contract);
                    };
                } else {
                    contract_ser = TokenStream2::new();
                }
            } else {
                contract_deser = TokenStream2::new();
                method_invocation = quote! {
                    #struct_type::#ident(#arg_list)
                };
                contract_ser = TokenStream2::new();
            }
            match returns {
                ReturnType::Default => quote! {
                    #contract_deser
                    #method_invocation;
                    #contract_ser
                },
                ReturnType::Type(_, return_type)
                    if utils::type_is_result(return_type) && *is_handles_result =>
                {
                    let value_ser = match result_serializer {
                        SerializerType::JSON => quote! {
                            let result = near_sdk::serde_json::to_vec(&result).expect("Failed to serialize the return value using JSON.");
                        },
                        SerializerType::Borsh => quote! {
                            let result = near_sdk::borsh::BorshSerialize::try_to_vec(&result).expect("Failed to serialize the return value using Borsh.");
                        },
                    };
                    quote! {
                        #contract_deser
                        let result = #method_invocation;
                        match result {
                            Ok(result) => {
                                #value_ser
                                near_sdk::env::value_return(&result);
                                #contract_ser
                            }
                            Err(err) => near_sdk::FunctionError::panic(&err)
                        }
                    }
                }
                ReturnType::Type(_, return_type) if *is_handles_result => {
                    return syn::Error::new(
                        return_type.span(),
                        "Method marked with #[handle_result] should return Result<T, E>.",
                    )
                    .to_compile_error();
                }
                ReturnType::Type(_, return_type) if utils::type_is_result(return_type) => {
                    return syn::Error::new(
                        return_type.span(),
                        "Serializing Result<T, E> has been deprecated. Consider marking your method \
                        with #[handle_result] if the second generic represents a panicable error or \
                        replacing Result with another two type sum enum otherwise. If you really want \
                        to keep the legacy behavior, mark the method with #[handle_result] and make \
                        it return Result<Result<T, E>, near_sdk::Abort>.",
                    )
                    .to_compile_error();
                }
                ReturnType::Type(_, _) => {
                    let value_ser = match result_serializer {
                        SerializerType::JSON => quote! {
                            let result = near_sdk::serde_json::to_vec(&result).expect("Failed to serialize the return value using JSON.");
                        },
                        SerializerType::Borsh => quote! {
                            let result = near_sdk::borsh::BorshSerialize::try_to_vec(&result).expect("Failed to serialize the return value using Borsh.");
                        },
                    };
                    quote! {
                        #contract_deser
                        let result = #method_invocation;
                        #value_ser
                        near_sdk::env::value_return(&result);
                        #contract_ser
                    }
                }
            }
        };
        let non_bindgen_attrs = non_bindgen_attrs.iter().fold(TokenStream2::new(), |acc, value| {
            quote! {
                #acc
                #value
            }
        });
        quote! {
            #non_bindgen_attrs
            #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            pub extern "C" fn #ident() {
                use #ident::Input;

                #panic_hook
                #is_private_check
                #deposit_check
                #arg_parsing
                #callback_deser
                #callback_vec_deser
                #body
            }
            pub mod #ident {
                use super::*;

                #input_struct
                #input_struct2
                #method
            }
        }
    }

    pub fn marshal_method(&self) -> TokenStream2 {
        let ImplItemMethodInfo { attr_signature_info, .. } = self;
        let has_input_args = attr_signature_info.input_args().next().is_some();

        let pat_type_list = attr_signature_info.pat_type_list();
        let serialize_args = if has_input_args {
            match &attr_signature_info.input_serializer {
                SerializerType::Borsh => crate::TraitItemMethodInfo::generate_serialier(
                    attr_signature_info,
                    &attr_signature_info.input_serializer,
                ),
                SerializerType::JSON => json_serialize(attr_signature_info),
            }
        } else {
            quote! {
             let args = vec![];
            }
        };

        let AttrSigInfo {
            non_bindgen_attrs,
            ident,
            // receiver,
            // returns,
            // result_serializer,
            // is_init,
            method_type,
            original_sig,
            ..
        } = attr_signature_info;
        let return_ident = quote! { -> near_sdk::PendingContractTx };
        let params = quote! {
            &self, #pat_type_list
        };
        let ident_str = ident.to_string();
        let is_view = if matches!(method_type, MethodType::View) {
            quote! {true}
        } else {
            quote! {false}
        };

        let non_bindgen_attrs = non_bindgen_attrs.iter().fold(TokenStream2::new(), |acc, value| {
            quote! {
                #acc
                #value
            }
        });
        let Signature { generics, .. } = original_sig;
        quote! {
            #[cfg(not(target_arch = "wasm32"))]
            #non_bindgen_attrs
            pub fn #ident#generics(#params) #return_ident {
                #serialize_args
                near_sdk::PendingContractTx::new_from_bytes(self.account_id.clone(), #ident_str, args, #is_view)
            }
        }
    }
}

fn init_method_wrapper(
    method_info: &ImplItemMethodInfo,
    check_state: bool,
) -> Result<TokenStream2, syn::Error> {
    let ImplItemMethodInfo { attr_signature_info, struct_type, .. } = method_info;
    let arg_list = attr_signature_info.arg_list();
    let AttrSigInfo { ident, returns, is_handles_result, .. } = attr_signature_info;
    let state_check = if check_state {
        quote! {
            if near_sdk::env::state_exists() {
                near_sdk::env::panic_str("The contract has already been initialized");
            }
        }
    } else {
        quote! {}
    };
    match returns {
        ReturnType::Default => {
            Err(syn::Error::new(ident.span(), "Init methods must return the contract state"))
        }
        ReturnType::Type(_, return_type)
            if utils::type_is_result(return_type) && *is_handles_result =>
        {
            Ok(quote! {
                #state_check
                let result = #struct_type::#ident(#arg_list);
                match result {
                    Ok(contract) => near_sdk::env::state_write(&contract),
                    Err(err) => near_sdk::FunctionError::panic(&err)
                }
            })
        }
        ReturnType::Type(_, return_type) if *is_handles_result => Err(syn::Error::new(
            return_type.span(),
            "Method marked with #[handle_result] should return Result<T, E>",
        )),
        ReturnType::Type(_, _) => Ok(quote! {
            #state_check
            let contract = #struct_type::#ident(#arg_list);
            near_sdk::env::state_write(&contract);
        }),
    }
}

fn json_serialize(attr_signature_info: &AttrSigInfo) -> TokenStream2 {
    let args: TokenStream2 = attr_signature_info
        .input_args()
        .fold(None, |acc: Option<TokenStream2>, value| {
            let ident = &value.ident;
            let ident_str = ident.to_string();
            Some(match acc {
                None => quote! { #ident_str: #ident },
                Some(a) => quote! { #a, #ident_str: #ident },
            })
        })
        .unwrap();
    quote! {
      let args = near_sdk::serde_json::json!({#args}).to_string().into_bytes();
    }
}
