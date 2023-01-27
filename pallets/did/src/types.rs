// #![allow(unused_imports)]
use codec::{Decode, Encode};
// use sp_core::RuntimeDebug;
// use sp_std::vec::Vec;
use frame_support::inherent::Vec;
use scale_info::TypeInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
// use sp_runtime::{Serialize, Deserialize};

/// Attributes or properties that make an DID.
// #[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
// #[derive(Serialize, Deserialize)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Attribute<BlockNumber, Moment> {
	pub name: Vec<u8>,
	pub value: Vec<u8>,
	pub validity: BlockNumber,
	pub creation: Moment,
	pub nonce: u64,
}

pub type AttributedId<BlockNumber, Moment> = (Attribute<BlockNumber, Moment>, [u8; 32]);

/// Off-chain signed transaction.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
pub struct AttributeTransaction<Signature, AccountId> {
	pub signature: Signature,
	pub name: Vec<u8>,
	pub value: Vec<u8>,
	pub validity: u32,
	pub signer: AccountId,
	pub identity: AccountId,
}
