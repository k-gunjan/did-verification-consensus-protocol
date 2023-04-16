#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

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
	use frame_support::{
		// inherent::Vec,
		pallet_prelude::*,
		sp_runtime::traits::AccountIdConversion,
		traits::{Currency, ExistenceRequirement},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Zero;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The Currency handler for the pallet.
		type Currency: Currency<Self::AccountId>;

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
		Verifier<T::AccountId, T::BlockNumber>,
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

			let pallet_account: T::AccountId = Self::account_id(who.clone());
			T::Currency::transfer(&who, &pallet_account, deposit, ExistenceRequirement::KeepAlive)?;

			let verifier = Verifier {
				account_id: who.clone(),
				score: 0u32,
				state: VerifierState::Pending,
				count_of_accepted_submissions: 0u32,
				count_of_un_accepted_submissions: 0u32,
				count_of_incompleted_processes: 0u32,
				threshold_breach_time: None.into(),
				// accuracy: Zero::zero(),
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

			let pallet_account: T::AccountId = Self::account_id(who.clone());
			T::Currency::transfer(&who, &pallet_account, deposit, ExistenceRequirement::KeepAlive)?;

			// Emit an event.
			Self::deposit_event(Event::VerifierDeposite(who, deposit));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn account_id(id: T::AccountId) -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(id)
		}
	}
}
