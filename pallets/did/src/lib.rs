#![cfg_attr(not(feature = "std"), no_std)]

/// Based on pallet-DID from https://github.com/substrate-developer-hub/pallet-did
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;
pub mod did;
pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use crate::did::Did;
	pub use crate::types::*;

	use frame_support::{
		dispatch::DispatchResult,
		// debug,
		inherent::Vec,
		pallet_prelude::*,
		sp_io::hashing::blake2_256,
	};
	// use runtime_io::{ self };
	pub use frame_support::traits::Time;
	use frame_system::pallet_prelude::*;
	// use sp_io::hashing::blake2_256;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		// // type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
		// type Public: IdentifyAccount<AccountId = Self::AccountId>;
		// type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
		type Time: Time;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Identity delegates stored by type.
	/// Delegates are only valid for a specific period defined as blocks number.
	#[pallet::storage]
	#[pallet::getter(fn delegate_of)]
	pub(super) type DelegateOf<T: Config> = StorageMap<
		//&identity, delegate_type, delegate), &validity)
		_,
		Blake2_128Concat,
		(T::AccountId, Vec<u8>, T::AccountId),
		T::BlockNumber,
	>;

	/// The attributes that belong to an identity.
	#[pallet::storage]
	#[pallet::getter(fn attribute_of)]
	pub(super) type AttributeOf<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, [u8; 32]),
		Attribute<T::BlockNumber, <<T as Config>::Time as Time>::Moment>,
		ValueQuery,
	>;

	/// Attribute nonce used to generate a unique hash even if the attribute is deleted and
	/// recreated.
	#[pallet::storage]
	#[pallet::getter(fn nonce_of)]
	/// Keeps track of adoptions events.
	pub(super) type AttributeNonce<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::AccountId, Vec<u8>), u64, ValueQuery>;

	/// Identity owner.
	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub(super) type OwnerStore<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId>;

	/// Tracking the latest identity update.
	#[pallet::storage]
	#[pallet::getter(fn updated_by)]
	/// Keeps track of adoptions events.
	pub(super) type UpdatedBy<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		(T::AccountId, T::BlockNumber, <<T as Config>::Time as Time>::Moment),
	>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AttributeAdded {
			identity: T::AccountId,
			name: Vec<u8>,
			till: Option<T::BlockNumber>,
		},
		/// Attribute not found
		AttributeNotFound {
			who: T::AccountId,
			identity: T::AccountId,
			name: Vec<u8>,
		},
		/// Attribute fetched
		AttributeFetched {
			who: T::AccountId,
			identity: T::AccountId,
			name: Vec<u8>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotOwner,
		InvalidDelegate,
		BadSignature,
		AttributeCreationFailed,
		AttributeResetFailed,
		AttributeRemovalFailed,
		InvalidAttribute,
		Overflow,
		BadTransaction,
		AttributeNameExceedMax64,
		/// error if creating the existing one
		DuplicateNotNeeded,
		/// error if atritbute is not found
		AttributeNotFound,
	}

	impl<T: Config> Did<T::AccountId, T::BlockNumber, <<T as Config>::Time as Time>::Moment>
		for Pallet<T>
	{
		/// Validates if the AccountId 'actual_owner' owns the identity.
		fn is_owner(identity: &T::AccountId, actual_owner: &T::AccountId) -> DispatchResult {
			let owner = Self::identity_owner(identity);
			match owner == *actual_owner {
				true => Ok(()),
				false => Err(Error::<T>::NotOwner.into()),
			}
		}

		/// Get the identity owner if set.
		/// If never changed, returns the identity as its owner.
		fn identity_owner(identity: &T::AccountId) -> T::AccountId {
			match Self::owner_of(identity) {
				Some(id) => id,
				None => identity.clone(),
			}
		}

		/// Validates if a delegate belongs to an identity and it has not expired.
		fn valid_delegate(
			identity: &T::AccountId,
			delegate_type: &[u8],
			delegate: &T::AccountId,
		) -> DispatchResult {
			ensure!(delegate_type.len() <= 64, Error::<T>::InvalidDelegate);
			ensure!(
				Self::valid_listed_delegate(identity, delegate_type, delegate).is_ok() &&
					Self::is_owner(identity, delegate).is_ok(),
				Error::<T>::InvalidDelegate
			);
			Ok(())
		}

		/// Validates that a delegate contains_key for specific purpose and remains valid at this
		/// block high.
		fn valid_listed_delegate(
			identity: &T::AccountId,
			delegate_type: &[u8],
			delegate: &T::AccountId,
		) -> DispatchResult {
			ensure!(
				<DelegateOf<T>>::contains_key((&identity, delegate_type, &delegate)),
				Error::<T>::InvalidDelegate
			);

			let validity = Self::delegate_of((identity, delegate_type, delegate));
			match validity > Some(<frame_system::Pallet<T>>::block_number()) {
				true => Ok(()),
				false => Err(Error::<T>::InvalidDelegate.into()),
			}
		}

		// Creates a new delegete for an account.
		fn create_delegate(
			who: &T::AccountId,
			identity: &T::AccountId,
			delegate: &T::AccountId,
			delegate_type: &[u8],
			valid_for: Option<T::BlockNumber>,
		) -> DispatchResult {
			Self::is_owner(&identity, who)?;
			ensure!(who != delegate, Error::<T>::InvalidDelegate);
			ensure!(
				!Self::valid_listed_delegate(identity, delegate_type, delegate).is_ok(),
				Error::<T>::InvalidDelegate
			);

			let now_block_number = <frame_system::Pallet<T>>::block_number();
			let validity: T::BlockNumber = match valid_for {
				Some(blocks) => now_block_number + blocks,
				None => u32::max_value().into(),
			};

			<DelegateOf<T>>::insert((&identity, delegate_type, delegate), &validity);
			Ok(())
		}

		/// Adds a new attribute to an identity and colects the storage fee.
		fn create_attribute(
			who: &T::AccountId,
			identity: &T::AccountId,
			name: &[u8],
			value: &[u8],
			valid_for: Option<T::BlockNumber>,
		) -> DispatchResult {
			Self::is_owner(&identity, &who)?;

			if Self::attribute_and_id(identity, name).is_some() {
				Err(Error::<T>::DuplicateNotNeeded.into())
			} else {
				let now_timestamp = T::Time::now();
				let now_block_number = <frame_system::Pallet<T>>::block_number();
				let validity: T::BlockNumber = match valid_for {
					Some(blocks) => now_block_number + blocks,
					None => u32::max_value().into(),
				};

				let mut nonce = Self::nonce_of((&identity, name.to_vec()));
				let id = (&identity, name, nonce).using_encoded(blake2_256);
				let new_attribute = Attribute {
					name: (&name).to_vec(),
					value: (&value).to_vec(),
					validity,
					creation: now_timestamp,
					nonce,
				};

				// Prevent panic overflow
				nonce = nonce.checked_add(1).ok_or(Error::<T>::Overflow)?;
				<AttributeOf<T>>::insert((&identity, &id), new_attribute);
				<AttributeNonce<T>>::mutate((&identity, name.to_vec()), |n| *n = nonce);
				<UpdatedBy<T>>::insert(identity, (who, now_block_number, now_timestamp));
				Ok(())
			}
		}

		/// Updates the attribute validity to make it expire and invalid.
		fn reset_attribute(
			who: T::AccountId,
			identity: &T::AccountId,
			name: &[u8],
		) -> DispatchResult {
			Self::is_owner(&identity, &who)?;
			// If the attribute contains_key, the latest valid block is set to the current block.
			let result = Self::attribute_and_id(identity, name);
			match result {
				Some((mut attribute, id)) => {
					attribute.validity = <frame_system::Pallet<T>>::block_number();
					<AttributeOf<T>>::mutate((&identity, id), |a| *a = attribute);
				},
				None => return Err(Error::<T>::AttributeResetFailed.into()),
			}

			// Keep track of the updates.
			<UpdatedBy<T>>::insert(
				identity,
				(who, <frame_system::Pallet<T>>::block_number(), T::Time::now()),
			);
			Ok(())
		}

		/// Validates if an attribute belongs to an identity and it has not expired.
		fn valid_attribute(identity: &T::AccountId, name: &[u8], value: &[u8]) -> DispatchResult {
			ensure!(name.len() <= 64, Error::<T>::InvalidAttribute);
			let result = Self::attribute_and_id(identity, name);

			let (attr, _) = match result {
				Some((attr, id)) => (attr, id),
				None => return Err(Error::<T>::InvalidAttribute.into()),
			};

			if (attr.validity > (<frame_system::Pallet<T>>::block_number())) &&
				(attr.value == value.to_vec())
			{
				Ok(())
			} else {
				Err(Error::<T>::InvalidAttribute.into())
			}
		}

		/// Returns the attribute and its hash identifier.
		/// Uses a nonce to keep track of identifiers making them unique after attributes deletion.
		fn attribute_and_id(
			identity: &T::AccountId,
			name: &[u8],
		) -> Option<AttributedId<T::BlockNumber, <<T as Config>::Time as Time>::Moment>> {
			let nonce = Self::nonce_of((&identity, name.to_vec()));

			// Used for first time attribute creation
			let lookup_nonce = match nonce {
				0u64 => 0u64,
				_ => nonce - 1u64,
			};

			// Looks up for the existing attribute.
			// Needs to use actual attribute nonce -1.
			let id = (&identity, name, lookup_nonce).using_encoded(blake2_256);

			if <AttributeOf<T>>::contains_key((&identity, &id)) {
				Some((Self::attribute_of((identity, id)), id))
			} else {
				None
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	//   where &<T as frame_system::Config>::AccountId: PartialEq<<T as
	// frame_system::Config>::AccountId>
	{
		/// Creates a new attribute as part of an identity.
		/// Sets its expiration period.
		/// takes following arguments
		/// 1. identity - identity of the user
		/// 2. name - name of the attribute
		/// 3. value - value of the attribute
		/// 4. valid_for - validity of the attribute
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn add_attribute(
			origin: OriginFor<T>,
			identity: T::AccountId,
			name: Vec<u8>,
			value: Vec<u8>,
			valid_for: Option<T::BlockNumber>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(name.len() <= 64, Error::<T>::AttributeNameExceedMax64);

			Self::create_attribute(&who, &identity, &name, &value, valid_for)?;
			// Emit an event that a new id has been added.
			Self::deposit_event(Event::AttributeAdded { identity, name, till: valid_for });
			Ok(())
		}

		///fetch an attribute of an identity
		/// takes following arguments
		/// 1. identity - identity of the user
		/// 2. name - name of the attribute
		/// returns the attribute in the event
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn get_attribute(
			origin: OriginFor<T>,
			identity: T::AccountId,
			name: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(name.len() <= 64, Error::<T>::AttributeNameExceedMax64);
			let result = Self::attribute_and_id(&identity, &name);

			match result {
				Some(_) => {
					//emit event on read of an attribute
					//may be considered to suppress it
					Self::deposit_event(Event::AttributeFetched { who, identity, name });
				},
				None => {
					//raise error
					return Err(Error::<T>::AttributeNotFound.into())
					// Self::deposit_event(Event::AttributeNotFound { who: who, identity: identity,
					// name: name});
				},
			}
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// test the rpc interface with simple call
		pub fn get_a_value(i: u32, j: u32) -> u32 {
			i * i + j // return the value i sqaure + j
		}

		/// read the value of an attribute from the chain
		pub fn read_attribute(
			identity: &T::AccountId,
			name: &[u8],
		) -> Option<Attribute<T::BlockNumber, <<T as Config>::Time as Time>::Moment>> {
			let nonce = Self::nonce_of((&identity, name.to_vec()));

			// Used for first time attribute creation
			let lookup_nonce = match nonce {
				0u64 => 0u64,
				_ => nonce - 1u64,
			};

			// Looks up for the existing attribute.
			// Needs to use actual attribute nonce -1.
			let id = (&identity, name, lookup_nonce).using_encoded(blake2_256);

			if <AttributeOf<T>>::contains_key((&identity, &id)) {
				Some(Self::attribute_of((identity, id)))
			} else {
				None
			}
		}
	}
}
