use syn::parenthesized;
use syn::parse::{Parse, ParseStream};

pub struct PropertyAttr {
    #[allow(dead_code)]
    paren_token: syn::token::Paren,
    pub key: syn::ExprLit,
    #[allow(dead_code)]
    comma_token: syn::token::Comma,
    pub value: syn::ExprLit,
}

impl PropertyAttr {
    pub fn to_key_value(&self) -> (String, String) {
        (lit_to_string(&self.key.lit), lit_to_string(&self.value.lit))
    }
}

pub fn lit_to_string(lit: &syn::Lit) -> String {
    match lit {
        syn::Lit::Str(s) => s.value(),
        syn::Lit::ByteStr(bs) => {
            let b64 = base64::encode(&bs.value());
            format!("base64: {}", b64)
        }
        syn::Lit::Byte(b) => {
            format!("byte: {:#04X}", b.value())
        }
        syn::Lit::Char(c) => {
            format!("char: {}", c.value())
        }
        syn::Lit::Int(i) => {
            format!("integer: {}", i.to_string())
        }
        syn::Lit::Float(f) => {
            format!("float: {}", f.to_string())
        }
        syn::Lit::Bool(b) => {
            format!("bool: {}", b.value())
        }
        syn::Lit::Verbatim(other) => {
            format!("verbatim: {}", other.to_string())
        }
    }
}

impl Parse for PropertyAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);
        let key: syn::ExprLit = content.parse()?;
        let comma_token: syn::token::Comma = content.parse()?;
        let value: syn::ExprLit = content.parse()?;

        Ok(Self { paren_token, key, comma_token, value })
    }
}
