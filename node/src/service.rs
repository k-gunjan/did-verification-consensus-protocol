//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use felidae_node_runtime::{self, opaque::Block, RuntimeApi};
// use sc_consensus_aura::{ImportQueueParams, SlotProportion, StartAuraParams};
pub use sc_executor::NativeElseWasmExecutor;
// use sc_finality_grandpa::SharedVoterState;
// use sc_keystore::LocalKeystore;
use sc_service::{error::Error as ServiceError, Configuration, RpcHandlers, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
// use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
// use std::{sync::Arc, time::Duration};
use crate::rpc::{create_full, BabeDeps, FullDeps, GrandpaDeps};
use sc_client_api::BlockBackend;
use sc_consensus_babe::{self, SlotProportion};
use sc_network::NetworkService;
use sc_rpc_api::DenyUnsafe;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;
// Our native executor instance.
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
	/// Only enable the benchmarking host functions when we actually want to benchmark.
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	/// Otherwise we only use the default Substrate host functions.
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		felidae_node_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		felidae_node_runtime::native_version()
	}
}

pub(crate) type FullClient =
	sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

/// The transaction pool type defintion.
pub type TransactionPool = sc_transaction_pool::FullPool<Block, FullClient>;
/// A IO handler that uses all Full RPC extensions.
// pub type IoHandler = jsonrpc_core::IoHandler<sc_rpc::Metadata>;
type FullGrandpaBlockImport =
	sc_finality_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>;

// pub fn new_partial(
// 	config: &Configuration,
// ) -> Result<
// 	sc_service::PartialComponents<
// 		FullClient,
// 		FullBackend,
// 		FullSelectChain,
// 		sc_consensus::DefaultImportQueue<Block, FullClient>,
// 		sc_transaction_pool::FullPool<Block, FullClient>,
// 		(
// 			sc_finality_grandpa::GrandpaBlockImport<
// 				FullBackend,
// 				Block,
// 				FullClient,
// 				FullSelectChain,
// 			>,
// 			sc_finality_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
// 			Option<Telemetry>,
// 		),
// 	>,
// 	ServiceError,
// > {
// 	if config.keystore_remote.is_some() {
// 		return Err(ServiceError::Other("Remote Keystores are not supported.".into()))
// 	}

// 	let telemetry = config
// 		.telemetry_endpoints
// 		.clone()
// 		.filter(|x| !x.is_empty())
// 		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
// 			let worker = TelemetryWorker::new(16)?;
// 			let telemetry = worker.handle().new_telemetry(endpoints);
// 			Ok((worker, telemetry))
// 		})
// 		.transpose()?;

// 	let executor = NativeElseWasmExecutor::<ExecutorDispatch>::new(
// 		config.wasm_method,
// 		config.default_heap_pages,
// 		config.max_runtime_instances,
// 		config.runtime_cache_size,
// 	);

// 	let (client, backend, keystore_container, task_manager) =
// 		sc_service::new_full_parts::<Block, RuntimeApi, _>(
// 			config,
// 			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
// 			executor,
// 		)?;
// 	let client = Arc::new(client);

// 	let telemetry = telemetry.map(|(worker, telemetry)| {
// 		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
// 		telemetry
// 	});

// 	let select_chain = sc_consensus::LongestChain::new(backend.clone());

// 	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
// 		config.transaction_pool.clone(),
// 		config.role.is_authority().into(),
// 		config.prometheus_registry(),
// 		task_manager.spawn_essential_handle(),
// 		client.clone(),
// 	);

// 	let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
// 		client.clone(),
// 		&(client.clone() as Arc<_>),
// 		select_chain.clone(),
// 		telemetry.as_ref().map(|x| x.handle()),
// 	)?;

// 	let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

// 	let import_queue =
// 		sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _>(ImportQueueParams {
// 			block_import: grandpa_block_import.clone(),
// 			justification_import: Some(Box::new(grandpa_block_import.clone())),
// 			client: client.clone(),
// 			create_inherent_data_providers: move |_, ()| async move {
// 				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

