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
		traits::{tokens::ExistenceRequirement, Currency},
		BoundedVec, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec;
	use sp_io::hashing::keccak_256;
	use sp_runtime::traits::AccountIdConversion;

	use crate::{types::*, verification_process::*};

	use sp_core::H256;

	// use core::mem::discriminant;
	// use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
	use sp_std::collections::btree_map::BTreeMap;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The Currency handler for the pallet.
		type Currency: Currency<Self::AccountId>;

		/// The pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		// type BlockNumber: Codec + EncodeLike + Default + TypeInfo;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// maximum lenght of parameter list_of_documents submitted and stored. its a CID
		type MaxLengthListOfDocuments: Get<u32>;

		// /// pallet DID API
		// type Did: DidPalletProvider;
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

	/// Stores the did creation records
	#[pallet::storage]
	#[pallet::getter(fn did_data)]
	pub(super) type DidData<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber>;

	/// Stores the verification requests
	#[pallet::storage]
	#[pallet::getter(fn verification_requests)]
	pub(super) type VerificationRequests<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, VerificationRequest<T>>;

	// List of verifiers who have done ack. stored by consumer_account_id
	#[pallet::storage]
	#[pallet::getter(fn task_acks)]
	pub(super) type TaskAcks<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>>;

	// Verificatoin parameters submitted by verifiers
	// (consumer_account_id, verifier_account_id) -> submitted_parameters
	#[pallet::storage]
	#[pallet::getter(fn verrification_process_records)]
	pub(super) type VerificationProcessRecords<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		VerificationProcessData<T>,
	>;

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
		/// Update protocol parameters for stages
		ParametersUpdated(ProtocolParameterValues),
		/// Task accepted by the verifier
		/// parameters. [ verifier_accountId, consumer_accountId]
		TaskAccepted(T::AccountId, T::AccountId),
		/// verification data submitted by the verifier
		/// parameters. [ verifier_accountId, consumer_accountId]
		VpSubmitted(T::AccountId, T::AccountId),
		/// Verification data revealed by the verifier
		/// parameters. [ verifier_accountId, consumer_accountId]
		Revealed(T::AccountId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Not elligible to act on the task
		NotAllowed,
		/// Length of the url for the submitted docs too long
		ListOfDocsTooLong,
		/// Length of the url for the submitted docs too short
		ListOfDocsTooShort,
		// On re-submission of request for Did
		CreationRequestAlreadyRegistered,
		VerifierAlreadyRegistered,
		NoVerifierFound,
		// normally this error should not arise
		// Allotted to verifier but not in the proper list
		WronglyAllottedTask,
		AlreadyAccepted,
		AckNotBeingAccepted,
		VpNotBeingAccepted,
		// Submit VP after accepting first only
		AcceptPending,
		// Reveal after Submittinhg the verification para only
		SubmitVpPending,
		AlreadyRevealed,
		RevealNotBeingAccepted,
		VpAlreadySubmitted,
		NoDidReqFound,
		SubmitVpFailed,
		RevealVpFailed,
		TaskAcceptFailed,
		// Revealed data is not same as submitted
		HashMismatch,
		// Revealed data is not in proper format
		InvalidRevealedData,
		// Verification record submitted by verifier ealier in the process not found
		VerificationDataNotFound,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// At block finalization
		fn on_finalize(_now: BlockNumberFor<T>) {
			// at the end of the block, change states of tasks
			let res = Self::app_chain_tasks(_now);
			if let Err(e) = res {
				log::error!("Error: {:?}", e);
			}
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
			//ensure the registration request is not submitted already
			ensure!(
				!VerificationRequests::<T>::contains_key(&_who),
				Error::<T>::CreationRequestAlreadyRegistered
			);

			Self::create_verification_request(&_who, _list_of_documents)?;

			// Emit an event.
			Self::deposit_event(Event::DidCreationRequestCreated(_who));

			Ok(())
		}

		/// Submit the acceptence to take the verification task. Takes
		/// confidance score in the parameter.
		/// Confidence score is taken into account while calculating reward/penalty and gamify the
		/// protocol
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn accept_verification_task(
			origin: OriginFor<T>,
			consumer_account_id: T::AccountId,
			confidence_score: u8,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			Self::ack_verification_task(&_who, &consumer_account_id, confidence_score)?;

			// emit event on ack
			Self::deposit_event(Event::TaskAccepted(_who, consumer_account_id));
			Ok(())
		}

		/// Submit the verification parameter. It takes two parameters
		/// 1. Account Id of the consumer
		/// 2. verification parameters
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn submit_verification_data(
			origin: OriginFor<T>,
			consumer_account_id: T::AccountId,
			verification_parameters: H256,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			Self::submit_verification_parameter(
				&_who,
				&consumer_account_id,
				verification_parameters,
			)?;
			Self::deposit_event(Event::VpSubmitted(_who, consumer_account_id));
			Ok(())
		}

		/// Change protocol parameters
		/// takes new parameters and updates the default value
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn update_protocol_parameters(
			origin: OriginFor<T>,
			new_parameters: ProtocolParameterValues,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			ProtocolParameters::<T>::put(&new_parameters);

			Self::deposit_event(Event::ParametersUpdated(new_parameters));
			Ok(())
		}

		/// Reveal the verification parameters. It takes three parameters
		/// 1. Account Id of the consumer
		/// 2. verification parameters
		/// 3. Secret which was used as salt
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn reveal_data(
			origin: OriginFor<T>,
			consumer_account_id: T::AccountId,
			clear_parameters: Vec<u8>,
			secret: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			Self::reveal_verification_parameter(
				&_who,
				&consumer_account_id,
				clear_parameters,
				secret,
			)?;
			Self::deposit_event(Event::VpSubmitted(_who, consumer_account_id));
			Ok(())
		}
	}

	impl<T: Config> VerificationProcess<T> for Pallet<T> {
		fn create_verification_request(
			_who: &T::AccountId,
			_list_of_documents: Vec<u8>,
		) -> DispatchResult {
			// Get the block number from the FRAME System pallet.
			let current_block = <frame_system::Pallet<T>>::block_number();
			// fetch the protocol parameters
			let parameters = Self::protocol_parameters();
			log::info!(
				"lisf of doc text:{:?}, length:{:?}, max-length-allowed:{:?}",
				_list_of_documents,
				_list_of_documents.len(),
				parameters.max_length_list_of_documents,
			);
			// //ensure the length of the list of the doc is proper
			let bounded_list_of_doc: BoundedVec<u8, T::MaxLengthListOfDocuments> =
				_list_of_documents.try_into().map_err(|_| Error::<T>::ListOfDocsTooLong)?;
			ensure!(bounded_list_of_doc.len() >= 5u8.into(), Error::<T>::ListOfDocsTooShort);

			let vr = VerificationRequest {
				consumer_account_id: _who.clone(),
				submitted_at: current_block,
				list_of_documents: bounded_list_of_doc,
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
			verification_requests: Vec<(&T::AccountId, u16)>,
		) -> DispatchResult {
			let current_block: T::BlockNumber = <frame_system::Pallet<T>>::block_number();
			log::info!(
				"----total requests pending for allotment:{:?}",
				verification_requests.len()
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
				// let mut vr = VerificationRequests::<T>::take(consumer_id).unwrap();
				VerificationRequests::<T>::try_mutate(consumer_id, |v| -> DispatchResult {
					let mut vr = v.as_mut().ok_or(Error::<T>::NoDidReqFound)?;

					let mut allotted_to_count = 0;

					for i in 0..count {
						log::info!("**********alltting:{:?} , out of total: {:?}", i, count);
						log::info!(
							"*-*-*-*-*-*-allotting for task:{:?}, to v number:{:?}",
							&consumer_id,
							count
						);
						let a_v = looped_verifiers.pop();
						match a_v {
							Some(v) => {
								if <VerificationProcessRecords<T>>::contains_key(consumer_id, v) {
									log::warn!(
										"##warning## attempting to allot again. state: count:{:?}",
										i
									);
									// put the  removed verifier back in the list
									looped_verifiers.push(v);
									break
								}

								let vpdata = VerificationProcessData::allot_to_verifier(
									v.clone(),
									current_block,
								);
								VerificationProcessRecords::<T>::insert(consumer_id, v, vpdata);
								Self::deposit_event(Event::VerificatoinTaskAllotted {
									consumer: consumer_id.clone(),
									verifier: v.clone(),
									document: vr.list_of_documents.to_vec(),
								});
								// increment the  count of allotted verifiers
								allotted_to_count += 1;
							},
							None => break,
						}
					}

					if allotted_to_count > 0 {
						// update general stage of the task
						vr.state.stage = VerificationStages::AllotAckVp;
						// update allot stage parameters
						// vr.act_on_fulfilled_allot(allotted_to_count, current_block);
						act_on_fulfilled!(allot, vr, allotted_to_count, current_block);

						// update accept task stage parameters
						start_stage!(
							ack,
							vr,
							parameters.min_count_at_ack_accept_stage,
							parameters.max_waiting_time_at_stages,
							current_block
						);

						// update submit v para stage parameters
						start_stage!(
							submit_vp,
							vr,
							parameters.min_count_at_submit_vp_stage,
							parameters.max_waiting_time_at_stages,
							current_block
						);
					}

					Ok(())
				})?;
			}
			Ok(())
		}

		fn ack_verification_task(
			_who: &T::AccountId,
			consumer_account_id: &T::AccountId,
			confidence_score: u8,
		) -> DispatchResult {
			Self::is_verifier_allowed_ack(&_who, &consumer_account_id)?;
			let current_block = <frame_system::Pallet<T>>::block_number();
			// update verification records
			VerificationProcessRecords::<T>::mutate(
				&consumer_account_id,
				_who,
				|vpr| -> DispatchResult {
					if let Some(v) = vpr {
						v.acknowledged = Some((current_block, confidence_score));
						return Ok(())
					} else {
						return Err(Error::<T>::TaskAcceptFailed.into())
					}
				},
			)?;

			// update verification request meta
			VerificationRequests::<T>::try_mutate(consumer_account_id, |v| -> DispatchResult {
				let mut vr = v.as_mut().ok_or(Error::<T>::NoDidReqFound)?;
				act_on_fulfilled!(ack, vr, 1, current_block);
				Ok(())
			})?;

			Ok(())
		}

		fn is_verifier_allowed_ack(
			_who: &T::AccountId,
			consumer_account_id: &T::AccountId,
		) -> DispatchResult {
			if let Some(r) = VerificationProcessRecords::<T>::get(consumer_account_id, _who) {
				if let Some(_) = r.allotted_at {
					if let Some(_) = r.acknowledged {
						return Err(Error::<T>::AlreadyAccepted.into())
					}
					// check if task is accepting ack
					if let Some(vr) = VerificationRequests::<T>::get(consumer_account_id.clone()) {
						if vr.state.ack.state {
							return Ok(())
						} else {
							return Err(Error::<T>::AckNotBeingAccepted.into())
						}
					}
				}
				return Err(Error::<T>::WronglyAllottedTask.into())
			}
			return Err(Error::<T>::NotAllowed.into())
		}

		fn submit_verification_parameter(
			_who: &T::AccountId,
			consumer_account_id: &T::AccountId,
			verification_parameters: H256,
		) -> DispatchResult {
			Self::is_verifier_allowed_vp(&_who, &consumer_account_id)?;
			let current_block = <frame_system::Pallet<T>>::block_number();
			VerificationProcessRecords::<T>::mutate(
				&consumer_account_id,
				_who,
				|vpr| -> DispatchResult {
					if let Some(v) = vpr {
						v.data = Some((current_block, verification_parameters));
						return Ok(())
					} else {
						return Err(Error::<T>::SubmitVpFailed.into())
					}
				},
			)?;
			// update verification request meta
			VerificationRequests::<T>::try_mutate(consumer_account_id, |v| -> DispatchResult {
				let mut vr = v.as_mut().ok_or(Error::<T>::NoDidReqFound)?;
				// vr.act_on_fulfilled_submit_vp(1, current_block);
				act_on_fulfilled!(submit_vp, vr, 1, current_block);
				Ok(())
			})?;
			Self::deposit_event(Event::<T>::VpSubmitted(_who.clone(), consumer_account_id.clone()));
			Ok(())
		}

		fn is_verifier_allowed_vp(
			_who: &T::AccountId,
			consumer_account_id: &T::AccountId,
		) -> DispatchResult {
			if let Some(r) = VerificationProcessRecords::<T>::get(consumer_account_id, _who) {
				if let Some(_) = r.acknowledged {
					if let Some(_) = r.data {
						return Err(Error::<T>::VpAlreadySubmitted.into())
					}
					// check if task is accepting vp
					if let Some(vr) = VerificationRequests::<T>::get(consumer_account_id.clone()) {
						if vr.state.submit_vp.state {
							return Ok(())
						} else {
							return Err(Error::<T>::VpNotBeingAccepted.into())
						}
					}
				}
				return Err(Error::<T>::AcceptPending.into())
			}
			return Err(Error::<T>::NotAllowed.into())
		}

		fn reveal_verification_parameter(
			_who: &T::AccountId,
			consumer_account_id: &T::AccountId,
			clear_parameters: Vec<u8>,
			secret: Vec<u8>,
		) -> DispatchResult {
			Self::is_verifier_allowed_reveal(&_who, &consumer_account_id)?;
			let current_block = <frame_system::Pallet<T>>::block_number();
			VerificationProcessRecords::<T>::mutate(
				&consumer_account_id,
				_who,
				|vpr| -> DispatchResult {
					if let Some(v) = vpr {
						if let Some((_, hashed_para)) = v.data.clone() {
							Self::does_revealed_data_match(
								&clear_parameters,
								&secret,
								hashed_para,
							)?;

							let reveald_parameter =
								Self::parse_clear_parameters(&clear_parameters)?;
							v.revealed_data = Some((current_block, reveald_parameter));
							return Ok(())
						} else {
							return Err(Error::<T>::SubmitVpPending.into())
						}
					} else {
						return Err(Error::<T>::RevealVpFailed.into())
					}
				},
			)?;
			// update verification request meta
			VerificationRequests::<T>::try_mutate(consumer_account_id, |v| -> DispatchResult {
				let mut vr = v.as_mut().ok_or(Error::<T>::NoDidReqFound)?;
				// vr.act_on_fulfilled_reveal(1, current_block);
				act_on_fulfilled!(reveal, vr, 1, current_block);
				Ok(())
			})?;

			Self::deposit_event(Event::<T>::Revealed(_who.clone(), consumer_account_id.clone()));
			Ok(())
		}

		fn is_verifier_allowed_reveal(
			_who: &T::AccountId,
			consumer_account_id: &T::AccountId,
		) -> DispatchResult {
			if let Some(r) = VerificationProcessRecords::<T>::get(consumer_account_id, _who) {
				if let Some(_) = r.data {
					if let Some(_) = r.revealed_data {
						return Err(Error::<T>::AlreadyRevealed.into())
					}
					// check if task is accepting reveal data
					if let Some(vr) = VerificationRequests::<T>::get(consumer_account_id.clone()) {
						if vr.state.reveal.state {
							return Ok(())
						} else {
							return Err(Error::<T>::RevealNotBeingAccepted.into())
						}
					}
				}
				return Err(Error::<T>::SubmitVpPending.into())
			}
			return Err(Error::<T>::NotAllowed.into())
		}

		fn act_on_wait_over_for_ack(list_verification_req: Vec<&T::AccountId>) -> DispatchResult {
			let current_block: T::BlockNumber = <frame_system::Pallet<T>>::block_number();
			for consumer_id in list_verification_req {
				VerificationRequests::<T>::try_mutate(consumer_id, |v| -> DispatchResult {
					let mut vr = v.as_mut().ok_or(Error::<T>::NoDidReqFound)?;
					let num_of_new_verifiers_required_allot =
						vr.state.ack.pending_count_of_verifiers * 2;
					// vr.start_allot(num_of_new_verifiers_required_allot, 0, current_block);
					start_stage!(allot, vr, num_of_new_verifiers_required_allot, 0, current_block);

					let state_duration_incr_ack =
						vr.state.ack.state_duration * vr.state.ack.round_number as u32;
					// vr.start_ack(0, state_duration_incr_ack, current_block);
					start_stage!(ack, vr, 0, state_duration_incr_ack, current_block);
					Ok(())
				})?;
			}
			Ok(())
		}

		fn act_on_wait_over_for_submit_vp(
			list_verification_req: Vec<&T::AccountId>,
		) -> DispatchResult {
			let current_block: T::BlockNumber = <frame_system::Pallet<T>>::block_number();
			for consumer_id in list_verification_req {
				VerificationRequests::<T>::try_mutate(consumer_id, |v| -> DispatchResult {
					let mut vr = v.as_mut().ok_or(Error::<T>::NoDidReqFound)?;
					let num_of_new_verifiers_required_allot =
						vr.state.submit_vp.pending_count_of_verifiers * 3;
					// vr.start_allot(num_of_new_verifiers_required_allot, 0, current_block);
					start_stage!(allot, vr, num_of_new_verifiers_required_allot, 0, current_block);

					let state_duration_incr_ack =
						vr.state.ack.state_duration * vr.state.ack.round_number as u32;
					let num_of_new_verifiers_required_ack =
						vr.state.submit_vp.pending_count_of_verifiers * 3;

					start_stage!(
						ack,
						vr,
						num_of_new_verifiers_required_ack,
						state_duration_incr_ack,
						current_block
					);

					let state_duration_incr_submit_vp =
						vr.state.submit_vp.state_duration * vr.state.submit_vp.round_number as u32;
					start_stage!(submit_vp, vr, 0, state_duration_incr_submit_vp, current_block);

					Ok(())
				})?;
			}
			Ok(())
		}

		fn start_reveal(list_verification_req: Vec<&T::AccountId>) -> DispatchResult {
			let current_block: T::BlockNumber = <frame_system::Pallet<T>>::block_number();
			let parameters = Self::protocol_parameters();
			for consumer_id in list_verification_req {
				VerificationRequests::<T>::try_mutate(consumer_id, |v| -> DispatchResult {
					let mut vr = v.as_mut().ok_or(Error::<T>::NoDidReqFound)?;
					// vr.start_allot(num_of_new_verifiers_required_allot, 0, current_block);
					start_stage!(
						reveal,
						vr,
						parameters.min_count_at_reveal_stage,
						parameters.max_waiting_time_at_stages,
						current_block
					);

					Ok(())
				})?;
			}
			Ok(())
		}

		fn eval(
			list_verification_req: Vec<&T::AccountId>,
		) -> Result<Vec<(T::AccountId, VerifierUpdateData)>, Error<T>> {
			let mut combined_result: Vec<(T::AccountId, VerifierUpdateData)> = Vec::new();
			for consumer_id in list_verification_req {
				match VerificationRequests::<T>::try_mutate(
					consumer_id,
					|v: &mut Option<VerificationRequest<T>>| -> Result<Vec<(T::AccountId, VerifierUpdateData)>, Error<T>> {
						let mut vr = v.as_mut().ok_or(Error::<T>::NoDidReqFound)?;

						let revealed_data_list: Vec<VerificationProcessData<T>> =
						// let revealed_data_list: Vec<RevealedParameters> =
							VerificationProcessRecords::<T>::iter_prefix_values(
								vr.consumer_account_id.clone(),
							)
							.map(|vpr| 	vpr)
							.collect();
						let (result, incentive_data) = VerificationProcessData::eval_incentive(revealed_data_list);
						vr.state.eval_vp_result = Some(result);
						vr.state.eval_vp_state = Some(EvalVpState::Done);
						Ok(incentive_data)
					},
				) {
					Ok(d) => combined_result.extend(d),
					Err(e) => return Err(e),
				}
			}
			Ok(combined_result)
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn app_chain_tasks(current_block: T::BlockNumber) -> DispatchResult {
			let verifiers: Vec<Verifier<T::AccountId>> = Verifiers::<T>::iter_values().collect();
			// log::info!("+++++++++++++found {:?} verifiers in the system", verifiers.len());

			// #####--Update New Verifiers----------//
			for v in verifiers.iter().filter(|v| v.state == VerifierState::Pending) {
				log::info!("+++++++++++++Updating verifier:{:?} in the system", &v.account_id);
				Verifiers::<T>::mutate(&v.account_id, |v| {
					if let Some(vr) = v {
						vr.state = VerifierState::Active;
					}
				});
			}
			// get sorted list of verifiers to receive tasks
			let active_verifiers: Vec<T::AccountId> = Verifiers::<T>::iter_values()
				.filter(|v| v.state == VerifierState::Active || v.state == VerifierState::Pending)
				.map(|v| v.account_id)
				.collect();
			// #####-----END--Update New Verifiers----//

			// get the list of pending tasks
			let verification_tasks = VerificationRequests::<T>::iter_values().collect::<Vec<_>>();
			let mut pending_allotments: Vec<(&T::AccountId, u16)> = Vec::new();
			let mut submit_vp_completed: Vec<&T::AccountId> = Vec::new();
			let mut pending_eval: Vec<&T::AccountId> = Vec::new();
			for vr_req in verification_tasks.iter() {
				if vr_req.state.stage == VerificationStages::Eval &&
					vr_req.state.eval_vp_state == Some(EvalVpState::Pending)
				{
					// allot state is true so start to allocate task to new verifiers
					pending_eval.push(&vr_req.consumer_account_id);
				} else if !vr_req.state.submit_vp.state &&
					vr_req.state.stage == VerificationStages::AllotAckVp
				{
					// submit_vp state has completed and in AllotAckVp stage
					// start reveal now
					submit_vp_completed.push(&vr_req.consumer_account_id);
				} else if vr_req.state.allot.state {
					// allot state is true so start to allocate task to new verifiers
					pending_allotments.push((
						&vr_req.consumer_account_id,
						vr_req.state.allot.pending_count_of_verifiers,
					));
				}
			}
			// // check new task pending for allotment

			if pending_eval.len() > 0 {
				let result = Self::eval(pending_eval)?;
				//TODO: act on result in verifiers pallet
			}

			if pending_allotments.len() > 0 && active_verifiers.len() > 0 {
				Self::allot_verification_task(active_verifiers, pending_allotments)?;
				Self::deposit_event(Event::AllotmentDone());
			};

			// END--check new task pending for allotment
			//---start reveal check ---
			if submit_vp_completed.len() > 0 {
				// log::info!("%%%--%%% found start reveal cases:{:?}", submit_vp_completed.len());
				Self::start_reveal(submit_vp_completed)?;
			};
			// end --start reveal check --

			let mut list_wait_over_ack: Vec<&T::AccountId> = Vec::new();
			let mut list_wait_over_submit_vp: Vec<&T::AccountId> = Vec::new();

			for vr_req in verification_tasks
				.iter()
				.filter(|v| v.state.ack.state || v.state.submit_vp.state)
			{
				if vr_req.state.submit_vp.state {
					if T::BlockNumber::from(vr_req.state.submit_vp.state_duration) +
						vr_req.state.submit_vp.started_at <
						current_block
					{
						//submitvp wait over
						list_wait_over_submit_vp.push(&vr_req.consumer_account_id);
						//action on this will update ack state para also, so skip
						continue
					}
				}
				if vr_req.state.ack.state {
					if T::BlockNumber::from(vr_req.state.submit_vp.state_duration) +
						vr_req.state.submit_vp.started_at <
						current_block
					{
						//ack wait over
						list_wait_over_ack.push(&vr_req.consumer_account_id);
					}
				}
			}
			if list_wait_over_ack.len() > 0 {
				Self::act_on_wait_over_for_ack(list_wait_over_ack)?;
			}
			if list_wait_over_submit_vp.len() > 0 {
				Self::act_on_wait_over_for_submit_vp(list_wait_over_submit_vp)?;
			}

			Ok(())
		}

		pub(crate) fn account_id(id: T::AccountId) -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(id)
		}

		// check if the reveal data is correct
		pub(crate) fn does_revealed_data_match(
			clear_parameters: &[u8],
			secret: &[u8],
			hashed_para: H256,
		) -> DispatchResult {
			let combined =
				clear_parameters.iter().chain(secret.iter()).copied().collect::<Vec<u8>>();
			let hash = keccak_256(&combined);
			if hash != hashed_para.as_bytes() {
				return Err(Error::<T>::HashMismatch.into())
			}
			Ok(())
		}

		// pub(crate) fn ll(result: EvalVpResult, submissions:)

		pub(crate) fn parse_clear_parameters(
			clear_parameters: &[u8],
		) -> Result<RevealedParameters, Error<T>> {
			// split on carrat symbol
			let split_vec: Vec<_> = clear_parameters.split(|b| *b == b'^').collect();
			match split_vec.len() {
				1 => {
					if split_vec[0] == b"REJECT" {
						// update as reject
						return Ok(RevealedParameters::Reject)
					} else {
						return Err(Error::<T>::InvalidRevealedData.into())
					}
				},
				6 => {
					if split_vec[3].len() != 32 ||
						split_vec[4].len() != 32 || split_vec[5].len() != 32
					{
						log::error!("X0X0X0X0-----InvalidRevealedData length");
						return Err(Error::<T>::InvalidRevealedData.into())
					}
					// update as accept with the parameters
					let consumer_details = ConsumerDetails {
						country: split_vec[0]
							.to_vec()
							.try_into()
							.map_err(|_| Error::<T>::InvalidRevealedData)?,
						id_issuing_authority: split_vec[1]
							.to_vec()
							.try_into()
							.map_err(|_| Error::<T>::InvalidRevealedData)?,
						type_of_id: split_vec[2]
							.to_vec()
							.try_into()
							.map_err(|_| Error::<T>::InvalidRevealedData)?,
						hash1_name_dob_father: H256::from_slice(split_vec[3]),
						hash2_name_dob_mother: H256::from_slice(split_vec[4]),
						hash3_name_dob_guardian: H256::from_slice(split_vec[5]),
					};
					return Ok(RevealedParameters::Accept(consumer_details))
				},
				_ => return Err(Error::<T>::InvalidRevealedData.into()),
			}
		}
	}
}
