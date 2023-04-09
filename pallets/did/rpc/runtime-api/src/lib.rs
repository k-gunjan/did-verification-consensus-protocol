//! Runtime API definition for did pallet.

#![cfg_attr(not(feature = "std"), no_std)]

// use sp_std::vec::Vec;
use frame_support::inherent::Vec;
use sp_runtime::codec::Codec;
// add Attribute from did pallet
use pallet_did::types::Attribute;

// The `RuntimeApi` trait is used to define the runtime api.
// add runtimeapi for get_attribute of did pallet
sp_api::decl_runtime_apis! {
	pub trait ReadAttributeApi<AccountId, Blocknumber, Moment> where
		AccountId: Codec,
		Blocknumber: Codec,
		Moment: Codec,
	{
		fn read(did: AccountId, name: Vec<u8>) -> Option<Attribute<Blocknumber, Moment>>;
	}
}
