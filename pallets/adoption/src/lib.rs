#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		// debug,
		inherent::Vec,
		dispatch::DispatchResult, 
		sp_runtime::{
			SaturatedConversion, 
			// print
		},
		traits::{
			// tokens::ExistenceRequirement, 
			Currency},
		pallet_prelude::*
	};
	// use runtime_io::{ self };
	use frame_system::pallet_prelude::*;
	use core::convert::TryInto;
	use scale_info::{
		TypeInfo, 
		// prelude::vec,
	};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The minimum length an id may be.
        #[pallet::constant]
        type MinLength: Get<u32>;
        /// The maximum length an id may be.
        #[pallet::constant]
        type MaxLength: Get<u32>;

		/// The minimum length a cid may be // 32.
        #[pallet::constant]
        type MinCIDLength: Get<u32>;
		/// The maximum length a cid may be.
        #[pallet::constant]
        type MaxCIDLength: Get<u32>;
		/// The Currency handler for the pallet.
        type Currency: Currency<Self::AccountId>;
		

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
    #[pallet::getter(fn members)]
    /// Keeps track of members.
    pub(super) type Members<T: Config> = StorageMap< _, Twox64Concat, T::AccountId, u128, ValueQuery,>;

	// The pallet's adoption partner storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
    #[pallet::getter(fn partners)]
    /// Keeps track of registered partners.
    pub(super) type Partners<T: Config> = StorageMap< _, Blake2_128Concat, BoundedVec<u8, T::MaxLength>, (BoundedVec<u8, T::MaxCIDLength>, T::BlockNumber),>;

	
	// The pallet's adoption event types storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
    #[pallet::getter(fn adoption_event_types)]
    /// Keeps track of event types.
    pub(super) type AdoptionEventTypes<T: Config> = StorageMap< _, Blake2_128Concat, BoundedVec<u8, T::MaxLength>, (BoundedVec<u8, T::MaxCIDLength>, T::BlockNumber),>;

	// The pallet's adoption event data storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
    #[pallet::getter(fn adoption_event_data_records)]
    /// Keeps track of adoptions events.
    pub(super) type AdoptionEventDataRecords<T: Config> = StorageMap<
	 _, 
	 Blake2_128Concat, 
	 BoundedVec<u8, T::MaxLength>,
	 (AdoptionEventData<T>, T::BlockNumber),
	 >;

	// The pallet's adoption event data storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
    #[pallet::getter(fn adoption_event_participants)]
    /// Keeps track of event_id to list of AccountIds.
    pub(super) type AdoptionEventParticipants<T: Config> = StorageDoubleMap<
	 _, 
	 Blake2_128Concat, 
	 BoundedVec<u8, T::MaxLength>,
	 Blake2_128Concat, 
	 T::AccountId,
	 u128,
	 ValueQuery,
	 >;


  
	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a new member is added.
		NewMemberAdded { member: T::AccountId },
		/// Event emitted when a member is removed.
		MemberRemoved { member: T::AccountId },
		///New partner addition event
		NewPartnerAdded { id: Vec<u8>, cid: Vec<u8> },
		///New adoption event type added
		NewEventTypeAdded { etype: Vec<u8>, details: Vec<u8> },
		///New adoption event added
		NewEventAdded { event_id: Vec<u8>, },
		///new participant added to an event
		NewParticipantAdded { event_id: Vec<u8>, member: T::AccountId},
		/// token minted to an account
		MintedToAccountID{ member:T::AccountId, amount: u128},

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// The member already exists.
		AlreadyAdded,
		ParticipantAlreadyAdded,
		/// Member does not exist so can not be removed
		NoSuchMember,
		///  Id longer than expected //10
		IdTooLong,
		/// Id shorter than expected //5
		IdTooShort,
		///error for CID lengths
		CidTooLong,
		CidTooShort,
		/// error if special chars exists
		OnlyAlhpaNumericAccepted,
		///non existing ids
		///error if type of event does not exists
		EventTypeDoesNotExist,
		///error if partner does not exists
		PartnerDoesNotExist,
		///error if adoptoin event id does not exist in storage
		AdoptionEventDoesNotExist,
		/// error if action is not allowed for the event state
		NotAllowedAtThisEventStage,
		///error if event state value is not valid i.e. 0 to 3
		InvalidEventStateValue,
	}

	/// Struct for holding Adoption Event Data information.
	/// event_state: u32,  //0-> created,  1->event-is-live, 2-> event-paused, 3->event-is-over
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct AdoptionEventData<T: Config> {
	  creator: T::AccountId, 
	  //event_id: BoundedVec<u8, T::MaxLength>, //created by user and fed here
	  partner_id: BoundedVec<u8, T::MaxLength>, 
	  event_type_id: BoundedVec<u8, T::MaxLength>, 
	  value: u128,
	  partner_wallet_id: T::AccountId,
	  referrer_id: BoundedVec<u8, T::MaxLength>,  //referrer partner id
	  details: BoundedVec<u8, T::MaxCIDLength>, //
	  event_state: u32,  //0-> created,  1->event-is-live, 2-> event-paused, 3->event-is-over
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> 
	//   where &<T as frame_system::Config>::AccountId: PartialEq<<T as frame_system::Config>::AccountId>
	{
		/// A dispatchable that takes following parameters  writes the values to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		///	1. partner_id : Max 10 (alphanumeric)   
        /// 2. partner_info_cid: CID( info-json )
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn partner_registration(
			origin: OriginFor<T>, 
			partner_id: Vec<u8>, 
			partner_info_cid:  Vec<u8>, //BoundedVec<u8, T::MaxCIDLength>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let _sender = ensure_signed(origin)?;
			// // check if the user hat root privileges 
			// ensure_root(origin)?;

			//check if contains only alpha numeric
			ensure!(partner_id.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::OnlyAlhpaNumericAccepted);
			ensure!(partner_info_cid.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::OnlyAlhpaNumericAccepted);

			
			
			// //ensure the length of the id is proper
			let bounded_pid: BoundedVec<u8, T::MaxLength> = partner_id.try_into().map_err(|_| Error::<T>::IdTooLong)?;
			// let bounded_pid: BoundedVec<_, _> =
	      	// 		partner_id.try_into().map_err(|()| Error::<T>::IdTooLong)?;
	        ensure!(bounded_pid.len() >= T::MinLength::get() as usize, Error::<T>::IdTooShort);

			//ensure the length of the cid is proper
			let bounded_cid: BoundedVec<_, _> =	partner_info_cid.try_into().map_err(|_| Error::<T>::CidTooLong)?;
	        ensure!(bounded_cid.len() >= T::MinCIDLength::get() as usize, Error::<T>::CidTooShort);
		
			// Verify that the specified partner has not already been added.
			ensure!(!Partners::<T>::contains_key(&bounded_pid), Error::<T>::AlreadyAdded);

			// Get the block number from the FRAME System pallet.
			let current_block = <frame_system::Pallet<T>>::block_number();
		
			// Add the partner .
			Partners::<T>::insert(&bounded_pid, (&bounded_cid, current_block));
		
			// Emit an event that a new member has been added.
			Self::deposit_event(Event::NewPartnerAdded { id: bounded_pid.to_vec(), cid: bounded_cid.to_vec() });
		
			Ok(())
		  }
		/// A dispatchable that takes following parameters  and writes the values to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		///	1. event_type_id : Max 10 (alphanumeric)   
        /// 2. details: CID( info-json )
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn adoption_event_types_creation(
			origin: OriginFor<T>, 
			event_type_id: Vec<u8>, 
			details: Vec<u8>, //BoundedVec<u8, T::MaxCIDLength>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let _sender = ensure_signed(origin)?;
			// // check if the user hat root privileges 
			// ensure_root(origin)?;

			//check if contains only alpha numeric
			ensure!(event_type_id.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::OnlyAlhpaNumericAccepted);
			ensure!(details.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::OnlyAlhpaNumericAccepted);

			//ensure the length of the type is proper
			let bounded_event_type: BoundedVec<_, _> =
	      			event_type_id.try_into().map_err(|_| Error::<T>::IdTooLong)?;
	        ensure!(bounded_event_type.len() >= T::MinLength::get() as usize, Error::<T>::IdTooShort);

			//ensure the length of the remarks is proper
			let bounded_event_details: BoundedVec<_, _> =
			         details.try_into().map_err(|_| Error::<T>::CidTooLong)?;
	        ensure!(bounded_event_details.len() >= 1 as usize, Error::<T>::CidTooShort);

		
			// Verify that the specified event type has not already been added.
			ensure!(!AdoptionEventTypes::<T>::contains_key(&bounded_event_type), Error::<T>::AlreadyAdded);

			// Get the block number from the FRAME System pallet.
			let current_block = <frame_system::Pallet<T>>::block_number();
		
			// Add the event type .
			AdoptionEventTypes::<T>::insert(&bounded_event_type, (&bounded_event_details, current_block));
		
			// Emit an event that a new vent type has been added.
			Self::deposit_event(Event::NewEventTypeAdded { etype: bounded_event_type.to_vec(), details: bounded_event_details.to_vec() });
		
			Ok(())
		}

		/// A dispatchable that takes following parameters  and writes the values to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		/// 1. event_id: new event id
		/// 2. partner_id: id of pre-created partner, 
		/// 3. event_type_id: id of pre-created event type, 
		/// 4. value: number of token to be minted to each participants,
		/// 5. partner_wallet_id: account of partner,
		/// 6. referrer_id: Vec<u8>,  //referrer partner id
		/// 7. details: Vec<u8>, //BoundedVec<u8, T::MaxCIDLength>,
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn adoption_event_creation(
			origin: OriginFor<T>, 
			event_id: Vec<u8>, //created by user and fed here
			partner_id: Vec<u8>, 
			event_type_id: Vec<u8>, 
			value: u128,
			partner_wallet_id: T::AccountId,
			referrer_id: Vec<u8>,  //referrer partner id
			details: Vec<u8>, //BoundedVec<u8, T::MaxCIDLength>, //CID
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let _sender = ensure_signed(origin)?;
			// // check if the user hat root privileges 
			// ensure_root(origin)?;

			//check if contains only alpha numeric in vec input field
			ensure!(event_id.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::OnlyAlhpaNumericAccepted);
			ensure!(partner_id.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::OnlyAlhpaNumericAccepted);
			ensure!(event_type_id.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::OnlyAlhpaNumericAccepted);
			ensure!(referrer_id.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::OnlyAlhpaNumericAccepted);
			ensure!(details.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::OnlyAlhpaNumericAccepted);

			//check if partner_wallet_id is related to the partner
			//--snip--

			//ensure the length of the event id is proper
			let bounded_referrer_id: BoundedVec<_, _> =
				referrer_id.try_into().map_err(|_| Error::<T>::IdTooLong)?;
			ensure!(bounded_referrer_id.len() >= T::MinLength::get() as usize, Error::<T>::IdTooShort);
			//ensure the length of the event id is proper
			let bounded_event_id: BoundedVec<_, _> =
				event_id.try_into().map_err(|_| Error::<T>::IdTooLong)?;
			ensure!(bounded_event_id.len() >= T::MinLength::get() as usize, Error::<T>::IdTooShort);
			//ensure the length of the type is proper
			let bounded_event_type: BoundedVec<_, _> =
	      			event_type_id.try_into().map_err(|_| Error::<T>::IdTooLong)?;
	        ensure!(bounded_event_type.len() >= T::MinLength::get() as usize, Error::<T>::IdTooShort);
			//ensure the length of the remarks is proper
			let bounded_event_details: BoundedVec<_, _> =
			         details.try_into().map_err(|_| Error::<T>::CidTooLong)?;
	        ensure!(bounded_event_details.len() >= 1 as usize, Error::<T>::CidTooShort);
			//ensure the length of the id is proper
			let bounded_pid: BoundedVec<_, _> =
	      			partner_id.try_into().map_err(|_| Error::<T>::IdTooLong)?;
	        ensure!(bounded_pid.len() >= T::MinLength::get() as usize, Error::<T>::IdTooShort);
		
			// Verify that the specified event id has not already been added.
			ensure!(!AdoptionEventDataRecords::<T>::contains_key(&bounded_event_id), Error::<T>::AlreadyAdded);
			// Verify that the specified partner id is valid.
			ensure!(Partners::<T>::contains_key(&bounded_pid), Error::<T>::PartnerDoesNotExist);
			// Verify that the specified event type id is valid.
			ensure!(AdoptionEventTypes::<T>::contains_key(&bounded_event_type), Error::<T>::EventTypeDoesNotExist);
			// Verify that the specified referrer is one of the partner.
			//need to change if the referrer can be any other type
			ensure!(Partners::<T>::contains_key(&bounded_referrer_id), Error::<T>::PartnerDoesNotExist);

			// Get the block number from the FRAME System pallet.
			let current_block = <frame_system::Pallet<T>>::block_number();

			//create event data
			let event_data: AdoptionEventData<T> = AdoptionEventData::<T> {
				creator: _sender, 
	            // event_id: BoundedVec<u8, T::MaxLength>, //created by user and fed here
	            partner_id: bounded_pid, 
	            event_type_id: bounded_event_type, 
	            value: value,
	            partner_wallet_id: partner_wallet_id,
	            referrer_id: bounded_referrer_id,  //referrer partner id
	            details:bounded_event_details, //
				event_state: 0,
				};
		
			// Add the event type .
			AdoptionEventDataRecords::<T>::insert(&bounded_event_id, (&event_data, current_block));
		
			// Emit an event that a new vent type has been added.
			Self::deposit_event(Event::NewEventAdded { event_id: bounded_event_id.to_vec(), });
		
			Ok(())
	    }

		/// A dispatchable that takes following parameters  and writes the values to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		///	1. event_id : Max 10 (alphanumeric)  id of the event  
        /// 2. participant: account id of the participant
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn adoption_event_participant_addition(
			origin: OriginFor<T>, 
			participant: T::AccountId,
			event_id: Vec<u8>, 
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let _sender = ensure_signed(origin)?;
			// // check if the user hat root privileges 
			// ensure_root(origin)?;

			//check if contains only alpha numeric in vec input field
			ensure!(event_id.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::OnlyAlhpaNumericAccepted);

			//ensure the length of the event id is proper
			let bounded_event_id: BoundedVec<_, _> =
				event_id.try_into().map_err(|_| Error::<T>::IdTooLong)?;
			ensure!(bounded_event_id.len() >= T::MinLength::get() as usize, Error::<T>::IdTooShort);

			// Verify that the specified event exists.
			ensure!(AdoptionEventDataRecords::<T>::contains_key(&bounded_event_id), Error::<T>::AdoptionEventDoesNotExist);

			//get the value from event record
			//AdoptionEventDataRecords::<T>::insert(&bounded_event_id, (&event_data, current_block));
            let event_data = <AdoptionEventDataRecords<T>>::get(&bounded_event_id).unwrap().0;
			//check if adding participants allowed in the state of event
			ensure!(event_data.event_state ==1, Error::<T>::NotAllowedAtThisEventStage);

			// Verify that the specified participant has not already been added into a event.
			ensure!(!AdoptionEventParticipants::<T>::contains_key(&bounded_event_id,  &participant), Error::<T>::ParticipantAlreadyAdded);

			// Add the new participant.
			AdoptionEventParticipants::<T>::insert(&bounded_event_id, &participant, 1);

			// Emit an event that a new vent type has been added.
			Self::deposit_event(Event::NewParticipantAdded { event_id: bounded_event_id.to_vec(), member: participant});
			Ok(())
		}

		/// A dispatchable that takes following parameters  and writes the values to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		///	1. event_id : Max 10 (alphanumeric)  id of the event  
        /// 2. participant: account id of the participant
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn mint_event_participants(
			origin: OriginFor<T>, 
			event_id: Vec<u8>, //created by user and fed here
			// participant: T::AccountId,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let _sender = ensure_signed(origin)?;
			// // check if the user hat root privileges 
			// ensure_root(origin)?;

			//check if contains only alpha numeric in vec input field
			ensure!(event_id.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::OnlyAlhpaNumericAccepted);

			//ensure the length of the event id is proper
			let bounded_event_id: BoundedVec<_, _> =
				event_id.try_into().map_err(|_| Error::<T>::IdTooLong)?;
			ensure!(bounded_event_id.len() >= T::MinLength::get() as usize, Error::<T>::IdTooShort);

			// Verify that the specified event exists.
			ensure!(AdoptionEventDataRecords::<T>::contains_key(&bounded_event_id), Error::<T>::AdoptionEventDoesNotExist);

			//get the value from event record
			//AdoptionEventDataRecords::<T>::insert(&bounded_event_id, (&event_data, current_block));
            let event_data = <AdoptionEventDataRecords<T>>::get(&bounded_event_id).unwrap().0;
			let mint_value = event_data.value;
			//check if minting allowed in the state of event
			ensure!(event_data.event_state > 0, Error::<T>::NotAllowedAtThisEventStage);

			let participants = <AdoptionEventParticipants<T>>::iter_prefix(&bounded_event_id);// PrefixIterator<(Key2, Value)>

			for p in participants {
				if p.1 == 1 { //1-> to be minted
					// print("- minting");
					let amount = T::Currency::issue(mint_value.saturated_into()) ;
					//mint into participants account
					T::Currency::resolve_creating(&p.0, amount);
					//update the mint status of the participant
					AdoptionEventParticipants::<T>::mutate(&bounded_event_id, &p.0, |x| *x=0); //set the value to 0->not-be-minted
					// Emit mint amount.
					Self::deposit_event(Event::MintedToAccountID{ member:p.0, amount: mint_value});
				}
			}

			Ok(())
		}

		/// A dispatchable that takes following parameters  and writes the values to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		///	1. event_id : Max 10 (alphanumeric)  id of the event  
        /// 2. event_state: u32,  //0-> created,  1->event-is-live, 2-> event-paused, 3->event-is-over
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn event_set_state(
			origin: OriginFor<T>, 
			event_id: Vec<u8>, //created by user and fed here
			event_state: u32,  //0-> created,  1->event-is-live, 2-> event-paused, 3->event-is-over
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let _sender = ensure_signed(origin)?;
			// // check if the user hat root privileges 
			// ensure_root(origin)?;

			//check if contains only alpha numeric in vec input field
			ensure!(event_id.iter().all(|&x| x.is_ascii_alphanumeric()), Error::<T>::OnlyAlhpaNumericAccepted);

			//ensure the length of the event id is proper
			let bounded_event_id: BoundedVec<_, _> =
				event_id.try_into().map_err(|_| Error::<T>::IdTooLong)?;
			ensure!(bounded_event_id.len() >= T::MinLength::get() as usize, Error::<T>::IdTooShort);
		
			// Verify that the specified event exists.
			ensure!(AdoptionEventDataRecords::<T>::contains_key(&bounded_event_id), Error::<T>::AdoptionEventDoesNotExist);

			//verify that the input event state is valid
			ensure!(event_state <=3, Error::<T>::InvalidEventStateValue);

			//set the event state
			//AdoptionEventDataRecords::<T>::insert(&bounded_event_id, (&event_data, current_block));
			AdoptionEventDataRecords::<T>::mutate(&bounded_event_id, | e| {
				if let Some((ref mut y, _)) = e {
					y.event_state= event_state;
				}
			});	
			Ok(())
		}
	}
}
