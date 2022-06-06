use crate::ImplItemMethodInfo;
use syn::spanned::Spanned;
use syn::{Error, Ident, ImplItem, ItemImpl, Type};

/// Information extracted from `impl` section.
pub struct ItemImplInfo {
    /// Whether this is a trait implementation.
    pub is_trait_impl: bool,
    pub underscore_trait: Ident,
    /// The type for which this `impl` is written.
    pub ty: Type,
    /// Info extracted for each method.
    pub methods: Vec<ImplItemMethodInfo>,
}

impl ItemImplInfo {
    pub fn new(original: &mut ItemImpl) -> syn::Result<Self> {
        if !original.generics.params.is_empty() {
            return Err(Error::new(
                original.generics.params.span(),
                "Impl type parameters are not supported for smart contracts.",
            ));
        }
        let is_trait_impl = original.trait_.is_some();
        let ty = (*original.self_ty.as_ref()).clone();
        let underscore_trait = match &original.trait_ {
            None => syn::Ident::new("_methods", proc_macro2::Span::call_site()),
            Some(x) => {
                let mut res = String::from("methods");
                for ident in x.1.segments.iter().map(|s| &s.ident) {
                    res = ident.to_string() + "_" + &res;
                }
                syn::Ident::new(&res, x.1.segments[0].span())
            }
        };

        let mut methods = vec![];
        for subitem in &mut original.items {
            if let ImplItem::Method(m) = subitem {
                let method_info = ImplItemMethodInfo::new(m, ty.clone())?;
                methods.push(method_info);
            }
        }
        Ok(Self { is_trait_impl, underscore_trait, ty, methods })
    }
}
