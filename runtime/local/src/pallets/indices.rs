#![allow(unused_imports)]

use crate::*;

parameter_types! {
	pub const IndexDeposit: Balance = 1 * PAN;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}
