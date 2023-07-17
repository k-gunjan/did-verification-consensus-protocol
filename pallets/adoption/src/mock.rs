use crate as pallet_adoption;
use frame_support::{
	parameter_types,
	traits::{ConstU128, ConstU16, ConstU32, ConstU64},
};
use frame_system as system;
use pallet_balances::AccountData;
use sp_core::{sr25519, Pair, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

/// Balance of an account.
pub type Balance = u128;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		AdoptionModule: pallet_adoption,
	}
);

impl frame_system::Config for Test {
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
	type AccountId = sr25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	///max length of id in adoption pallet
	pub const MaxLength:u32 = 10;
	///min length of id in adoption pallet
	pub const MinLength:u32 = 5;
	///max and min (32 for v0) lenght of CID
	pub const MinCIDLength:u32 = 32;
	//most likely max as lenght depends on hashing algo
	pub const MaxCIDLength:u32 = 100;
}

/// Configure the pallet-adoption in pallets/adoption.
impl pallet_adoption::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxLength = MaxLength;
	type MinLength = MinLength;
	type MinCIDLength = MinCIDLength;
	type MaxCIDLength = MaxCIDLength;
	type Currency = Balances;
	type WeightInfo = pallet_adoption::weights::SubstrateWeight<Test>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<500>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn mock_account(ss58: &str) -> sr25519::Public {
	let (pair, _) = sr25519::Pair::from_string_with_seed(ss58, None).unwrap();
	pair.public()
}
