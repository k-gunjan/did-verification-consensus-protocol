use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_std::vec::Vec;
// use frame_support::inherent::Vec;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Attributes or properties that make an DID.
#[derive(
	PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug, TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Attribute<BlockNumber, Moment> {
	pub name: Vec<u8>,
	pub value: Vec<u8>,
	pub validity: BlockNumber,
	pub creation: Moment,
}
