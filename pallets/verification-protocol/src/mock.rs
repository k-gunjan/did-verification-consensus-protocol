pub(crate) use crate as pallet_verification_protocol;
pub use crate::mock::{
	sp_api_hidden_includes_construct_runtime::hidden_include::traits::Hooks, sr25519::Public,
};
use frame_support::{
	parameter_types,
	traits::{ConstU128, ConstU16, ConstU64},
	PalletId,
};
use frame_system as system;
use pallet_balances::AccountData;
pub(crate) use pallet_did::DidProvider;
pub(crate) use pallet_verification_protocol::{
	types::{ProtocolParameterValues as VerificationProtocolParameterValues, *},
	*,
};
pub use sp_core::{bounded_vec::BoundedVec, keccak_256, sr25519, Pair, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};
/// Balance of an account.
pub type Balance = u128;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
// pub type MaxLengthListOfDocumentsType =
// 	<Test as pallet_verification_protocol::Config>::MaxLengthListOfDocuments;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
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
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<100>;
	type AccountStore = System;
	type WeightInfo = ();
}

// //parameters of pallet verification protocol & verifiers
parameter_types! {
#[derive(PartialEq, Debug)]
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

pub struct ExtBuilder {
	balances: Vec<(sr25519::Public, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![
				(account_key("//Alice"), 1000_000_000_000_000),
				(account_key("//Bob"), 1000_000_000_000_000),
				(account_key("//Dave"), 1000_000_000_000_000),
				(account_key("//Charlie"), 1000_000_000_000_000),
			],
		}
	}
}

