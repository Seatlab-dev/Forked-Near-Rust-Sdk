use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::require;
use near_sdk::schemars::JsonSchema;
use near_sdk::serde::{Deserialize, Serialize};

/// This spec can be treated like a version of the standard.
pub const NFT_METADATA_SPEC: &str = "nft-1.0.0";

/// Metadata for the NFT contract itself.
///
/// See [NEP-177](https://nomicon.io/Standards/Tokens/NonFungibleToken/Metadata) for
/// more info.
#[derive(
    BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema,
)]
#[serde(crate = "near_sdk::serde")]
#[schemars(crate = "near_sdk::schemars")]
#[schemars(example = "nft_contract_metadata_example")]
pub struct NFTContractMetadata {
    /// The Metadata spec version.
    ///
    /// This will allow consumers of the Non-Fungible Token to know which set of
    /// metadata features the contract supports.
    ///
    /// Eg. "nft-1.0.0".
    #[schemars(regex(pattern = r"^nft-\d+\.\d+\.\d+$"))]
    pub spec: String,

    /// The human-readable name of the contract.
    ///
    /// Eg. "Mochi Rising — Digital Edition", "Metaverse 3".
    pub name: String,

    /// The abbreviated symbol of the contract.
    ///
    /// Eg. "MOCHI", "MV3".
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

    /// Centralized gateway known to have reliable access to decentralized storage
    /// assets referenced by `reference` or `media` URLs.
    ///
    /// Can be used by other frontends for initial retrieval of assets, even if
    /// these frontends then replicate the data to their own decentralized nodes,
    /// which they are encouraged to do.
    pub base_uri: Option<String>,

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
}

pub fn nft_contract_metadata_example() -> NFTContractMetadata {
    NFTContractMetadata {
        spec: NFT_METADATA_SPEC.into(),
        name: "Mochi Rising — Digital Edition".into(),
        symbol: "MOCHI".into(),
        icon: Some("data%3Aimage%2Fsvg%2Bxml%2C%3Csvg%20xmlns%3D%27http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%27%3E%3Crect%20width%3D%2750%27%20height%3D%2750%27%20%2F%3E%3C%2Fsvg%3E".into()),
        base_uri: None,
        reference: Some("https://example.com/token.json".into()),
        // from the hex:
        // 00112233445566778899AABBCCDDEEFF0112233445566778899AABBCCDDEEFF0
        reference_hash: Some(b"ABEiM0RVZneImaq7zN3u/wESIzRFVmd4iZqrvM3e7/A=".to_vec().into()),
    }
}

/// Metadata on the individual token level.
///
/// Based on the standard Non-Fungible Token Metadata.
///
/// See [NEP-177](https://nomicon.io/Standards/Tokens/NonFungibleToken/Metadata) for
/// more info.
#[cfg_attr(not(target_arch = "wasm32"), derive())]
#[derive(
    Clone, Debug, PartialEq, Serialize, Deserialize, BorshDeserialize, BorshSerialize, JsonSchema,
)]
#[serde(crate = "near_sdk::serde")]
#[serde(deny_unknown_fields)]
#[schemars(crate = "near_sdk::schemars")]
#[schemars(example = "token_metadata_example")]
pub struct TokenMetadata {
    /// The name of this specific token.
    ///
    /// Eg. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub title: Option<String>,

    /// Free-form description of the token.
    pub description: Option<String>,

    /// URL to associated media.
    ///
    /// Preferably to decentralized, content-addressed storage.
    pub media: Option<String>,

    /// Base64-encoded sha256 hash of content referenced by the `media` field.
    ///
    /// This is to guard against off-chain tampering.
    ///
    /// Required if `media` is included.
    pub media_hash: Option<Base64VecU8>,

    /// The number of tokens with this set of metadata or `media` known to exist at
    /// time of minting.
    pub copies: Option<u64>,

    /// Unix epoch in milliseconds when token was issued or minted (an unsigned
    /// 32-bit integer would suffice until the year 2106).
    pub issued_at: Option<String>,

    /// Unix epoch in milliseconds when token expires.
    pub expires_at: Option<String>,

    /// Unix epoch in milliseconds when token starts being valid.
    pub starts_at: Option<String>,

    /// Unix epoch in milliseconds when token was last updated.
    pub updated_at: Option<String>,

    /// Anything extra the NFT wants to store on-chain. Can be stringified JSON.
    pub extra: Option<String>,

    /// URL to an off-chain JSON file with more info.
    pub reference: Option<String>,

    /// Base64-encoded sha256 hash of JSON from `reference` field. Required if reference is included.
    pub reference_hash: Option<Base64VecU8>,
}

pub fn token_metadata_example() -> TokenMetadata {
    TokenMetadata {
        title: Some("Arch Nemesis: Mail Carrier".into()),
        description: Some("My free-form description".into()),
        media: Some("https://example.com/token/media.xyz".into()),
        // from the hex:
        // 00112233445566778899AABBCCDDEEFF0112233445566778899AABBCCDDEEFF0
        media_hash: Some(b"ABEiM0RVZneImaq7zN3u/wESIzRFVmd4iZqrvM3e7/A=".to_vec().into()),
        copies: Some(1),
        issued_at: Some("1640995200000".into()),
        expires_at: Some("1640995200000".into()),
        starts_at: Some("1640995200000".into()),
        updated_at: Some("1640995200000".into()),
        extra: Some(r#"{"my_key_a": false, "my_key_b": {"inner": [1, 2, 3]}}"#.into()),
        reference: Some("https://example.com/token.json".into()),
        // from the hex:
        // 00112233445566778899AABBCCDDEEFF0112233445566778899AABBCCDDEEFF0
        reference_hash: Some(b"ABEiM0RVZneImaq7zN3u/wESIzRFVmd4iZqrvM3e7/A=".to_vec().into()),
    }
}

/// Offers details on the contract-level metadata.
pub trait NonFungibleTokenMetadataProvider {
    fn nft_metadata(&self) -> NFTContractMetadata;
}

impl NFTContractMetadata {
    pub fn assert_valid(&self) {
        require!(self.spec == NFT_METADATA_SPEC, "Spec is not NFT metadata");
        require!(
            self.reference.is_some() == self.reference_hash.is_some(),
            "Reference and reference hash must be present"
        );
        if let Some(reference_hash) = &self.reference_hash {
            require!(reference_hash.0.len() == 32, "Hash has to be 32 bytes");
        }
    }
}

impl TokenMetadata {
    pub fn assert_valid(&self) {
        require!(self.media.is_some() == self.media_hash.is_some());
        if let Some(media_hash) = &self.media_hash {
            require!(media_hash.0.len() == 32, "Media hash has to be 32 bytes");
        }

        require!(self.reference.is_some() == self.reference_hash.is_some());
        if let Some(reference_hash) = &self.reference_hash {
            require!(reference_hash.0.len() == 32, "Reference hash has to be 32 bytes");
        }
    }
}
