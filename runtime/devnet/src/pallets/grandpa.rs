#![allow(unused_imports)]

use frame_support::traits::KeyOwnerProofSystem;
use sp_core::crypto::KeyTypeId;

use crate::*;

pub use pallet_grandpa::AuthorityId as GrandpaId;

parameter_types! {
	pub const MaxSetIdSessionEntries: u32 = BondingDuration::get() * SessionsPerEra::get();
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type KeyOwnerProofSystem = Historical;

	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		GrandpaId,
	)>>::IdentificationTuple;

	type HandleEquivocation = pallet_grandpa::EquivocationHandler<
		Self::KeyOwnerIdentification,
		Offences,
		ReportLongevity,
	>;

	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
}
