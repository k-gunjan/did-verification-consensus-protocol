use crate as pallet_verification_protocol;
use frame_support::{
	parameter_types,
	traits::{ConstU128, ConstU16, ConstU32, ConstU64},
	PalletId,
};
use frame_system as system;
use pallet_balances as balances;
use pallet_balances::AccountData;
use pallet_verification_protocol::types::IdType;
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
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		DidModule: pallet_did,
		VerificationProtocol: pallet_verification_protocol,
		Verifiers: verifiers,
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

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<100>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

// //parameters of pallet verification protocol & verifiers
parameter_types! {
pub const MaxLengthListOfDocuments: u32 = 150;
pub const MaxEligibleVerifiers: u32 = 1000;
}

parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"00vrfksn");
	pub const TreasuryPalletId2: PalletId = PalletId(*b"0000vrfr");
}
// Configure the  pallet verification protocol
impl pallet_verification_protocol::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type MaxLengthListOfDocuments = MaxLengthListOfDocuments;
	type VerifiersProvider = Verifiers;
	type DidProvider = DidModule;
	type IdDocument = IdType<Self>;
}

// Configure the verifier pallets
impl verifiers::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type PalletId = TreasuryPalletId2;
	type MaxEligibleVerifiers = MaxEligibleVerifiers;
}

// Configure the pallet-did in pallets/did.
impl pallet_did::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Time = Timestamp;
	type WeightInfo = pallet_did::weights::SubstrateWeight<Test>;
}

parameter_types! {
	pub const MinimumPeriod1: u64 = 5;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod1;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();

	balances::GenesisConfig::<Test> {
		balances: vec![
			(account_key("//Alice"), 1000_000_000_000_000),
			(account_key("//Bob"), 1000_000_000_000_000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	t.into()
}

pub fn account_key(s: &str) -> sr25519::Public {
	sr25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}
