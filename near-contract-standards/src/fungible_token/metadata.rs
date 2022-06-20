use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::require;
use near_sdk::schemars::JsonSchema;
use near_sdk::serde::{Deserialize, Serialize};

pub const FT_METADATA_SPEC: &str = "ft-1.0.0";

/// Fungible Token Metadata.
///
/// See [NEP-148](https://nomicon.io/Standards/Tokens/FungibleToken/Metadata) for
/// more info.
#[derive(BorshDeserialize, BorshSerialize, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
#[schemars(crate = "near_sdk::schemars")]
pub struct FungibleTokenMetadata {
    /// Spec style and version.
    ///
    /// The standard validation for this structure requires the value "ft-1.0.0".
    pub spec: String,
    /// The human-readable name of the token.
    pub name: String,
    /// The abbreviation, like "wETH" or "AMPL".
    pub symbol: String,
    /// A small image associated with this token.
    ///
    /// Must be a data URL, to help consumers display it quickly while protecting
    /// user data. For a "square" symbol, one can use the URL-escaped version of
    /// `data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg'><rect width='50' height='50' /></svg>`.
    pub icon: Option<String>,
    /// A link to a valid JSON file containing various keys offering supplementary details on the token.  
    ///
    /// Eg:
    /// - `/ipfs/QmdmQXB2mzChmMeKY47C43LxUdg1NDJ5MWcKMKxDu7RgQm`.
    /// - `https://example.com/token.json`.
    ///
    /// If the information given in this document conflicts with the on-chain
    /// attributes, the values in reference shall be considered the source of truth.
    pub reference: Option<String>,
    /// The base64-encoded sha256 hash of the JSON file contained in the reference
    /// field. This is to guard against off-chain tampering.
    pub reference_hash: Option<Base64VecU8>,
    /// The amount of decimals used by the token unit. Used in frontends to show the
    /// proper significant digits of a token. This concept is explained well in this
    /// [OpenZeppelin post](https://docs.openzeppelin.com/contracts/3.x/erc20#a-note-on-decimals).
    pub decimals: u8,
}

pub trait FungibleTokenMetadataProvider {
    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

impl FungibleTokenMetadata {
    pub fn assert_valid(&self) {
        require!(self.spec == FT_METADATA_SPEC);
        require!(self.reference.is_some() == self.reference_hash.is_some());
        if let Some(reference_hash) = &self.reference_hash {
            require!(reference_hash.0.len() == 32, "Hash has to be 32 bytes");
        }
    }
}
