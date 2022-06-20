use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::schemars::JsonSchema;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;

/// The balance status.
///
/// See [NEP-145](https://nomicon.io/Standards/StorageManagement) for more info.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
#[schemars(crate = "near_sdk::schemars")]
pub struct StorageBalance {
    /// The total amount of yoctoNEAR that the user has deposited for him.
    pub total: U128,
    /// The amount of yoctoNEAR that the user has deposited but is not being used by
    /// the contract, and which the user is free to withdraw.
    pub available: U128,
}

/// The minimum and maximum balance amounts that the contract may require from the
/// user.
///
/// See [NEP-145](https://nomicon.io/Standards/StorageManagement) for more info.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
#[schemars(crate = "near_sdk::schemars")]
pub struct StorageBalanceBounds {
    /// The amount of yoctoNEAR that is required by some functionality, such as for
    /// registering a user on a contract.
    ///
    /// If a new user attaches `min` NEAR to a `storage_deposit` call, subsequent
    /// calls to `storage_balance_of` for this user must show their `total` equal to
    /// `min`, and `available=0`.
    pub min: U128,
    /// The maximum amount of yoctoNEAR that the contract may require from the user.
    ///
    /// - If `null`, then there's no specific maximum balance amount that the
    /// contract may require from the user.  
    /// - If `max` enquals `min`, then the contract only charges for initial
    /// registration, and does not adjust per-user storage over time.  
    /// - Otherwise for some `max` amount, if the user has tried to deposit some
    /// amount higher than `max`, then the contract refunds that extra amount back
    /// to the user.
    pub max: Option<U128>,
}

pub trait StorageManagement {
    // if `registration_only=true` MUST refund above the minimum balance if the account didn't exist and
    //     refund full deposit if the account exists.
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;

    /// Withdraw specified amount of available Ⓝ for predecessor account.
    ///
    /// This method is safe to call. It MUST NOT remove data.
    ///
    /// `amount` is sent as a string representing an unsigned 128-bit integer. If
    /// omitted, contract MUST refund full `available` balance. If `amount` exceeds
    /// predecessor account's available balance, contract MUST panic.
    ///
    /// If predecessor account not registered, contract MUST panic.
    ///
    /// MUST require exactly 1 yoctoNEAR attached balance to prevent restricted
    /// function-call access-key call (UX wallet security)
    ///
    /// Returns the StorageBalance structure showing updated balances.
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance;

    /// Unregisters the predecessor account and returns the storage NEAR deposit back.
    ///
    /// If the predecessor account is not registered, the function MUST return `false` without panic.
    ///
    /// If `force=true` the function SHOULD ignore account balances (burn them) and close the account.
    /// Otherwise, MUST panic if caller has a positive registered balance (eg token holdings) or
    ///     the contract doesn't support force unregistration.
    /// MUST require exactly 1 yoctoNEAR attached balance to prevent restricted function-call access-key call
    /// (UX wallet security)
    /// Returns `true` iff the account was unregistered.
    /// Returns `false` iff account was not registered before.
    fn storage_unregister(&mut self, force: Option<bool>) -> bool;

    fn storage_balance_bounds(&self) -> StorageBalanceBounds;

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance>;
}
