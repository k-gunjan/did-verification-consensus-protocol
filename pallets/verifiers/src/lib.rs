#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

// ///import the pallet verification protocol
// pub use pallet_verification_protocol;
pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

	use crate::types::*;
	use frame_support::sp_runtime::SaturatedConversion;
	use frame_support::{
		// inherent::Vec,
		pallet_prelude::*,
		sp_runtime::traits::AccountIdConversion,
		traits::{Currency, ExistenceRequirement},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	// use pallet_verification_protocol::types::VerifierUpdateData;
	use sp_runtime::{
		traits::{CheckedAdd, Zero},
		ArithmeticError,
	};
	use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub struct UpdateData {
		pub incr_accepted_submissions: Option<u32>,
		pub incr_un_accepted_submissions: Option<u32>,
		pub incr_incompleted_processes: Option<u32>,
	}
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The Currency handler for the pallet.
		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// storage to hold the list of verifiers
	#[pallet::storage]
	#[pallet::getter(fn verifiers)]
	pub type Verifiers<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Verifier<T::AccountId, T::BlockNumber, BalanceOf<T>>,
		OptionQuery,
	>;

	// Store the protocol parameters
	#[pallet::storage]
	#[pallet::getter(fn protocol_parameters)]
	pub type ProtocolParameters<T> = StorageValue<_, ProtocolParameterValues, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// parameters. [verifier_account_id]
		VerifierRegistrationRequest(T::AccountId),
		/// parameters. [verifier_account_id, amount]
		VerifierDeposite(T::AccountId, BalanceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		VerifierAlreadyRegistered,
		VerifierNotRegistered,
		InvalidDepositeAmount,
		/// Erron in pallet_account_id  generation
		PalletAccountIdFailure,
		/// Error in updating value
		ArithmeticOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a verifier. Takes following parameters
		/// 1. deposit amount
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn register_verifier(origin: OriginFor<T>, deposit: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// check if the verifier is already registered
			ensure!(
				!<Verifiers<T>>::contains_key(who.clone()),
				Error::<T>::VerifierAlreadyRegistered
			);
			// Check that the deposited value is greater than zero.
			ensure!(deposit > Zero::zero(), Error::<T>::InvalidDepositeAmount);

			let pallet_account: T::AccountId = Self::pallet_account_id()?;
			T::Currency::transfer(&who, &pallet_account, deposit, ExistenceRequirement::KeepAlive)?;
			let minimum_deposite_for_being_active: BalanceOf<T> = ProtocolParameters::<T>::get()
				.minimum_deposite_for_being_active
				.saturated_into::<BalanceOf<T>>();

			let state = if deposit >= minimum_deposite_for_being_active {
				VerifierState::Active
			} else {
				VerifierState::Pending
			};
			let verifier = Verifier {
				account_id: who.clone(),
				balance: deposit,
				selection_score: 0,
				state,
				count_of_accepted_submissions: 0u32,
				count_of_un_accepted_submissions: 0u32,
				count_of_incompleted_processes: 0u32,
				threshold_breach_time: None.into(),
				reputation_score: 0,
			};

			// Update Verifiers storage.
			<Verifiers<T>>::insert(who.clone(), verifier.clone());

			// Emit an event.
			Self::deposit_event(Event::VerifierRegistrationRequest(who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn verifier_deposite(origin: OriginFor<T>, deposit: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// check if the verifier is already registered
			ensure!(<Verifiers<T>>::contains_key(who.clone()), Error::<T>::VerifierNotRegistered);
			// Check that the deposited value is greater than zero.
			ensure!(deposit > Zero::zero(), Error::<T>::InvalidDepositeAmount);

			let minimum_deposite_for_being_active = ProtocolParameters::<T>::get()
				.minimum_deposite_for_being_active
				.saturated_into::<BalanceOf<T>>();

			// update balance and change state if required
			Verifiers::<T>::try_mutate(who.clone(), |v| -> DispatchResult {
				if let Some(ref mut verifier) = v {
					let pallet_account: T::AccountId = Self::pallet_account_id()?;
					T::Currency::transfer(
						&who,
						&pallet_account,
						deposit,
						ExistenceRequirement::KeepAlive,
					)?;

					verifier.balance =
						verifier.balance.checked_add(&deposit).ok_or(ArithmeticError::Overflow)?;
					if verifier.balance >= minimum_deposite_for_being_active {
						verifier.state = VerifierState::Active;
					}
				}
				Ok(())
			})?;

			// Emit an event.
			Self::deposit_event(Event::VerifierDeposite(who, deposit));
			// Return a successful DispatchResult
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn sub_account_id(id: T::AccountId) -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(id)
		}

		pub(crate) fn pallet_account_id() -> Result<T::AccountId, Error<T>> {
			if let Some(account) = T::PalletId::get().try_into_account() {
				Ok(account)
			} else {
				Err(Error::<T>::PalletAccountIdFailure.into())
			}
		}

		pub(crate) fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub(crate) fn update_profile(
			who: T::AccountId,
			update_data: UpdateData,
		) -> Result<(), ArithmeticError> {
			Verifiers::<T>::try_mutate(who, |v| -> Result<(), ArithmeticError> {
				if let Some(ref mut verifier) = v {
					if let Some(accepted_count) = update_data.incr_accepted_submissions {
						verifier.count_of_accepted_submissions = verifier
							.count_of_accepted_submissions
							.checked_add(accepted_count)
							.ok_or(ArithmeticError::Overflow)?;
					}
					if let Some(unaccepted_count) = update_data.incr_un_accepted_submissions {
						verifier.count_of_un_accepted_submissions = verifier
							.count_of_un_accepted_submissions
							.checked_add(unaccepted_count)
							.ok_or(ArithmeticError::Overflow)?;
					}
					if let Some(incompleted_count) = update_data.incr_incompleted_processes {
						verifier.count_of_incompleted_processes = verifier
							.count_of_incompleted_processes
							.checked_add(incompleted_count)
							.ok_or(ArithmeticError::Overflow)?;
					}
				}
				Ok(())
			})?;
			Ok(())
		}
	}

	pub trait VerifiersProvider {
		type AccountId;
		type UpdateData;

		fn get_verifiers() -> Vec<Self::AccountId>;
		fn update_verifier_profiles(
			data: Vec<(Self::AccountId, Self::UpdateData)>,
		) -> Result<(), ArithmeticError>;
	}

	impl<T: Config> VerifiersProvider for Pallet<T> {
		type AccountId = T::AccountId;
		type UpdateData = VerifierUpdateData;

		fn get_verifiers() -> Vec<Self::AccountId> {
			let verifiers: Vec<T::AccountId> = Verifiers::<T>::iter_values()
				.filter(|v| v.state == VerifierState::Active)
				.map(|v| v.account_id)
				.collect();
			verifiers
		}
		fn update_verifier_profiles(
			data: Vec<(Self::AccountId, Self::UpdateData)>,
		) -> Result<(), ArithmeticError> {
			for (who, update_data) in data.iter() {
				Verifiers::<T>::try_mutate(who, |v| -> Result<(), ArithmeticError> {
					if let Some(ref mut verifier) = v {
						match update_data.increment {
							Increment::Accepted(n) => {
								verifier.count_of_accepted_submissions = verifier
									.count_of_accepted_submissions
									.checked_add(n.into())
									.ok_or(ArithmeticError::Overflow)?;
							},
							Increment::UnAccepted(n) => {
								verifier.count_of_un_accepted_submissions = verifier
									.count_of_un_accepted_submissions
									.checked_add(n.into())
									.ok_or(ArithmeticError::Overflow)?;
							},
							Increment::NotCompleted(n) => {
								verifier.count_of_incompleted_processes = verifier
									.count_of_incompleted_processes
									.checked_add(n.into())
									.ok_or(ArithmeticError::Overflow)?;
							},
						}
					} else {
						log::info!("+++++++++++++verifier not found+++++++++++++++++");
					}
					Ok(())
				})?;
			}
			Ok(())
		}
	}
}
