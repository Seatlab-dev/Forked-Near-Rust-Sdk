use proc_macro2::TokenStream as TokenStream2;

use crate::core_impl::info_extractor::{
    ArgInfo, AttrSigInfo, BindgenArgType, InputStructType, PropertyAttr, SerializerType,
};
use crate::core_impl::utils;
use quote::quote;

impl AttrSigInfo {
    /// Create struct representing input arguments.
    /// * input_struct_type represents whether the input structure will be used for serialization
    ///     (e.g. for a promise input) or deserialization (e.g. for a method input).
    /// Each argument is getting converted to a field in a struct. Specifically argument:
    /// `ATTRIBUTES ref mut binding @ SUBPATTERN : TYPE` is getting converted to:
    /// `binding: SUBTYPE,` where `TYPE` is one of the following: `& SUBTYPE`, `&mut SUBTYPE`,
    /// and `SUBTYPE` is one of the following: `[T; n]`, path like
    /// `std::collections::HashMap<SUBTYPE, SUBTYPE>`, or tuple `(SUBTYPE0, SUBTYPE1, ...)`.
    /// # Example
    /// ```ignore
    /// struct Input {
    ///   arg0: Vec<String>,
    ///   arg1: [u64; 10],
    ///   arg2: (u64, Vec<String>),
    /// }
    /// ```
    pub fn input_struct(&self, input_struct_type: InputStructType) -> TokenStream2 {
        let args: Vec<_> = self.input_args().collect();
        // assert!(
        //     !args.is_empty(),
        //     "Can only generate input struct for when input args are specified"
        // );
        let schemars_renamed = format!("{}.Input", self.ident);
        let attribute = match input_struct_type {
            InputStructType::Serialization => match &self.input_serializer {
                SerializerType::JSON => quote! {
                    #[derive(near_sdk::serde::Serialize)]
                    #[serde(crate = "near_sdk::serde")]
                },
                SerializerType::Borsh => {
                    quote! {
                        #[derive(near_sdk::borsh::BorshSerialize)]
                    }
                }
            },
            InputStructType::Deserialization => match &self.input_serializer {
                SerializerType::JSON => quote! {
                    #[derive(
                        near_sdk::serde::Serialize,
                        near_sdk::serde::Deserialize,
                        near_sdk::schemars::JsonSchema
                    )]
                    #[serde(crate = "near_sdk::serde")]
                    #[schemars(crate = "near_sdk::schemars")]
                    #[schemars(rename = #schemars_renamed)]
                },
                SerializerType::Borsh => {
                    quote! {
                        #[derive(
                            near_sdk::borsh::BorshSerialize, near_sdk::borsh::BorshDeserialize
                        )]
                    }
                }
            },
        };
        // pub a: (), pub b: (), ..
        let mut fields = TokenStream2::new();
        for arg in args {
            let ArgInfo { ty, ident, doc_attrs, forward_attrs: schemars_attrs, .. } = &arg;
            fields.extend(quote! {
                #(#doc_attrs)*
                #(#schemars_attrs)*
                pub #ident: #ty,
            });
        }
        let doc_attrs = &self.doc_attrs;
        let schemars_attrs = &self.forward_attrs;
        let mut doc_args_attr = vec![];
        let mut has_any_parameter_doc = false;
        for arg in &self.args {
            let header = format!("- `{}`", arg.ident);
            let attrs = &arg.doc_attrs;
            if !attrs.is_empty() {
                has_any_parameter_doc = true;
            }
            doc_args_attr.push(quote! {
                #[doc = #header]
                #(#attrs)*
            });
        }
        let parameters_doc = if has_any_parameter_doc {
            quote! {
                #[doc = ""]
                #[doc = "#### Parameters"]
                #[doc = ""]
                #(#doc_args_attr)*
            }
        } else {
            quote!()
        };

        let mut properties = vec![];
        match self.method_type {
            crate::core_impl::MethodType::Regular => {}
            crate::core_impl::MethodType::View => {}
            crate::core_impl::MethodType::Init => {
                properties.push(("init".to_string(), "✓".to_string()));
            }
            crate::core_impl::MethodType::InitIgnoreState => {
                properties.push(("init".to_string(), "✓ (ignore state)".to_string()));
            }
        };
        if !matches!(self.method_type, crate::core_impl::MethodType::View) {
            properties.push((
                "payable".to_string(),
                if self.is_payable { "✓".to_string() } else { "✕".to_string() },
            ));
        }

        if self.is_private {
            properties.push(("private".to_string(), "✓".to_string()));
        }

        for property in self.property_attrs.iter().map(PropertyAttr::to_key_value) {
            properties.push(property)
        }

        let properties =
            properties.into_iter().map(|(k, v)| format!("- {}: {}", k, v)).collect::<Vec<_>>();

        let properties = if properties.is_empty() {
            quote!()
        } else {
            quote! {
                #[doc = ""]
                #[doc = "#### Properties"]
                #[doc = ""]
                #(#[doc = #properties])*
            }
        };

        quote! {
            #attribute
            #(#doc_attrs)*
            #(#schemars_attrs)*
            #parameters_doc
            #properties
            pub struct Input {
                #fields
            }
        }
    }

    pub fn input_struct2(
        &self,
        contract_ty: &syn::Type,
        method_type: &crate::MethodType,
    ) -> TokenStream2 {
        let args: Vec<_> = self.input_args().collect();
        let fn_ident = &self.ident;

        let marshall_contract = {
            use quote::{format_ident, ToTokens};
            let orig_name = contract_ty.clone().into_token_stream();
            let mut name = quote! {Contract};
            if let Ok(input) = syn::parse::<syn::Ident>(orig_name.into()) {
                let new_name = format_ident!("{}Contract", input);
                name = quote! {#new_name};
            };
            name
        };

        // a, b, ..
        let mut field_names = TokenStream2::new();
        // a: impl Into<()>, b: impl Into<()>, ..
        let mut impl_into_args = TokenStream2::new();
        // a: a.into(), b: b.into(), ..
        let mut impl_into_fields = TokenStream2::new();
        for arg in args {
            let ArgInfo { ty, ident, .. } = &arg;
            field_names.extend(quote! (#ident,));
            impl_into_args.extend(quote! {
                #ident: impl Into<#ty>,
            });
            impl_into_fields.extend(quote! {
                #ident: #ident.into(),
            });
        }

        let wrap_trait = if matches!(method_type, &crate::MethodType::Init) {
            quote! {
                pub trait WrapInput<Local>: Sized {
                    fn #fn_ident(#impl_into_args) -> near_sdk::utils::InputWrapped<'static, std::marker::PhantomData<Self>, Input> {
                        let input = Input::new(#field_names);
                        near_sdk::utils::InputWrapped {
                            inner: &std::marker::PhantomData,
                            input
                        }
                    }
                }
            }
        } else {
            quote! {
                pub trait WrapInput<Local>: Sized {
                    fn #fn_ident(&self, #impl_into_args) -> near_sdk::utils::InputWrapped<Self, Input> {
                        let input = Input::new(#field_names);
                        near_sdk::utils::InputWrapped {
                            inner: self,
                            input
                        }
                    }
                }
            }
        };

        quote! {
            impl Input {
                pub fn new(#impl_into_args) -> Input {
                    Input {
                        #impl_into_fields
                    }
                }
            }

            pub use WrapInput as #fn_ident;
            #wrap_trait

            #[cfg(not(target_arch = "wasm32"))]
            struct Here;

            #[cfg(not(target_arch = "wasm32"))]
            impl <Local, T> WrapInput<Local> for T where
                T: near_sdk::HasContract<Local, Contract=#marshall_contract>
            {}

            #[cfg(not(target_arch = "wasm32"))]
            impl WrapInput<Here> for #marshall_contract {}
        }
    }

    /// Create pattern that decomposes input struct using correct mutability modifiers.
    /// # Example:
    /// ```ignore
    /// Input {
    ///     arg0,
    ///     mut arg1,
    ///     arg2
    /// }
    /// ```
    pub fn decomposition_pattern(&self) -> TokenStream2 {
        let args: Vec<_> = self.input_args().collect();
        assert!(
            !args.is_empty(),
            "Can only generate decomposition pattern for when input args are specified."
        );
        let mut fields = TokenStream2::new();
        for arg in args {
            let ArgInfo { mutability, ident, .. } = &arg;
            fields.extend(quote! {
            #mutability #ident,
            });
        }
        quote! {
            Input {
                #fields
            }
        }
    }

    /// Create expression that constructs the struct.
    /// # Example:
    /// ```ignore
    /// Input {
    ///     arg0,
    ///     arg1,
    ///     arg2,
    /// }
    /// ```
    pub fn constructor_expr(&self) -> TokenStream2 {
        let args: Vec<_> = self.input_args().collect();
        assert!(
            !args.is_empty(),
            "Can only generate constructor expression for when input args are specified."
        );
        let mut fields = TokenStream2::new();
        for arg in args {
            let ArgInfo { ident, .. } = &arg;
            fields.extend(quote! {
            #ident,
            });
        }
        quote! {
            Input {
                #fields
            }
        }
    }

    /// Create a sequence of arguments that can be used to call the method or the function
    /// of the smart contract.
    ///
    /// # Example:
    /// ```ignore
    /// a, &b, &mut c,
    /// ```
    pub fn arg_list(&self) -> TokenStream2 {
        let mut result = TokenStream2::new();
        for arg in &self.args {
            let ArgInfo { reference, mutability, ident, .. } = &arg;
            result.extend(quote! {
                #reference #mutability #ident,
            });
        }
        result
    }

    /// Create a sequence of patterns and types to be used in the method signature.
    ///
    /// # Example:
    /// ```ignore
    /// a: u64, b: &mut T, ref mut c: Vec<String>,
    /// ```
    pub fn pat_type_list(&self) -> TokenStream2 {
        let mut result = TokenStream2::new();
        for arg in self.input_args() {
            let ArgInfo { original, .. } = &arg;
            result.extend(quote! {
                #original,
            });
        }
        result
    }

    /// Create code that deserializes arguments that were decorated with `#[callback*]`
    pub fn callback_deserialization(&self) -> TokenStream2 {
        self.args
            .iter()
            .filter(|arg| {
                matches!(
                    arg.bindgen_ty,
                    BindgenArgType::CallbackArg | BindgenArgType::CallbackResultArg
                )
            })
            .enumerate()
            .fold(TokenStream2::new(), |acc, (idx, arg)| {
                let idx = idx as u64;
                let ArgInfo { mutability, ident, ty, bindgen_ty, serializer_ty, .. } = arg;
                match &bindgen_ty {
                    BindgenArgType::CallbackArg => {
                        let error_msg = format!("Callback computation {} was not successful", idx);
                        let read_data = quote! {
                            let data: Vec<u8> = match near_sdk::env::promise_result(#idx) {
                                near_sdk::PromiseResult::Successful(x) => x,
                                _ => near_sdk::env::panic_str(#error_msg)
                            };
                        };
                        let invocation = deserialize_data(serializer_ty);
                        quote! {
                            #acc
                            #read_data
                            let #mutability #ident: #ty = #invocation;
                        }
                    }
                    BindgenArgType::CallbackResultArg => {
                        let ok_type = if let Some(ok_type) = utils::extract_ok_type(ty) {
                            ok_type
                        } else {
                            return syn::Error::new_spanned(ty, "Function parameters marked with \
                                #[callback_result] should have type Result<T, PromiseError>").into_compile_error()
                        };
                        let deserialize = deserialize_data(serializer_ty);
                        let deserialization_branch = match ok_type {
                            // The unit type in this context is a bit special because functions
                            // without an explicit return type do not serialize their response.
                            // But when someone tries to refer to their callback result with
                            // `#[callback_result]` they specify the callback type as
                            // `Result<(), PromiseError>` which cannot be correctly deserialized from
                            // an empty byte array.
                            //
                            // So instead of going through serde, we consider deserialization to be
                            // successful if the byte array is empty or try the normal
                            // deserialization otherwise.
                            syn::Type::Tuple(type_tuple) if type_tuple.elems.is_empty() =>
                                quote! {
                                    near_sdk::PromiseResult::Successful(data) if data.is_empty() =>
                                        Ok(()),
                                    near_sdk::PromiseResult::Successful(data) => Ok(#deserialize)
                                },
                            _ =>
                                quote! {
                                    near_sdk::PromiseResult::Successful(data) => Ok(#deserialize)
                                }
                        };
                        let result = quote! {
                            match near_sdk::env::promise_result(#idx) {
                                #deserialization_branch,
                                near_sdk::PromiseResult::NotReady => Err(near_sdk::PromiseError::NotReady),
                                near_sdk::PromiseResult::Failed => Err(near_sdk::PromiseError::Failed),
                            }
                        };
                        quote! {
                            #acc
                            let #mutability #ident: #ty = #result;
                        }
                    }
                    _ => unreachable!()
                }
            })
    }

    /// Create code that deserializes arguments that were decorated with `#[callback_vec]`.
    pub fn callback_vec_deserialization(&self) -> TokenStream2 {
        self
            .args
            .iter()
            .filter(|arg| matches!(arg.bindgen_ty, BindgenArgType::CallbackArgVec))
            .fold(TokenStream2::new(), |acc, arg| {
                let ArgInfo { mutability, ident, ty, .. } = arg;
                let invocation = deserialize_data(&arg.serializer_ty);
                quote! {
                #acc
                let #mutability #ident: #ty = (0..near_sdk::env::promise_results_count())
                .map(|i| {
                    let data: Vec<u8> = match near_sdk::env::promise_result(i) {
                        near_sdk::PromiseResult::Successful(x) => x,
                        _ => near_sdk::env::panic_str(&format!("Callback computation {} was not successful", i)),
                    };
                    #invocation
                }).collect();
            }
            })
    }
}

pub fn deserialize_data(ty: &SerializerType) -> TokenStream2 {
    match ty {
        SerializerType::JSON => quote! {
            near_sdk::serde_json::from_slice(&data).expect("Failed to deserialize callback using JSON")
        },
        SerializerType::Borsh => quote! {
            near_sdk::borsh::BorshDeserialize::try_from_slice(&data).expect("Failed to deserialize callback using Borsh")
        },
    }
}
