// #![allow(unused_imports)]
use codec::{Decode, Encode};
use frame_support::inherent::Vec;
use scale_info::TypeInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
/// Attributes or properties that make an DID.

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
