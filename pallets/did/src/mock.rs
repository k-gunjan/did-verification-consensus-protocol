use crate as pallet_did;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// /// The AccountId alias in this test module.
// pub(crate) type AccountId = u64;
// pub(crate) type AccountIndex = u64;
// pub(crate) type BlockNumber = u64;
// pub(crate) type Balance = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		DidModule: pallet_did,
		Timestamp: pallet_timestamp,
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_did::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Time = Timestamp;
}

parameter_types! {
	pub const MinimumPeriod1: u64 = 5;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod1;
	type WeightInfo = (); //pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()

	// let _ = pallet_balances::GenesisConfig::<Test> {
	// 	balances: vec![
	// 		(1, 10 * self.balance_factor),
	// 		(2, 20 * self.balance_factor),
	// 		(3, 300 * self.balance_factor),
	// 		(4, 400 * self.balance_factor),
	// 		// controllers
	// 		(10, self.balance_factor),
	// 		(20, self.balance_factor),
	// 		(30, self.balance_factor),
	// 		(40, self.balance_factor),
	// 		(50, self.balance_factor),
	// 		// stashes
	// 		(11, self.balance_factor * 1000),
	// 		(21, self.balance_factor * 2000),
	// 		(31, self.balance_factor * 2000),
	// 		(41, self.balance_factor * 2000),
	// 		(51, self.balance_factor * 2000),
	// 		// optional nominator
	// 		(100, self.balance_factor * 2000),
	// 		(101, self.balance_factor * 2000),
	// 		// aux accounts
	// 		(60, self.balance_factor),
	// 		(61, self.balance_factor * 2000),
	// 		(70, self.balance_factor),
	// 		(71, self.balance_factor * 2000),
	// 		(80, self.balance_factor),
	// 		(81, self.balance_factor * 2000),
	// 		// This allows us to have a total_payout different from 0.
	// 		(999, 1_000_000_000_000),
	// 	],
	// }
	// .assimilate_storage(&mut storage);
}

// pub fn account_key(account_id: u64) -> AccountId {
// 	AccountId::from(account_id)
// }

// pub fn account_key_from_string(account_id: &str) -> AccountId {
// 	AccountId::from(account_id)
// }