// 				let slot =
// 					sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
// 						*timestamp,
// 						slot_duration,
// 					);

// 				Ok((slot, timestamp))
// 			},
// 			spawner: &task_manager.spawn_essential_handle(),
// 			registry: config.prometheus_registry(),
// 			check_for_equivocation: Default::default(),
// 			telemetry: telemetry.as_ref().map(|x| x.handle()),
// 		})?;

// 	Ok(sc_service::PartialComponents {
// 		client,
// 		backend,
// 		task_manager,
// 		import_queue,
// 		keystore_container,
// 		select_chain,
// 		transaction_pool,
// 		other: (grandpa_block_import, grandpa_link, telemetry),
// 	})
// }

// fn remote_keystore(_url: &String) -> Result<Arc<LocalKeystore>, &'static str> {
// 	// FIXME: here would the concrete keystore be built,
// 	//        must return a concrete type (NOT `LocalKeystore`) that
// 	//        implements `CryptoStore` and `SyncCryptoStore`
// 	Err("Remote Keystore not supported.")
// }

/// Builds a new service for a full client.
pub fn new_full(
	config: Configuration,
	disable_hardware_benchmarks: bool,
) -> Result<TaskManager, ServiceError> {
	new_full_base(config, disable_hardware_benchmarks, |_, _| ())
		.map(|NewFullBase { task_manager, .. }| task_manager)
}

/// Creates a new partial node.
pub fn new_partial(
	config: &Configuration,
) -> Result<
	sc_service::PartialComponents<
		FullClient,
		FullBackend,
		FullSelectChain,
		sc_consensus::DefaultImportQueue<Block, FullClient>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		(
			impl Fn(
				DenyUnsafe,
				sc_rpc::SubscriptionTaskExecutor,
			) -> Result<jsonrpsee::RpcModule<()>, sc_service::Error>,
			(
				sc_consensus_babe::BabeBlockImport<Block, FullClient, FullGrandpaBlockImport>,
				sc_finality_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
				sc_consensus_babe::BabeLink<Block>,
			),
			sc_finality_grandpa::SharedVoterState,
			Option<Telemetry>,
		),
	>,
	ServiceError,
> {
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = NativeElseWasmExecutor::<ExecutorDispatch>::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
		config.runtime_cache_size,
	);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
		client.clone(),
		&(client.clone() as Arc<_>),
		select_chain.clone(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;
	let justification_import = grandpa_block_import.clone();

	let (block_import, babe_link) = sc_consensus_babe::block_import(
		sc_consensus_babe::configuration(&*client)?,
		grandpa_block_import,
		client.clone(),
	)?;

	let slot_duration = babe_link.config().slot_duration();
	let import_queue = sc_consensus_babe::import_queue(
		babe_link.clone(),
		block_import.clone(),
		Some(Box::new(justification_import)),
		client.clone(),
		select_chain.clone(),
		move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

			let slot =
				sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
					*timestamp,
					slot_duration,
				);

			let uncles =
				sp_authorship::InherentDataProvider::<<Block as BlockT>::Header>::check_inherents();

			Ok((slot, timestamp, uncles))
		},
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;

	let import_setup = (block_import, grandpa_link, babe_link);

	let (rpc_extensions_builder, rpc_setup) = {
		let (_, grandpa_link, babe_link) = &import_setup;

		let justification_stream = grandpa_link.justification_stream();
		let shared_authority_set = grandpa_link.shared_authority_set().clone();
		let shared_voter_state = sc_finality_grandpa::SharedVoterState::empty();
		let shared_voter_state2 = shared_voter_state.clone();

		let finality_proof_provider = sc_finality_grandpa::FinalityProofProvider::new_for_service(
			backend.clone(),
			Some(shared_authority_set.clone()),
		);

		let babe_config = babe_link.config().clone();
		let shared_epoch_changes = babe_link.epoch_changes().clone();

		let client = client.clone();
		let pool = transaction_pool.clone();
		let select_chain = select_chain.clone();
		let keystore = keystore_container.sync_keystore();
		let chain_spec = config.chain_spec.cloned_box();

		let rpc_backend = backend.clone();
		let rpc_extensions_builder = move |deny_unsafe, subscription_executor| {
			let deps = FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				select_chain: select_chain.clone(),
				chain_spec: chain_spec.cloned_box(),
				deny_unsafe,
				babe: BabeDeps {
					babe_config: babe_config.clone(),
					shared_epoch_changes: shared_epoch_changes.clone(),
					keystore: keystore.clone(),
				},
				grandpa: GrandpaDeps {
					shared_voter_state: shared_voter_state.clone(),
					shared_authority_set: shared_authority_set.clone(),
					justification_stream: justification_stream.clone(),
					subscription_executor,
					finality_provider: finality_proof_provider.clone(),
				},
			};

			create_full(deps, rpc_backend.clone()).map_err(Into::into)
		};

		(rpc_extensions_builder, shared_voter_state2)
	};

	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		keystore_container,
		select_chain,
		import_queue,
		transaction_pool,
		other: (rpc_extensions_builder, import_setup, rpc_setup, telemetry),
	})
}