impl ExtBuilder {
	pub fn _balances(mut self, balances: &[(sr25519::Public, Balance)]) -> Self {
		self.balances = balances.to_vec();
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let config = GenesisConfig {
			system: Default::default(),
			balances: BalancesConfig { balances: self.balances },
		};

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn account_key(s: &str) -> sr25519::Public {
	sr25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 0 {
			Verifiers::on_finalize(System::block_number());
			VerificationProtocol::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::reset_events();
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Verifiers::on_initialize(System::block_number());
		VerificationProtocol::on_initialize(System::block_number());
	}
}

pub fn create_verification_request(account: Public, docs: Vec<u8>) -> VerificationRequest<Test> {
	let parameters = VerificationProtocol::protocol_parameters();
	let vr = VerificationRequest {
		consumer_account_id: account.clone(),
		submitted_at: System::block_number(),
		list_of_documents: docs.try_into().unwrap(),
		did_creation_status: DidCreationStatus::default(),
		round_number: 1,
		state: StateConfig {
			allot: StateAttributes {
				done_count_of_verifiers: 0,
				pending_count_of_verifiers: parameters.min_count_at_allot_stage,
				state: true,
				started_at: System::block_number(),
				ended_at: None.into(),
				state_duration: parameters.max_waiting_time_at_stages,
			},
			..StateConfig::default()
		},
	};
	vr
}

pub fn update_verification_request_on_allot(vr: &mut VerificationRequest<Test>, count: u16) {
	vr.state.stage = VerificationStages::AllotAckVp;
	vr.state.allot.done_count_of_verifiers += count;
	vr.state.allot.pending_count_of_verifiers -= count;
	if vr.state.allot.pending_count_of_verifiers == 0 {
		vr.state.allot.state = false;
		vr.state.allot.ended_at = Some(System::block_number());
	}
}

pub fn update_verification_request_start_ack(vr: &mut VerificationRequest<Test>) {
	let parameters = VerificationProtocol::protocol_parameters();
	// vr.state.ack.pending_count_of_verifiers = parameters.min_count_at_ack_accept_stage;
	vr.state.ack.started_at = System::block_number();
	vr.state.ack.state = true;
	vr.state.ack.state_duration = parameters.max_waiting_time_at_stages;
}

pub fn update_verification_request_start_submit_vp(vr: &mut VerificationRequest<Test>) {
	let parameters = VerificationProtocol::protocol_parameters();
	// vr.state.submit_vp.pending_count_of_verifiers = parameters.min_count_at_submit_vp_stage;
	vr.state.submit_vp.started_at = System::block_number();
	vr.state.submit_vp.state = true;
	vr.state.submit_vp.state_duration = parameters.max_waiting_time_at_stages;
}

pub fn update_verification_request_start_reveal(vr: &mut VerificationRequest<Test>) {
	let parameters = VerificationProtocol::protocol_parameters();
	vr.state.reveal.pending_count_of_verifiers = parameters.min_count_at_reveal_stage;
	vr.state.reveal.started_at = System::block_number();
	vr.state.reveal.state = true;
	vr.state.reveal.state_duration = parameters.max_waiting_time_at_stages;
	vr.state.stage = VerificationStages::Reveal;
	vr.state.eval_vp_state = Some(EvalVpState::Pending);
	vr.state.eval_vp_result = Some(EvalVpResult::Pending);
}

pub fn update_verification_request_on_reveal(
	vr: &mut VerificationRequest<Test>,
	count: u16,
	threshold: u16,
) {
	vr.state.reveal.done_count_of_verifiers += count;
	// vr.state.reveal.pending_count_of_verifiers -= count;
	if vr.state.reveal.done_count_of_verifiers >= threshold {
		vr.state.reveal.state = false;
		vr.state.reveal.ended_at = Some(System::block_number());
		//change stage to Reveal and stop accepting at upper stages
		vr.state.stage = VerificationStages::Eval;
		vr.state.allot.state = false;
		vr.state.ack.state = false;
		vr.state.submit_vp.state = false;
		vr.state.ack.ended_at = Some(System::block_number());
		vr.state.submit_vp.ended_at = Some(System::block_number());
	}
}

pub fn generate_consumer_data_indian_passport() -> (H256, Vec<u8>, Vec<u8>, ConsumerDetails) {
	let secret = "123";
	let issuer = "GOVTOFINDIA";
	let type_of_id = "PASSPORT";
	let country = "INDIA";
	let delimiter = "^";
	let dnf = "001123JONDOESONDOE";
	let dnm = "";
	let dng = "";

	let secret_vecbytes: Vec<u8> = secret.bytes().collect();

	let hash_dnf = keccak_256(dnf.as_bytes());
	let hash_dnm = keccak_256(dnm.as_bytes());
	let hash_dng = keccak_256(dng.as_bytes());

	let clear_vec = country
		.as_bytes()
		.iter()
		.chain(delimiter.as_bytes().iter())
		.chain(issuer.as_bytes().iter())
		.chain(delimiter.as_bytes().iter())
		.chain(type_of_id.as_bytes().iter())
		.chain(delimiter.as_bytes().iter())
		.chain(hash_dnf.iter())
		.chain(delimiter.as_bytes().iter())
		.chain(hash_dnm.iter())
		.chain(delimiter.as_bytes().iter())
		.chain(hash_dng.iter())
		.copied()
		.collect::<Vec<u8>>();

	let combined_clear_vec = clear_vec
		.iter()
		// .chain(delimiter.as_bytes().iter())
		.chain(secret.as_bytes().iter())
		.copied()
		.collect::<Vec<u8>>();

	let keccak256_hash_combined_clear_vec_with_secret = keccak_256(&combined_clear_vec);
	let h256_hash_combined_clear_vec_with_secret =
		H256::from_slice(&keccak256_hash_combined_clear_vec_with_secret);

	let consumer_details = ConsumerDetails {
		country: country.as_bytes().to_vec().try_into().unwrap(),
		id_issuing_authority: issuer.as_bytes().to_vec().try_into().unwrap(),
		type_of_id: type_of_id.as_bytes().to_vec().try_into().unwrap(),
		hash1_name_dob_father: Some(H256::from_slice(&hash_dnf)),
		hash2_name_dob_mother: None,
		hash3_name_dob_guardian: None,
	};

	(h256_hash_combined_clear_vec_with_secret, clear_vec, secret_vecbytes, consumer_details)
}
