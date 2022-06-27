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
#[schemars(example = "fungible_token_metadata_example")]
pub struct FungibleTokenMetadata {
    /// Spec style and version.
    ///
    /// Should be ft-1.0.0 to indicate that a Fungible Token contract adheres to the
    /// current versions of this Metadata and the
    /// [Fungible Token Core](https://nomicon.io/Standards/Tokens/FungibleToken/Core)
    /// specs.
    /// This will allow consumers of the Fungible Token to know if they support the
    /// features of a given contract.
    ///
    /// The standard validation for this structure requires the value "ft-1.0.0".
    pub spec: String,

    /// The human-readable name of the token.
    pub name: String,

    /// The abbreviation, like "wETH" or "AMPL".
    pub symbol: String,

    /// A small image associated with this token.
    ///
    /// Must be a
    /// [data URL](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URLs),
    /// to help consumers display it quickly while protecting user data.
    ///
    /// Recommendation: use optimized SVG, which can result in high-resolution
    /// images with only 100s of bytes of
    /// [storage cost](https://docs.near.org/docs/concepts/storage-staking).
    /// (Note that these storage costs are incurred to the token owner/deployer, but
    /// that querying these icons is a very cheap & cacheable read operation for all
    /// consumers of the contract and the RPC nodes that serve the data.)
    ///
    /// Recommendation: create icons that will work well with both light-mode and
    /// dark-mode websites by either using middle-tone color schemes, or by
    /// [embedding media queries in the SVG](https://timkadlec.com/2013/04/media-queries-within-svg/).
    ///
    /// For a "square" symbol, one can use the URL-escaped version of
    /// `data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg'><rect width='50' height='50' /></svg>`.
    pub icon: Option<String>,

    /// A link to a valid JSON file containing various keys offering supplementary
    /// details on the token.  
    ///
    /// Eg:
    /// - `/ipfs/QmdmQXB2mzChmMeKY47C43LxUdg1NDJ5MWcKMKxDu7RgQm`.
    /// - `https://example.com/token.json`.
    ///
    /// If the information given in this document conflicts with the on-chain
    /// attributes, the values in reference shall be considered the source of truth.
    pub reference: Option<String>,

    /// The base64-encoded sha256 hash of the JSON file contained in the `reference`
    /// field. This is to guard against off-chain tampering.
    pub reference_hash: Option<Base64VecU8>,

    /// The amount of decimals used by the token unit. Used in frontends to show the
    /// proper significant digits of a token.
    ///
    /// This concept is explained well in this
    /// [OpenZeppelin post](https://docs.openzeppelin.com/contracts/3.x/erc20#a-note-on-decimals).
    #[schemars(range(min = 0, max = 39))]
    pub decimals: u8,
}

pub fn fungible_token_metadata_example() -> FungibleTokenMetadata {
    FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.into(),
        name: "FT Example".into(),
        symbol: "xFT".into(),
        icon: Some("data%3Aimage%2Fsvg%2Bxml%2C%3Csvg%20xmlns%3D%27http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%27%3E%3Crect%20width%3D%2750%27%20height%3D%2750%27%20%2F%3E%3C%2Fsvg%3E".into()),
        reference: Some("https://example.com/token.json".into()),
        // from the hex:
        // 00112233445566778899AABBCCDDEEFF0112233445566778899AABBCCDDEEFF0
        reference_hash: Some(b"ABEiM0RVZneImaq7zN3u/wESIzRFVmd4iZqrvM3e7/A=".to_vec().into()),
        decimals: 8,
    }
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