/// Result of [`new_full_base`].
pub struct NewFullBase {
	/// The task manager of the node.
	pub task_manager: TaskManager,
	/// The client instance of the node.
	pub client: Arc<FullClient>,
	/// The networking service of the node.
	pub network: Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
	/// The transaction pool of the node.
	pub transaction_pool: Arc<TransactionPool>,
	/// The rpc handlers of the node.
	pub rpc_handlers: RpcHandlers,
}

/// Creates a full service from the configuration.
pub fn new_full_base(
	mut config: Configuration,
	disable_hardware_benchmarks: bool,
	with_startup_data: impl FnOnce(
		&sc_consensus_babe::BabeBlockImport<Block, FullClient, FullGrandpaBlockImport>,
		&sc_consensus_babe::BabeLink<Block>,
	),
) -> Result<NewFullBase, ServiceError> {
	let hwbench = if !disable_hardware_benchmarks {
		config.database.path().map(|database_path| {
			let _ = std::fs::create_dir_all(&database_path);
			sc_sysinfo::gather_hwbench(Some(database_path))
		})
	} else {
		None
	};
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (rpc_builder, import_setup, rpc_setup, mut telemetry),
	} = new_partial(&config)?;

	// if let Some(url) = &config.keystore_remote {
	// 	match remote_keystore(url) {
	// 		Ok(k) => keystore_container.set_remote_keystore(k),
	// 		Err(e) =>
	// 			return Err(ServiceError::Other(format!(
	// 				"Error hooking up remote keystore for {}: {}",
	// 				url, e
	// 			))),
	// 	};
	// }

	let shared_voter_state = rpc_setup;

	// let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;
	let grandpa_protocol_name = sc_finality_grandpa::protocol_standard_name(
		&client.block_hash(0).ok().flatten().expect("Genesis block exists; qed"),
		&config.chain_spec,
	);

	config
		.network
		.extra_sets
		.push(sc_finality_grandpa::grandpa_peers_set_config(grandpa_protocol_name.clone()));
	let warp_sync = Arc::new(sc_finality_grandpa::warp_proof::NetworkProvider::new(
		backend.clone(),
		import_setup.1.shared_authority_set().clone(),
		Vec::default(),
	));

	let (network, system_rpc_tx, tx_handler_controller, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: Some(WarpSyncParams::WithProvider(warp_sync)),
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	// let backoff_authoring_blocks: Option<()> = None;
	let backoff_authoring_blocks =
		Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
	let name = config.network.node_name.clone();
	let enable_grandpa = !config.disable_grandpa;
	let prometheus_registry = config.prometheus_registry().cloned();

	// let rpc_extensions_builder = {
	// 	let client = client.clone();
	// 	let pool = transaction_pool.clone();

	// 	Box::new(move |deny_unsafe, _| {
	// 		let deps =
	// 			FullDeps { client: client.clone(), pool: pool.clone(), deny_unsafe };
	// 		create_full(deps).map_err(Into::into)
	// 	})
	// };

	// let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
	// 	network: network.clone(),
	// 	client: client.clone(),
	// 	keystore: keystore_container.sync_keystore(),
	// 	task_manager: &mut task_manager,
	// 	transaction_pool: transaction_pool.clone(),
	// 	rpc_builder: rpc_extensions_builder,
	// 	backend,
	// 	system_rpc_tx,
	// 	tx_handler_controller,
	// 	config,
	// 	telemetry: telemetry.as_mut(),
	// })?;

	let rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		config,
		backend,
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		network: network.clone(),
		rpc_builder: Box::new(rpc_builder),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}

	// if role.is_authority() {
	// 	let proposer_factory = sc_basic_authorship::ProposerFactory::new(
	let (block_import, grandpa_link, babe_link) = import_setup;
	(with_startup_data)(&block_import, &babe_link);

	if let sc_service::config::Role::Authority { .. } = &role {
		let proposer = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		// let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

		let client_clone = client.clone();
		let slot_duration = babe_link.config().slot_duration();

		let babe_config = sc_consensus_babe::BabeParams {
			keystore: keystore_container.sync_keystore(),
			client: client.clone(),
			select_chain,
			env: proposer,
			block_import,
			sync_oracle: network.clone(),
			justification_sync_link: network.clone(),
			create_inherent_data_providers: move |parent, ()| {
				let client_clone = client_clone.clone();
				async move {
					let uncles = sc_consensus_uncles::create_uncles_inherent_data_provider(
						&*client_clone,
						parent,
					)?;

					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

					let slot =
						// sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);

					let storage_proof =
						sp_transaction_storage_proof::registration::new_data_provider(
							&*client_clone,
							&parent,
						)?;

					Ok((slot, timestamp, uncles, storage_proof))
				}
			},
			force_authoring,
			backoff_authoring_blocks,
			babe_link,
			block_proposal_slot_portion: SlotProportion::new(0.5),
			max_block_proposal_slot_portion: None,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
		};

		let babe = sc_consensus_babe::start_babe(babe_config)?;
		task_manager.spawn_essential_handle().spawn_blocking(
			"babe-proposer",
			Some("block-authoring"),
			babe,
		);
	}

	if enable_grandpa {
		// if the node isn't actively participating in consensus then it doesn't
		// need a keystore, regardless of which protocol we use below.
		let keystore =
			if role.is_authority() { Some(keystore_container.sync_keystore()) } else { None };

		let config = sc_finality_grandpa::Config {
			// FIXME #1578 make this available through chainspec
			gossip_duration: std::time::Duration::from_millis(333),
			justification_period: 512,
			name: Some(name),
			observer_enabled: false,
			keystore,
			local_role: role,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			protocol_name: grandpa_protocol_name,
		};

		// start the full GRANDPA voter
		// NOTE: non-authorities could run the GRANDPA observer protocol, but at
		// this point the full voter should provide better guarantees of block
		// and vote data availability than the observer. The observer has not
		// been tested extensively yet and having most nodes in a network run it
		// could lead to finality stalls.
		let grandpa_config = sc_finality_grandpa::GrandpaParams {
			config,
			link: grandpa_link,
			network: network.clone(),
			voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
		};

		// the GRANDPA voter task is considered infallible, i.e.
		// if it fails we take down the service with it.
		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			None,
			sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	}

	network_starter.start_network();
	Ok(NewFullBase { task_manager, client, network, transaction_pool, rpc_handlers })
}
