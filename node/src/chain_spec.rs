use felidae_runtime::{
	constants::currency::*, opaque::SessionKeys, wasm_binary_unwrap, AccountId,
	AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, Block, CouncilConfig, DemocracyConfig,
	ElectionsConfig, GenesisConfig, GrandpaConfig, ImOnlineConfig, IndicesConfig, MaxNominations,
	NominationPoolsConfig, SessionConfig, Signature, StakerStatus, StakingConfig, SudoConfig,
	SystemConfig, TechnicalCommitteeConfig,
};

use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use serde_json::map::Map;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;

use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

// The URL for the telemetry server.
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

type AccountPublic = <Signature as Verify>::Signer;

fn get_common_properties_map() -> Properties {
	let mut properties = Map::new();
	properties.insert("tokenSymbol".into(), "PAN".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties
}

fn get_dev_properties() -> Properties {
	let mut properties = get_common_properties_map();
	properties.insert("ss58Format".into(), 42.into());
	properties
}

// fn get_mainnet_properties() -> Properties {
//   let mut properties = get_common_properties_map();
//   properties.insert("ss58Format".into(), 51.into());
//   properties
// }

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { babe, grandpa, im_online, authority_discovery }
}

fn staging_testnet_config_genesis() -> GenesisConfig {
	#[rustfmt::skip]
	// stash, controller, session-key
	// generated with secret:
	// for i in 1 2 3 4 ; do for j in stash controller; do subkey inspect "$secret"/fir/$j/$i; done; done
	//
	// and
	//
	// for i in 1 2 3 4 ; do for j in session; do subkey --ed25519 inspect "$secret"//fir//$j//$i; done; done

	let initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)> = vec![
		(
			// 5Ec15LH7q8HJqpCpCVMpFjeh2Yc45FQRxn5oiLz4LSVTL14u
			array_bytes::hex_n_into_unchecked("7057095d75c7bd79e0fae628781857897685a56f277cf5ce74d138419cbddd1c"),
			// 5HTVKiqguCHz5kToLBantKnrN3D8qQ18jY3RxW8NtsW3b4dJ
			array_bytes::hex_n_into_unchecked("ee8d9ead9773a108321fd28f24b8983b11edf197eb44b66061fb2c8b0a82fe51"),
			// 5CuaF5jF64gdt3Vt49hT8TChHx4AoAESD92Sz3xnJnmBwPfM
			array_bytes::hex2array_unchecked("25456531bab5a05e99c31e71f44e634ceca3abdc470060b2cc5f4bb24ce91aa6").unchecked_into(),
			// 5GCT4MoLs73naKmnZdjqKdjzhu3BSaCBXqCMzuyP9JSfQjCQ
			array_bytes::hex2array_unchecked("b6d914c0e0867c0f30be1b2bbb9adb12b2e9ce1fca787bd82378786dc9e2f65e").unchecked_into(),
			// 5Cfq4BxgF7j1xYQhcNNjdggeoyETJ2pBNKyi24vVrqyrUZPx
			array_bytes::hex2array_unchecked("1ac9d269fd7055ad74d00a5bedc2e1073b1b3c69a06664f48d02c523b2c29d65").unchecked_into(),
			// 5HEx8oBqKoYyQ2zfCe1v4yww3dcnTQT2kNzhWM92JrPS54Mo
			array_bytes::hex2array_unchecked("e4fdaf83334c46d90be17c5be7374f28bf450bbfdad2cd48275e038c824e384d").unchecked_into(),
			),
			(
			// 5EeG3Lv2uL6Y5QbnhwhAbA85H8xNM7BFAWjHPqL2BMzjhLuK
			array_bytes::hex_n_into_unchecked("720fe80d17b62d6f649ab8d7e425c06817b54f2f38c4268533909b5512a4fb1f"),
			// 5DXvJrbKvSjYMXb29C4sufgYNZPf79B2EvBKytGnhBiaAQQW
			array_bytes::hex_n_into_unchecked("40fdc65d56b3c2c0ee0a3b3a678d5bd91f21b6f1b05fa9832075aa7ce1326f3a"),
			// 5GQerxpNohj7vS4Rzpdh8jzGhwehCKaCKDJpuc7m3kUKGa8P
			array_bytes::hex2array_unchecked("c027c2951314e08c5a87360bc8ff675e41696029f746ddf5408468a38fce1f71").unchecked_into(),
			// 5EsVEFtr3fPhk9JU9B9UrBPuR5HPtVb73RArQC8sWSBJJMG5
			array_bytes::hex2array_unchecked("7c267df50215050f51b2dc8daf8c8862add3eec8c3c7bd34c7416f142bcc602d").unchecked_into(),
			// 5CajMWJaVpZ71JoquPwb2Q9X7N4yEF6qzhciYdu2gfDVJfZX
			array_bytes::hex2array_unchecked("16e666f279051a7b78488427e0cf6e2ecdd12c1a6fa39cafd187159bece45b5e").unchecked_into(),
			// 5FsCVP9ykMp8LHjCsuVeJQijH3s2mvGhc4rhN1Hia7MeYygg
			array_bytes::hex2array_unchecked("a82a6050c0e374451a1b7f25aff82f12e0198cd97daaecab7be804c0dceca833").unchecked_into(),
			),
			(
			// 5Djy3YvvhWCCkab4ePFESoFt6tBDDaxRGUWnKPjHTuGP2MPw
			array_bytes::hex_n_into_unchecked("4a2dee2b1c17eaf3a80fb4068fd24ed56a054f71831d8e504c9af0c2f35af534"),
			// 5GCUF8WSuHf74x9SStxsTX86uMgmoSzTepKzpUDYERWLehD8
			array_bytes::hex_n_into_unchecked("b6dd128f9ec50bd42026939e53fafd517957ef0ad778d67b4afc93682abd7766"),
			// 5DGoToXx45yvFtBGo6cPwaxiBLuNhF7wrEnfFHqW7JHZZrNw
			array_bytes::hex2array_unchecked("35760c8ac3aa85df80544b20347f782927f0db9f13ccf4001013c89199a352f4").unchecked_into(),
			// 5Ec2PR8EwMuaPRo4oAQVDwZW9Wh9bdR8LGN5ugZy6iU6Rk7W
			array_bytes::hex2array_unchecked("705b73d388629bf6591d96285a527f0fee36d4c66955ccd73b1e3cd20fb20002").unchecked_into(),
			// 5EqZCSPB6FoeRHHYu2WpUT88LFNAHhEQfNBQRfNMUP3RhUDs
			array_bytes::hex2array_unchecked("7aad5c5f332396096f05afc304350b5d13ca262fa3645129cee460173acd1827").unchecked_into(),
			// 5FWLzFus2rDETubyD7EvSSuct3kfFpK5MHbENAaHf7qzv8z3
			array_bytes::hex2array_unchecked("9842d43c280fb08b5d7b8d84683162660fd40b69389acb48ab7da4af2554a26a").unchecked_into(),
			),
			(
			// 5GmbfJoWYwbnPqiCspdVXAVnxUafA6h51ixnJLcTVeSHEsRp
			array_bytes::hex_n_into_unchecked("d02123612afeaeb0f5da1370ec989c34e3f4cffe15635ba8edd90094cd6bf229"),
			// 5CA3vR9ZoBBxXdMLcMMZT1Mh7jUQf3GcpU1hN1p8rBLg3MNe
			array_bytes::hex_n_into_unchecked("0413c544b1303301f9fc622609750daadc29e4a5346c3fb73983274c5f87cf21"),
			// 5CoZje3pPHzsEJ6CxDSFtBhE9UFuV7BYfoVoEu7bKsLTXdYw
			array_bytes::hex2array_unchecked("20b036c9c9a85516ce6a2314a02bcffa1d28e7b50a4bc05aef8a85285f2f318b").unchecked_into(),
			// 5FRc2RDMwRduirLcteT9t2yqew9jaH7rqPV1i3VKBWChtszD
			array_bytes::hex2array_unchecked("94a538cf115ae998fb1c43c364b0f66b7d9d7cff6eb0e78b0144be031c46a91b").unchecked_into(),
			// 5HmqZeWbir1sR9i4f2zvBUG65b5iiz5fz8wBoAexVedTRAHW
			array_bytes::hex2array_unchecked("fc8c2bdce66b710132d0cdac7ce83bf8adf6ae2d5bc9dddcd237641b48c3555a").unchecked_into(),
			// 5CJJZtihoPJ7cyYPGqpSsijcRGb27qWMv7oLbe2bRWT4HYis
			array_bytes::hex2array_unchecked("0a5f097d9b0f5eba22528ecfe85fee43a2225a3d789ca68c98163f0346180235").unchecked_into(),
			),						
	];

	// generated with secret: subkey inspect "$secret"/fir
	let root_key: AccountId = array_bytes::hex_n_into_unchecked(
		// 5Gk2vWjUq1Z75HVhkKKQpEuvrZwnXYG2GAKyS6hU428yuv9K
		"ceefaffb49b14c577806cb62761ae002d82d06a965988ee606eccae9977f5a16",
	);

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(initial_authorities, vec![], root_key, Some(endowed_accounts))
}
/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
	let boot_nodes = vec![];
	ChainSpec::from_genesis(
		"Staging Testnet",
		"staging_testnet",
		ChainType::Live,
		staging_testnet_config_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		None,
		None,
		Some(get_dev_properties()),
		Default::default(),
	)
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
	seed: &str,
) -> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
) -> GenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		]
	});
	// endow all authorities and nominators.
	initial_authorities
		.iter()
		.map(|x| &x.0)
		.chain(initial_nominators.iter())
		.for_each(|x| {
			if !endowed_accounts.contains(x) {
				endowed_accounts.push(x.clone())
			}
		});

	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	let num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: Balance = 10_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;

	GenesisConfig {
		system: SystemConfig { code: wasm_binary_unwrap().to_vec() },
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
		},
		indices: IndicesConfig { indices: vec![] },
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			..Default::default()
		},
		democracy: DemocracyConfig::default(),
		elections: ElectionsConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, STASH))
				.collect(),
		},
		council: CouncilConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
		sudo: SudoConfig { key: Some(root_key) },
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(felidae_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		im_online: ImOnlineConfig { keys: vec![] },
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		grandpa: GrandpaConfig { authorities: vec![] },
		technical_membership: Default::default(),
		treasury: Default::default(),
		transaction_payment: Default::default(),
		nomination_pools: NominationPoolsConfig {
			min_create_bond: 10 * DOLLARS,
			min_join_bond: 1 * DOLLARS,
			..Default::default()
		},
	}
}

fn development_config_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		development_config_genesis,
		vec![],
		None,
		None,
		None,
		Some(get_dev_properties()),
		Default::default(),
	)
}

fn local_testnet_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		local_testnet_genesis,
		vec![],
		None,
		None,
		None,
		None,
		Default::default(),
	)
}
