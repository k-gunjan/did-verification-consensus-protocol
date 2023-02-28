#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
pub mod types;
pub mod verification_process;
// pub use log;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		inherent::Vec,
		log,
		pallet_prelude::*,
		traits::{tokens::ExistenceRequirement, ConstU8, Currency},
		BoundedVec, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec;
	use sp_runtime::traits::AccountIdConversion;
	use sp_runtime::traits::AtLeast32BitUnsigned;
	// use core::fmt::Debug;

	use codec::{Codec, Decode, Encode, EncodeLike};
	use core::fmt::Debug;

	use scale_info::TypeInfo;
	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};

	pub use crate::types::*;
	pub use crate::verification_process::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type AccountOf<T> = <T as frame_system::Config>::AccountId;

	type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The Currency handler for the pallet.
		type Currency: Currency<Self::AccountId>;

		/// The pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		// type BlockNumber: Codec + EncodeLike + Default + TypeInfo;

		type Balance: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// maximum lenght of parameter list_of_documents submitted and stored. its a CID
		type MaxLengthListOfDocuments: Get<u32>;
		/// Minimum number of verification parameters required at the reveal phase. say X
		type MinCountatVPRevealStage: Get<u32>;
		/// Count multiplier to above at the allotment stage. say 4 * X
		type MinCountatAllotStage: Get<u32>;
		/// Count multiplier to minimum at the Ack stage. say 3 * X
		type MinCountatAckAcceptStage: Get<u32>;
		/// Count multiplier to minimum at the Submit Verification Para stage. say 2 * X
		type MinCountatSubmitVPStage: Get<u32>;
		/// Count multiplier to minimum at the Reveal stage. say X equal to the minimum
		type MinCountatRevealStage: Get<u32>;
		/// Waiting period at each stage to receive CountXat<stage> submissions. say 1hr (3600/6 = 600 blocks)
		type MaxWaitingTimeAtStages: Get<u32>;

		// pub const MaxLengthListOfDocuments: u32= 150;
		// pub const MinCountatVPRevealStage: u32= 5;
		// pub const MinCountatAllotStage: u32 = 20;
		// pub const MinCountatAckAcceptStage: u32 = 15;
		// pub const MinCountatSubmitVPStage: u32 = 10;
		// pub const MinCountatRevealStage: u32 = 5;
		// pub const MaxWaitingTimeAtStages: u32 = 1 * HOURS as u32 ;
	}

	// storage to hold the list of verifiers
	#[pallet::storage]
	#[pallet::getter(fn verifiers)]
	pub type Verifiers<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Verifier<T::AccountId>, OptionQuery>;

	// Store the protocol parameters
	#[pallet::storage]
	#[pallet::getter(fn protocol_parameters)]
	pub type ProtocolParameters<T> = StorageValue<_, ProtocolParameterValues, ValueQuery>;

	/// Stores the verification requests
	#[pallet::storage]
	#[pallet::getter(fn verification_requests)]
	pub(super) type VerificationRequests<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, VerificationRequest<T>>;

	/// Stores the verification tasks by consumer_account_id->[verifier_account_id]
	#[pallet::storage]
	#[pallet::getter(fn verification_tasks)]
	pub(super) type VerificationTasks<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>>;

	// List of verifiers who have done ack. stored by consumer_account_id
	#[pallet::storage]
	#[pallet::getter(fn task_acks)]
	pub(super) type TaskAcks<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>>;

	// Verificatoin parameters submitted by verifiers
	// (consumer_account_id, verifier_account_id) -> submitted_parameters
	#[pallet::storage]
	#[pallet::getter(fn verrification_process_records)]
	pub(super) type VerificationProcessRecords<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::AccountId, T::AccountId), VerificationProcessData<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// On DID verification request accpted
		/// parameters. [consumer_accountId]
		DidCreationRequestCreated(T::AccountId),
		/// On Ack by verifier
		/// parameters. [ verifier_accountId, consumer_accountId]
		AcceptTask(T::AccountId, T::AccountId),
		/// test event on allot
		AllotmentDone(),
		/// New verifier registration request created
		VerifierRegistrationRequest(T::AccountId),
		VerificatoinTaskAllotted {
			consumer: T::AccountId,
			verifier: T::AccountId,
			document: Vec<u8>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Not elligible to act on the task
		NotAllowed,
		ListOfDocsTooLong,
		ListOfDocsTooShort,
		// On re-submission of request
		CreationRequestAlreadyRegistered,
		VerifierAlreadyRegistered,
		NoVerifierFound,
		// normally this error should not arise
		TaskAlreadyAllotted,
		// normally this error should not arise
		WronglyAllottedTask,
		AlreadyAccepted,
		AckNotBeingAccepted,
		VpNotBeingAccepted,
		// Submit VP after accepting first only
		AcceptPending,
		SubmitVpPending,
		AlreadyRevealed,
		RevealNotBeingAccepted,
		VpAlreadySubmitted,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// At block finalization
		fn on_finalize(_now: BlockNumberFor<T>) {
			// at the end of the block, change states of tasks
			Self::system_tasks();
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a verifier. Takes following parameters
		/// 1. account_id : of the verifier and
		/// 2. deposit amount
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn register_verifier(origin: OriginFor<T>, deposit: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// check if the verifier is already registered
			ensure!(
				!<Verifiers<T>>::contains_key(who.clone()),
				Error::<T>::VerifierAlreadyRegistered
			);
			T::Currency::transfer(
				&who,
				&Self::account_id(who.clone()),
				deposit,
				ExistenceRequirement::AllowDeath,
			)?;

			let verifier = Verifier {
				account_id: who.clone(),
				score: 0u32,
				state: VerifierState::Pending,
				count_of_accepted_submissions: 0u128,
				count_of_rejected_submissions: 0u128,
				count_of_incompleted_processes: 0u128,
			};

			// Update Verifiers storage.
			<Verifiers<T>>::insert(who.clone(), verifier.clone());

			// Emit an event.
			Self::deposit_event(Event::VerifierRegistrationRequest(who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// Submit new did creation request. Takes following parameters
		/// 1. list of documents submitted for verification. Douments are uploaded in
		/// IPFS and CIDs are submitted here
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn submit_did_creation_request(
			origin: OriginFor<T>,
			_list_of_documents: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			//ensure the registration is not submitted already
			ensure!(
				!VerificationRequests::<T>::contains_key(&_who),
				Error::<T>::CreationRequestAlreadyRegistered
			);

			// //ensure the length of the list of the doc is proper
			log::info!("lisf of doc text:{:?}", _list_of_documents);
			let bounded_list_of_doc: BoundedVec<_, _> =
				_list_of_documents.try_into().map_err(|_| Error::<T>::ListOfDocsTooLong)?;
			ensure!(bounded_list_of_doc.len() >= 5usize, Error::<T>::ListOfDocsTooShort);

			Self::create_verification_request(&_who, bounded_list_of_doc)?;

			// Emit an event.
			Self::deposit_event(Event::DidCreationRequestCreated(_who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// Submit the acceptence to take the verification task. Takes
		/// confidance score in the parameter.
		/// Confidence score is taken into account while calculating reward/penalty and gamify the  protocol
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn accept_verification_task(
			origin: OriginFor<T>,
			consumer_account_id: T::AccountId,
			confidence_score: Option<BoundedVec<u8, ConstU32<10>>>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			Self::ack_verification_task(&_who, &consumer_account_id, confidence_score)?;

			// emit event on ack
			Self::deposit_event(Event::AcceptTask(_who, consumer_account_id));
			Ok(())
		}

		/// Submit the verification parameter. It takes two parameters
		/// 1. Account Id of the consumer
		/// 2. verification parameters
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn submit_verification_parameter(
			origin: OriginFor<T>,
			consumer_account_id: T::AccountId,
			verification_parameters: VerificationParameter,
		) -> DispatchResult {
			Ok(())
		}
	}

	impl<T: Config> VerificationProcess<T> for Pallet<T> {
		fn create_verification_request(
			_who: &T::AccountId,
			_list_of_documents: BoundedVec<u8, T::MaxLengthListOfDocuments>,
		) -> DispatchResult {
			// Get the block number from the FRAME System pallet.
			let current_block = <frame_system::Pallet<T>>::block_number();
			// fetch the protocol parameters
			let parameters = Self::protocol_parameters();

			let vr = VerificationRequest {
				consumer_account_id: _who.clone(),
				submitted_at: current_block,
				list_of_documents: _list_of_documents,
				list_of_id_hashes: BoundedVec::try_from(vec![]).unwrap(),
				did_creation_status: DidCreationStatus::default(),
				state: StateConfig {
					allot: StateAttributes {
						done_count_of_verifiers: 0,
						pending_count_of_verifiers: parameters.min_count_at_allot_stage,
						round_number: 1,
						state: true,
						started_at: current_block,
						ended_at: None.into(),
						state_duration: parameters.max_waiting_time_at_stages,
					},
					..StateConfig::default()
				},
			};

			// Store the new request
			VerificationRequests::<T>::insert(_who, vr);

			Ok(())
		}

		fn allot_verification_task(
			verifiers: Vec<T::AccountId>,
			verification_requests: Vec<(&T::AccountId, u8)>,
		) -> DispatchResult {
			let current_block = <frame_system::Pallet<T>>::block_number();
			log::info!("total requests:{:?}", verification_requests.len());
			log::info!(
				"Hello World from verification protocol!",
				// verification_requests[0].state
			);

			let total_v_required: u32 =
				verification_requests.clone().iter().map(|(_, c)| *c as u32).sum();
			let mut looped_verifiers: Vec<_> =
				verifiers.iter().cycle().take(total_v_required as usize).collect();
			log::info!(
				"total verifiers required:{:?}, total v in looped list:{:?}",
				total_v_required,
				looped_verifiers.len()
			);

			// fetch protocol parameters
			let parameters = Self::protocol_parameters();
			for (consumer_id, count) in verification_requests.into_iter() {
				let mut vr = VerificationRequests::<T>::take(consumer_id).unwrap();

				for _ in 0..count {
					log::debug!("allotting for task:{:?}, to v number:{:?}", &consumer_id, count);
					let a_v = looped_verifiers.pop();
					match a_v {
						Some(v) => {
							VerificationTasks::<T>::append(&consumer_id, v);
							ensure!(
								!<VerificationProcessRecords<T>>::contains_key((consumer_id, v)),
								Error::<T>::TaskAlreadyAllotted
							);
							let vpdata = VerificationProcessData {
								verifier_account_id: v.clone(),
								allotted: Some(current_block),
								acknowledged: None.into(),
								data: None.into(),
								revealed_data: None.into(),
								is_valid: None.into(),
							};
							VerificationProcessRecords::<T>::insert((consumer_id, v), vpdata);
							Self::deposit_event(Event::VerificatoinTaskAllotted {
								consumer: consumer_id.clone(),
								verifier: v.clone(),
								document: vr.list_of_documents.to_vec(),
							});
						},
						None => break,
					}
				}
				// update allot stage parameters
				vr.state.allot.state = false;
				vr.state.allot.done_count_of_verifiers += count;
				vr.state.allot.pending_count_of_verifiers -= count;
				vr.state.allot.ended_at = Some(current_block);

				// update general stage of the task
				vr.state.stage = VerificationStages::AllotAckVp;

				// update accept task stage parameters
				vr.state.ack.state = true;
				vr.state.ack.pending_count_of_verifiers = parameters.min_count_at_ack_accept_stage;
				vr.state.ack.round_number = 1;
				vr.state.ack.started_at = current_block;
				vr.state.ack.state_duration = parameters.max_waiting_time_at_stages;

				// update submit v para stage parameters
				vr.state.submit_vp.state = true;
				vr.state.submit_vp.pending_count_of_verifiers =
					parameters.min_count_at_submit_vp_stage;
				vr.state.submit_vp.round_number = 1;
				vr.state.submit_vp.started_at = current_block;
				vr.state.submit_vp.state_duration = parameters.max_waiting_time_at_stages;

				log::info!(
					"******Hello World from verification protocol, updated value:{:?}",
					vr.state.stage.clone()
				);
				// Store the updated request
				VerificationRequests::<T>::insert(vr.consumer_account_id.clone(), vr);
			}
			//  TODO!//
			Ok(())
		}

		fn ack_verification_task(
			_who: &T::AccountId,
			consumer_account_id: &T::AccountId,
			confidence_score: Option<BoundedVec<u8, ConstU32<10>>>,
		) -> DispatchResult {
			Self::is_verifier_allowed_ack(&_who, &consumer_account_id)?;

			// TODO!
			Ok(())
		}

		fn is_verifier_allowed_ack(
			_who: &T::AccountId,
			consumer_account_id: &T::AccountId,
		) -> DispatchResult {
			if let Some(r) = VerificationProcessRecords::<T>::get((consumer_account_id, _who)) {
				if let Some(_) = r.allotted {
					if let Some(_) = r.acknowledged {
						return Err(Error::<T>::AlreadyAccepted.into());
					}
					// check if task is accepting ack
					if let Some(vr) = VerificationRequests::<T>::get(consumer_account_id.clone()) {
						if vr.state.ack.state {
							return Ok(());
						} else {
							return Err(Error::<T>::AckNotBeingAccepted.into());
						}
					}
				}
				return Err(Error::<T>::WronglyAllottedTask.into());
			}
			return Err(Error::<T>::NotAllowed.into());
		}

		fn submit_verification_parameter(
			_who: &T::AccountId,
			consumer_account_id: &T::AccountId,
			verification_parameters: VerificationParameter,
		) -> DispatchResult {
			Self::is_verifier_allowed_vp(&_who, &consumer_account_id)?;
			//TODO!
			Ok(())
		}

		fn is_verifier_allowed_vp(
			_who: &T::AccountId,
			consumer_account_id: &T::AccountId,
		) -> DispatchResult {
			if let Some(r) = VerificationProcessRecords::<T>::get((consumer_account_id, _who)) {
				if let Some(_) = r.acknowledged {
					if let Some(_) = r.data {
						return Err(Error::<T>::VpAlreadySubmitted.into());
					}
					// check if task is accepting vp
					if let Some(vr) = VerificationRequests::<T>::get(consumer_account_id.clone()) {
						if vr.state.submit_vp.state {
							return Ok(());
						} else {
							return Err(Error::<T>::VpNotBeingAccepted.into());
						}
					}
				}
				return Err(Error::<T>::AcceptPending.into());
			}
			return Err(Error::<T>::NotAllowed.into());
		}

		fn reveal_verification_parameter(
			_who: &T::AccountId,
			consumer_account_id: &T::AccountId,
			verification_parameters: Vec<T::Hash>,
			secret: BoundedVec<u8, ConstU8<20>>,
		) -> DispatchResult {
			Self::is_verifier_allowed_reveal(&_who, &consumer_account_id)?;
			//TODO!
			Ok(())
		}

		fn is_verifier_allowed_reveal(
			_who: &T::AccountId,
			consumer_account_id: &T::AccountId,
		) -> DispatchResult {
			if let Some(r) = VerificationProcessRecords::<T>::get((consumer_account_id, _who)) {
				if let Some(_) = r.data {
					if let Some(_) = r.revealed_data {
						return Err(Error::<T>::AlreadyRevealed.into());
					}
					// check if task is accepting reveal data
					if let Some(vr) = VerificationRequests::<T>::get(consumer_account_id.clone()) {
						if vr.state.reveal.state {
							return Ok(());
						} else {
							return Err(Error::<T>::RevealNotBeingAccepted.into());
						}
					}
				}
				return Err(Error::<T>::SubmitVpPending.into());
			}
			return Err(Error::<T>::NotAllowed.into());
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn system_tasks() -> DispatchResult {
			let verifiers: Vec<Verifier<T::AccountId>> = Verifiers::<T>::iter_values().collect();
			log::info!("+++++++++++++found {:?} verifiers in the system", verifiers.len());

			// -------Update New Verifiers----------//
			for v in verifiers.iter().filter(|v| v.state == VerifierState::Pending) {
				log::info!("+++++++++++++Updating verifier:{:?} in the system", &v.account_id);
				Verifiers::<T>::mutate(&v.account_id, |v| {
					// let vr = v.as_mut().ok_or(Error::<T>::NoVerifierFound)?;
					if let Some(vr) = v {
						vr.state = VerifierState::Active;
					}
					// Ok(())
				});
			}
			// get sorted list of verifiers to receive tasks
			let active_verifiers: Vec<T::AccountId> = Verifiers::<T>::iter_values()
				.filter(|v| v.state == VerifierState::Active || v.state == VerifierState::Pending)
				.map(|v| v.account_id)
				.collect();

			// //
			// let mut verification_tasks_to_allot: Vec<VerificationRequest<T>> = verifiers.clone();

			let verification_tasks = VerificationRequests::<T>::iter_values().collect::<Vec<_>>();

			let pending_allotments = verification_tasks
				.iter()
				.filter(|v| v.state.allot.state)
				.map(|v| (&v.consumer_account_id, v.state.allot.pending_count_of_verifiers))
				.collect::<Vec<_>>();
			// Self::allot_verification_task(pending_allotments)?;

			if pending_allotments.len() > 0 {
				Self::allot_verification_task(active_verifiers, pending_allotments)?;
				Self::deposit_event(Event::AllotmentDone());
			};
			Ok(())
		}

		pub(crate) fn account_id(id: T::AccountId) -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(id)
		}
	}
}
