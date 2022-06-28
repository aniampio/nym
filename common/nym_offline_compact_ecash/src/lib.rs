use std::convert::TryInto;

use bls12_381::Scalar;

pub use scheme::aggregation::aggregate_verification_keys;
pub use scheme::aggregation::aggregate_wallets;
pub use scheme::keygen::generate_keypair_user;
pub use scheme::keygen::ttp_keygen;
pub use scheme::keygen::VerificationKeyAuth;
pub use scheme::setup;
pub use scheme::withdrawal::issue_verify;
pub use scheme::withdrawal::issue_wallet;
pub use scheme::withdrawal::withdrawal_request;
pub use scheme::PartialWallet;
pub use scheme::PayInfo;
pub use traits::Base58;

use crate::error::CompactEcashError;
use crate::traits::Bytable;

mod constants;
mod error;
mod proofs;
mod scheme;
#[cfg(test)]
mod tests;
mod traits;
mod utils;

pub type Attribute = Scalar;

impl Bytable for Attribute {
    fn to_byte_vec(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }

    fn try_from_byte_slice(slice: &[u8]) -> Result<Self, CompactEcashError> {
        Ok(Attribute::from_bytes(slice.try_into().unwrap()).unwrap())
    }
}