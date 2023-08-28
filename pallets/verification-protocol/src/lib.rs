#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
pub mod invalidate;
pub mod types;
pub mod verification_process;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		inherent::Vec,
		log,
		pallet_prelude::{DispatchResult, OptionQuery, ValueQuery, *},
		traits::Currency,
		BoundedVec, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec;
	use sp_io::hashing::keccak_256;
	use sp_runtime::traits::AccountIdConversion;

	use crate::{
		invalidate::{Invalidate, ReasonToInvalidate},
		types::*,
		verification_process::*,
	};
	use pallet_did::pallet::DidProvider;
	use sp_core::{ConstU32, H256};
	use sp_std::borrow::ToOwned;
	use verifiers::{pallet::VerifiersProvider, types::VerifierUpdateData};

	type IdDocumentOf<T> = <<T as Config>::IdDocument as IdDocument>::IdType;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

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

		/// pallet verifier API
		type VerifiersProvider: VerifiersProvider<
			AccountId = Self::AccountId,
			UpdateData = VerifierUpdateData,
			BlockNumber = Self::BlockNumber,
		>;

		/// pallet did API
		type DidProvider: DidProvider<AccountId = Self::AccountId>;

		/// IdDocument
		type IdDocument: IdDocument<IdType = IdType<Self>, Error = Error<Self>>;
	}

	// Store the list of whitelisted countries
	#[pallet::storage]
	#[pallet::getter(fn whitelisted_countries)]
	pub(super) type WhitelistedCountries<T> =
		StorageValue<_, BoundedVec<Country, ConstU32<200>>, ValueQuery>;

	// Stores the whitelisted IDs of countries
	#[pallet::storage]
	#[pallet::getter(fn whitelisted_id_types)]
	pub(super) type WhitelistedIdTypes<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Country,
		BoundedVec<IdDocumentOf<T>, ConstU32<10>>,
		ValueQuery,
	>;

	// Store the protocol parameters
	#[pallet::storage]
	#[pallet::getter(fn protocol_parameters)]
	pub(super) type ProtocolParameters<T> = StorageValue<_, ProtocolParameterValues, ValueQuery>;

	/// Stores the did creation records
	#[pallet::storage]
	#[pallet::getter(fn consumer_hashes)]
	pub(super) type ConsumerHashes<T: Config> =
		StorageMap<_, Blake2_128Concat, H256, (T::AccountId, T::BlockNumber), OptionQuery>;

	/// Stores the verification requests
	#[pallet::storage]
	#[pallet::getter(fn verification_requests)]
	pub(super) type VerificationRequests<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, VerificationRequest<T>>;

	/// Stores the verification results
	#[pallet::storage]
	#[pallet::getter(fn verification_results)]
	pub(super) type VerificationResults<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, VerificationResult<T>>;

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
		/// On DID verification request accpted
		/// parameters. [consumer_accountId]
		DidCreationRequest(T::AccountId),

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
		/// Verification completed event
		/// parameters. [ consumer_accountId, DidCreationStatus]
		DidCreationResult(T::AccountId, DidCreationStatus),
		/// parameters [IdType]
		IdTypeWhitelisted(IdDocumentOf<T>),
		/// parameters [IdType]
		IdTypeRemoved(IdDocumentOf<T>),
		/// Invalidated Ids
		DidInvalidation(BoundedVec<T::AccountId, ConstU32<1000>>),
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
		// Did already created for the account
		AlreadyCreated4Account,
		//Consumer hash already registered for some DID
		HashAlreadyRegistered,
		// IdType not defined
		IdTypeNotDefined,
		// Storage update failed
		UpdateFailed,
		InvalidIdName,
		InvalidIdIssuer,
		InvalidCountry,
		// ID not eligible for action or not verified or invalidated
		WrongIdState,
		// ID document type present still/already
		IdTypeWhitelisted,
		// DID not in valid state in DID module
		DidNotValid,
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
		/// Submit new did creation request. Takes following parameters
		/// 1. list of documents submitted for verification. Douments are uploaded in
		/// IPFS and CIDs are submitted here
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(0)]
		pub fn submit_did_creation_request(
			origin: OriginFor<T>,
			_list_of_documents: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			// ensure that the did is not created already for the account
			ensure!(!T::DidProvider::is_account_created(&_who), Error::<T>::AlreadyCreated4Account);
			//ensure the registration request is not submitted already
			ensure!(
				!VerificationRequests::<T>::contains_key(&_who),
				Error::<T>::CreationRequestAlreadyRegistered
			);

			Self::create_verification_request(&_who, _list_of_documents)?;

			// Emit an event.
			Self::deposit_event(Event::DidCreationRequest(_who));

			Ok(())
		}

		/// Submit the acceptence to take the verification task. Takes
		/// confidance score in the parameter.
		/// Confidence score is taken into account while calculating reward/penalty and gamify the
		/// protocol
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(1)]
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
		#[pallet::call_index(2)]
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
		#[pallet::call_index(3)]
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
		#[pallet::call_index(4)]
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
			Self::deposit_event(Event::Revealed(_who, consumer_account_id));
			Ok(())
		}

		/// Inster a new ID Type. It takes new IdType
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(5)]
		pub fn whitelist_id_type(origin: OriginFor<T>, id_type: IdDocumentOf<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			let IdType { country, .. } = &id_type;
			let countries = Self::whitelisted_countries();
			if !countries.contains(&country) {
				WhitelistedCountries::<T>::try_append(&country)
					.map_err(|_| Error::<T>::UpdateFailed)?;
			}

			let whitelisted_id_types = Self::whitelisted_id_types(country);
			if !whitelisted_id_types.contains(&id_type) {
				WhitelistedIdTypes::<T>::try_append(country, &id_type)
					.map_err(|_| Error::<T>::UpdateFailed)?;
			}

			Self::deposit_event(Event::IdTypeWhitelisted(id_type));
			Ok(())
		}

		/// Removes a whitelisted ID Type. It takes new IdType. After removing, if no entry
		/// left for the corresponding county, country is removed from the country_whitelist
		/// also.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(6)]
		pub fn remove_id_type(origin: OriginFor<T>, id_type: IdDocumentOf<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			let (country, count) = Self::validate_id_type(&id_type)?;

			WhitelistedIdTypes::<T>::mutate(&country, |whitelist| {
				*whitelist = whitelist
					.iter()
					.cloned()
					.filter(|x| *x != id_type)
					.collect::<Vec<_>>()
					.try_into()
					.expect("Error in updating whitelisted_id_types. This should not happen!!");
			});

			// remove the country from the list as no id_type for this country exists
			if count == 1 {
				WhitelistedCountries::<T>::mutate(|vc| {
					*vc = vc
						.iter()
						.cloned()
						.filter(|x| *x != country)
						.collect::<Vec<Country>>()
						.try_into()
						.expect(
							"Error in updating whitelisted_countries. This should not happen!!",
						);
				});
			}

			Self::deposit_event(Event::IdTypeRemoved(id_type));
			Ok(())
		}
		/// Removes a whitelisted ID Type. It takes new IdType. After removing, if no entry
		/// left for the corresponding county, country is removed from the country_whitelist
		/// also.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(7)]
		pub fn invalidate_ids(
			origin: OriginFor<T>,
			ids: BoundedVec<T::AccountId, ConstU32<1000>>,
			reason: ReasonToInvalidate,
		) -> DispatchResult {
			ensure_signed(origin)?;
			// let mut invalidated_ids: Vec<T::AccountId> = Vec::with_capacity(1000);
			for id in ids.iter() {
				<ReasonToInvalidate as Invalidate<T>>::is_verified(&id)?;
			}

			for id in ids.iter() {
				<ReasonToInvalidate as Invalidate<T>>::invalidate(&reason, &id)?;
			}
			// let invalidated_ids_bounded: BoundedVec<T::AccountId, ConstU32<1000>> =
			// 	invalidated_ids.try_into().expect("it can not be more than 1000");
			Self::deposit_event(Event::DidInvalidation(ids));
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

			//ensure the length of the doc URL is proper
			let bounded_list_of_doc: BoundedVec<u8, T::MaxLengthListOfDocuments> =
				_list_of_documents.try_into().map_err(|_| Error::<T>::ListOfDocsTooLong)?;
			ensure!(bounded_list_of_doc.len() >= 5u8.into(), Error::<T>::ListOfDocsTooShort);

			let vr = VerificationRequest {
				consumer_account_id: _who.clone(),
				submitted_at: current_block,
				list_of_documents: bounded_list_of_doc,
				did_creation_status: DidCreationStatus::default(),
				round_number: 1,
				state: StateConfig {
					allot: StateAttributes {
						done_count_of_verifiers: 0,
						pending_count_of_verifiers: parameters.min_count_at_allot_stage,
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
			current_block: T::BlockNumber,
			verifiers: Vec<T::AccountId>,
			verification_requests: Vec<(&T::AccountId, u16)>,
		) -> Result<(), Error<T>> {
			let mut verifier_index: usize = 0;
			let total_varifiers = verifiers.len();
			// for every `consumer_id` the task will be allotted to `count` no of verifiers
			for (consumer_id, count) in verification_requests.into_iter() {
				// total number of verifier allotted for the consumer in this block
				let mut allotted_to_count = 0;
				for _ in 0..count {
					// track if not un-allotted verifer left
					let mut all_already_got_this_task = true;
					// Try to allot a verifier. at most check all the verifiers
					for _tried in 0..total_varifiers {
						let chosen_verifier =
							&verifiers[(verifier_index + _tried) % total_varifiers];

						if !VerificationProcessRecords::<T>::contains_key(
							&consumer_id,
							&chosen_verifier,
						) {
							let vpdata = VerificationProcessData::allot_to_verifier(
								chosen_verifier.clone(),
								current_block,
							);
							VerificationProcessRecords::<T>::insert(
								consumer_id.clone(),
								chosen_verifier,
								vpdata,
							);
							allotted_to_count += 1;
							all_already_got_this_task = false;
							break
						}
					}
					verifier_index += 1;

					if all_already_got_this_task {
						// trying to allot again will not succeed
						break
					}
				}

				if allotted_to_count > 0 {
					VerificationRequests::<T>::mutate(consumer_id, |v| -> Result<(), Error<T>> {
						let mut vr = v.as_mut().ok_or(Error::<T>::NoDidReqFound)?;
						// fetch protocol parameters
						let parameters = Self::protocol_parameters();
						// update general stage of the task
						vr.state.stage = VerificationStages::AllotAckVp;
						// update allot stage parameters
						vr.state.allot.done_count_of_verifiers += allotted_to_count;
						vr.state.allot.pending_count_of_verifiers -= allotted_to_count;
						if vr.state.allot.pending_count_of_verifiers == 0 {
							vr.state.allot.state = false;
							vr.state.allot.ended_at = Some(current_block);
						}
						if !vr.state.ack.state {
							vr.state.ack.state = true;
							vr.state.ack.started_at = current_block;
							vr.state.ack.state_duration = parameters.max_waiting_time_at_stages;
						}
						// update submit v para stage parameters
						if !vr.state.submit_vp.state {
							vr.state.submit_vp.state = true;
							vr.state.submit_vp.started_at = current_block;
							vr.state.submit_vp.state_duration =
								parameters.max_waiting_time_at_stages;
						}

						Ok(())
					})?;
				}
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
				vr.state.ack.done_count_of_verifiers += 1;
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
				vr.state.submit_vp.done_count_of_verifiers += 1;
				Ok(())
			})?;
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
							if let RevealedParameters::Accept(consumer_details) =
								reveald_parameter.clone()
							{
								let id_type = IdDocumentOf::<T>::build(
									consumer_details.type_of_id.into(),
									consumer_details.id_issuing_authority.into(),
									consumer_details.country.into(),
								)?;
								let _ = Self::validate_id_type(&id_type)?;
							}
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
			let parameters = Self::protocol_parameters();
			VerificationRequests::<T>::try_mutate(consumer_account_id, |v| -> DispatchResult {
				let mut vr = v.as_mut().ok_or(Error::<T>::NoDidReqFound)?;
				// vr.act_on_fulfilled_reveal(1, current_block);
				vr.state.reveal.done_count_of_verifiers += 1;
				// vr.state.reveal.pending_count_of_verifiers -= 1;
				if vr.state.reveal.done_count_of_verifiers >= parameters.min_count_at_reveal_stage {
					vr.state.reveal.state = false;
					vr.state.reveal.ended_at = Some(current_block);
					//change stage to Reveal and stop accepting at upper stages
					vr.state.stage = VerificationStages::Eval;
					vr.state.allot.state = false;
					vr.state.ack.state = false;
					vr.state.ack.ended_at = Some(current_block);

					vr.state.submit_vp.state = false;
					vr.state.submit_vp.ended_at = Some(current_block);
					//start the next stage: evaluation of revealed parameters
					vr.state.stage = VerificationStages::Eval;
					vr.state.eval_vp_state = Some(EvalVpState::Pending);
					vr.state.eval_vp_result = Some(EvalVpResult::Pending);
				}

				Ok(())
			})?;
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

		fn act_on_wait_over_for_submit_vp(
			current_block: T::BlockNumber,
			list_verification_req: Vec<&T::AccountId>,
		) -> Result<(), Error<T>> {
			for consumer_id in list_verification_req {
				VerificationRequests::<T>::try_mutate(consumer_id, |v| -> Result<(), Error<T>> {
					let mut vr = v.as_mut().ok_or(Error::<T>::NoDidReqFound)?;
					let num_of_new_verifiers_required_allot =
						vr.state.submit_vp.pending_count_of_verifiers * 3;

					vr.round_number += 1;
					if !vr.state.allot.state {
						vr.state.allot.state = true;
						vr.state.allot.pending_count_of_verifiers +=
							num_of_new_verifiers_required_allot;
					}
					let state_duration_incr_submit_vp =
						vr.state.submit_vp.state_duration * vr.round_number as u32;
					vr.state.submit_vp.state_duration += state_duration_incr_submit_vp;

					Ok(())
				})?;
			}
			Ok(())
		}

		fn start_reveal(
			current_block: T::BlockNumber,
			list_verification_req: Vec<&T::AccountId>,
		) -> Result<(), Error<T>> {
			let parameters = Self::protocol_parameters();
			for consumer_id in list_verification_req {
				VerificationRequests::<T>::try_mutate(consumer_id, |v| -> Result<(), Error<T>> {
					let mut vr = v.as_mut().ok_or(Error::<T>::NoDidReqFound)?;
					vr.state.reveal.state = true;
					vr.state.reveal.started_at = current_block;
					// default value started at zero.
					vr.state.reveal.pending_count_of_verifiers +=
						parameters.min_count_at_reveal_stage;
					// state duration to be set not incremented
					vr.state.reveal.state_duration = parameters.max_waiting_time_at_stages;

					// update verification request stage to indicate Reveal state
					// and close previous stages if starting reveal stage

					vr.state.stage = VerificationStages::Reveal;
					vr.state.allot.state = false;
					vr.state.ack.state = false;
					vr.state.submit_vp.state = false;

					Ok(())
				})?;
			}
			Ok(())
		}

		fn eval(
			current_block: T::BlockNumber,
			list_verification_req: Vec<&T::AccountId>,
		) -> Result<Vec<(T::AccountId, VerifierUpdateData)>, Error<T>> {
			// fetch protocol parameters
			let parameters = Self::protocol_parameters();
			// (verifier_account_id , verifier_update_data)
			let mut combined_result: Vec<(T::AccountId, VerifierUpdateData)> = Vec::new();
			for consumer_id in list_verification_req {
				// list of all the verification data submitted for a particular request
				let revealed_data_list: Vec<VerificationProcessData<T>> =
					VerificationProcessRecords::<T>::drain_prefix(consumer_id.clone())
						.map(|(_, v)| v)
						.collect();

				let (result, incentive_data) = VerificationProcessData::eval_incentive(
					revealed_data_list,
					parameters.threshold_winning_percentage,
				);

				//TODO: more particular status of did creation
				let did_creation_status = match result.clone() {
					EvalVpResult::Accepted(consumer_details) => {
						//check hashes and reject if hashes are already there
						if let Err(e) = Self::check_insert_hashes(
							consumer_details.hashes(),
							consumer_id.clone(),
							current_block,
						) {
							log::info!(
								"Did creation of {:?} rejected . Error: {:?}",
								consumer_id.clone(),
								e
							);
							DidCreationStatus::RejectedDuplicate
						} else {
							let r = T::DidProvider::creat_new_did(&consumer_id.clone());
							if r.is_ok() {
								DidCreationStatus::Created
							} else {
								DidCreationStatus::Failed
							}
						}
					},
					_ => DidCreationStatus::Rejected,
				};
				Self::deposit_event(Event::DidCreationResult(
					consumer_id.clone(),
					did_creation_status,
				));

				if let Some(completed_request) = VerificationRequests::<T>::take(consumer_id) {
					let final_result = VerificationResult::<T>::from_completed_request(
						completed_request,
						result.clone(),
						did_creation_status,
						current_block,
					);
					VerificationResults::<T>::insert(consumer_id, final_result);
				}

				combined_result.extend(incentive_data);
			}
			Ok(combined_result)
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn app_chain_tasks(current_block: T::BlockNumber) -> Result<(), Error<T>> {
			let parameters = Self::protocol_parameters();
			// get the list of pending tasks, max 500
			let verification_tasks =
				VerificationRequests::<T>::iter_values().take(1000).collect::<Vec<_>>();
			let mut pending_allotments: Vec<(&T::AccountId, u16)> = Vec::new();
			let mut submit_vp_completed: Vec<&T::AccountId> = Vec::new();
			let mut pending_eval: Vec<&T::AccountId> = Vec::new();
			for vr_req in verification_tasks.iter() {
				if vr_req.state.stage == VerificationStages::Eval &&
					vr_req.state.eval_vp_state == Some(EvalVpState::Pending)
				{
					// allot state is true so start to allocate task to new verifiers
					pending_eval.push(&vr_req.consumer_account_id);
				} else if vr_req.state.submit_vp.done_count_of_verifiers >=
					parameters.min_count_at_submit_vp_stage &&
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

			if pending_eval.len() > 0 {
				let result = Self::eval(current_block, pending_eval);
				match result {
					Ok(s) =>
						if let Err(_) =
							T::VerifiersProvider::update_verifier_profiles(s, current_block)
						{
							log::error!(
								"error in updating the incentive feed to verifier profiles"
							);
						},
					Err(_) => log::error!("Error in evaluating incentive data feed"),
				}
			}

			if pending_allotments.len() > 0 {
				// get sorted list of verifiers to receive tasks
				let active_verifiers: Vec<T::AccountId> = T::VerifiersProvider::get_verifiers();
				if active_verifiers.len() > 0 {
					Self::allot_verification_task(
						current_block,
						active_verifiers,
						pending_allotments,
					)?;
				}
			};

			if submit_vp_completed.len() > 0 {
				Self::start_reveal(current_block, submit_vp_completed)?;
			};

			// let mut list_wait_over_ack: Vec<&T::AccountId> = Vec::new();
			let mut list_wait_over_submit_vp: Vec<&T::AccountId> = Vec::new();

			for vr_req in verification_tasks.iter().filter(|v| v.state.submit_vp.state) {
				if vr_req.state.submit_vp.state {
					if T::BlockNumber::from(vr_req.state.submit_vp.state_duration) +
						vr_req.state.submit_vp.started_at <
						current_block
					{
						//submitvp wait over
						list_wait_over_submit_vp.push(&vr_req.consumer_account_id);
					}
				}
			}

			if list_wait_over_submit_vp.len() > 0 {
				Self::act_on_wait_over_for_submit_vp(current_block, list_wait_over_submit_vp)?;
			}

			Ok(())
		}

		pub(crate) fn _account_id(id: T::AccountId) -> T::AccountId {
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
						return Err(Error::<T>::InvalidRevealedData.into())
					}
					let mut at_least_one = false;
					let mut hash1_name_dob_father: Option<H256> = None;
					let mut hash2_name_dob_mother: Option<H256> = None;
					let mut hash3_name_dob_guardian: Option<H256> = None;
					// discard the empty submitted fields
					if split_vec[3] != keccak_256(b"") {
						hash1_name_dob_father = Some(H256::from_slice(split_vec[3]));
						at_least_one = true;
					}
					if split_vec[4] != keccak_256(b"") {
						hash2_name_dob_mother = Some(H256::from_slice(split_vec[4]));
						at_least_one = true;
					}
					if split_vec[5] != keccak_256(b"") {
						hash3_name_dob_guardian = Some(H256::from_slice(split_vec[5]));
						at_least_one = true;
					}

					if at_least_one == false {
						// all should not be blank(invalid char)
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
						hash1_name_dob_father,
						hash2_name_dob_mother,
						hash3_name_dob_guardian,
					};
					return Ok(RevealedParameters::Accept(consumer_details))
				},
				_ => return Err(Error::<T>::InvalidRevealedData.into()),
			}
		}
		pub(crate) fn check_insert_hashes(
			hashes: Vec<H256>,
			consumer_id: T::AccountId,
			current_block: T::BlockNumber,
		) -> Result<(), Error<T>> {
			// check if hashes have be claimed already there
			for hash in hashes.iter() {
				ensure!(
					!ConsumerHashes::<T>::contains_key(&hash),
					Error::<T>::HashAlreadyRegistered
				);
			}
			//insert the hash into record
			for hash in hashes.iter() {
				ConsumerHashes::<T>::insert(&hash, (consumer_id.clone(), current_block));
			}
			Ok(())
		}
		// checkes if id_type is whitelisted and returns the Country and total number of whitelisted
		// ID Documents for that country
		pub(crate) fn validate_id_type(
			id_type: &IdDocumentOf<T>,
		) -> Result<(Country, usize), Error<T>> {
			let IdType { country, .. } = &id_type;
			let whitelisted_id_types = Self::whitelisted_id_types(country);
			ensure!(whitelisted_id_types.contains(&id_type), Error::<T>::IdTypeNotDefined);
			Ok((country.to_owned(), whitelisted_id_types.len()))
		}
	}
}
