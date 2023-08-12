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
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{traits::AccountIdConversion, FixedI64},
		traits::{Currency, ExistenceRequirement},
		PalletId,
	};

	use frame_support::sp_runtime::SaturatedConversion;
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{Bounded, CheckedAdd, CheckedMul, CheckedSub, Zero},
		ArithmeticError, FixedU128,
	};
	use sp_std::vec::Vec;

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

		#[pallet::constant]
		type MaxEligibleVerifiers: Get<u32>;
	}

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

	// storage to hold the list of active eligible verifiers who is ready to take task
	#[pallet::storage]
	#[pallet::getter(fn eligible_verifiers)]
	pub type EligibleVerifiers<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxEligibleVerifiers>, ValueQuery>;

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
		/// Update protocol parameters for stages
		ParametersUpdated(ProtocolParameterValues),
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

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// At block finalization
		fn on_finalize(_now: BlockNumberFor<T>) {
			let last_verifiers = Self::eligible_verifiers().to_vec();

			// Retrieve all verifiers in an active state
			let mut verifiers: Vec<Verifier<T::AccountId, T::BlockNumber, BalanceOf<T>>> =
				Verifiers::<T>::iter_values()
					.filter(|v| v.state == VerifierState::Active)
					.map(|v| v)
					.collect();

			// Create a random seed using the verifiers' accuracy score and index
			let accuracy_scores: Vec<(FixedI64, usize)> = verifiers
			.iter()
			.enumerate()
			.map(|(index, v)| (v.accuracy(), index))
			.collect();

			// Shuffle verifiers using the accuracy scores as the random seed
			Self::shuffle_verifiers(&mut verifiers, &accuracy_scores);

			// Calculate verifiers count to swap
			let percentage = (verifiers.len() as f64 * 0.09) as usize;

			// 9%~ shuffled verifiers subset
			let shuffled_subset = verifiers.iter().take(percentage + 1).cloned().collect::<Vec<_>>();

			// sort by accuracy of the verifiers
			verifiers.sort_by_key(|k| k.accuracy());

			// Iterate through the sorted verifiers and swap shuffled_subset verifiers
			for i in 0..percentage {
				if let Some(index) = verifiers.iter().position(|v| v.account_id == shuffled_subset[i].account_id) {
					if i + 1 < percentage {
						let next_verifier = &shuffled_subset[i + 1];
						if let Some(next_index) = verifiers.iter().position(|v| v.account_id == next_verifier.account_id) {
							verifiers.swap(index, next_index);
						}
					}
				}
			}

			// Create a new list of account IDs based on the shuffled verifiers
			let new_verifiers: Vec<T::AccountId> =
				verifiers.iter().map(|v| v.account_id.clone()).collect();

			if last_verifiers != new_verifiers {
				if new_verifiers.len() as u32 > T::MaxEligibleVerifiers::get() {
					log::warn!(
						"Next verifiers list larger than {}, truncating",
						T::MaxEligibleVerifiers::get(),
					);
				}

				// Truncate the list to fit within the maximum eligible verifiers limit
				let bounded =
					<BoundedVec<_, T::MaxEligibleVerifiers>>::truncate_from(new_verifiers);

				// Store the bounded list of eligible verifiers in storage
				EligibleVerifiers::<T>::put(bounded);
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a verifier. Takes following parameters
		/// 1. deposit amount
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(0)]
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
			let minimum_deposit_for_being_active: BalanceOf<T> = Self::protocol_parameters()
				.minimum_deposit_for_being_active
				.saturated_into::<BalanceOf<T>>();

			let state = if deposit >= minimum_deposit_for_being_active {
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
				threshold_breach_at: None.into(),
				reputation_score: sp_runtime::FixedI64::min_value(),
			};

			// Update Verifiers storage.
			<Verifiers<T>>::insert(who.clone(), verifier.clone());

			// Emit an event.
			Self::deposit_event(Event::VerifierRegistrationRequest(who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(1)]
		pub fn verifier_deposit(origin: OriginFor<T>, deposit: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// check if the verifier is already registered
			ensure!(<Verifiers<T>>::contains_key(who.clone()), Error::<T>::VerifierNotRegistered);
			// Check that the deposited value is greater than zero.
			ensure!(deposit > Zero::zero(), Error::<T>::InvalidDepositeAmount);

			let minimum_deposit_for_being_active = ProtocolParameters::<T>::get()
				.minimum_deposit_for_being_active
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
					if verifier.balance >= minimum_deposit_for_being_active {
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

		/// Change protocol parameters
		/// takes new parameters and updates the default value
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(2)]
		pub fn update_protocol_parameters(
			origin: OriginFor<T>,
			new_parameters: ProtocolParameterValues,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			ProtocolParameters::<T>::put(&new_parameters);

			Self::deposit_event(Event::ParametersUpdated(new_parameters));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Shuffles a vector using the accuracy scores as the random seed.
		fn shuffle_verifiers(
			verifiers: &mut Vec<Verifier<T::AccountId, T::BlockNumber, BalanceOf<T>>>,
			accuracy_scores: &[(FixedI64, usize)],
		) {
			let mut random_seed = accuracy_scores.to_vec();

			// Fisher-Yates shuffle using the accuracy scores as the random seed
			let mut i = verifiers.len();
			while i > 1 {
				i -= 1;
				let j = random_seed[i].1 % (i + 1);
				random_seed.swap(i, j);
				verifiers.swap(i, j);
			}
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn _sub_account_id(id: T::AccountId) -> T::AccountId {
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
	}

	pub trait VerifiersProvider {
		type AccountId;
		type UpdateData;
		type BlockNumber;

		fn get_verifiers() -> Vec<Self::AccountId>;
		fn update_verifier_profiles(
			data: Vec<(Self::AccountId, Self::UpdateData)>,
			current_block: Self::BlockNumber,
		) -> Result<(), ArithmeticError>;
	}

	impl<T: Config> VerifiersProvider for Pallet<T> {
		type AccountId = T::AccountId;
		type UpdateData = VerifierUpdateData;
		type BlockNumber = T::BlockNumber;

		fn get_verifiers() -> Vec<Self::AccountId> {
			EligibleVerifiers::<T>::get().to_vec()
		}
		fn update_verifier_profiles(
			data: Vec<(Self::AccountId, Self::UpdateData)>,
			current_block: Self::BlockNumber,
		) -> Result<(), ArithmeticError> {
			let parameters = Self::protocol_parameters();
			for (who, update_data) in data.iter() {
				Verifiers::<T>::try_mutate(who, |v| -> Result<(), ArithmeticError> {
					if let Some(ref mut verifier) = v {
						let accuracy = verifier.accuracy();
						match update_data.increment {
							Increment::Accepted(n) => {
								verifier.count_of_accepted_submissions = verifier
									.count_of_accepted_submissions
									.checked_add(n.into())
									.ok_or(ArithmeticError::Overflow)?;

								// reward only if the accuracy score is equal to or higher than the
								// threshold
								// and
								// the verifier is not serving in the resemption period
								if parameters.threshold_accuracy_score <= accuracy &&
									verifier.threshold_breach_at.is_none()
								{
									// reward*factor + reward
									let amount = FixedU128::from_inner(parameters.reward_amount)
										.checked_mul(&update_data.incentive_factor)
										.ok_or(ArithmeticError::Overflow)?
										.checked_add(&FixedU128::from_inner(
											parameters.reward_amount,
										))
										.ok_or(ArithmeticError::Overflow)?;

									let incentive_amount: BalanceOf<T> =
										amount.into_inner().saturated_into();
									verifier.balance = verifier
										.balance
										.checked_add(&incentive_amount)
										.ok_or(ArithmeticError::Overflow)?;

									let _ = T::Currency::transfer(
										&Self::account_id(),
										&verifier.account_id,
										incentive_amount,
										ExistenceRequirement::KeepAlive,
									);
									// Activate if balance goes above limit and in InActive state
									if verifier.balance >=
										parameters
											.minimum_deposit_for_being_active
											.saturated_into() && verifier.state ==
										VerifierState::InActive
									{
										verifier.state = VerifierState::Active;
									}
								}
								// check if accuracy goes above the threshold and resumption period
								// is over
								if verifier.accuracy() >= parameters.threshold_accuracy_score {
									if let Some(crossed_down_at) = verifier.threshold_breach_at {
										// check if resumption period is over
										if current_block >
											crossed_down_at +
												parameters.resumption_waiting_period.into()
										{
											// record the breach time
											verifier.threshold_breach_at = None;
										}
									}
								}
							},
							Increment::UnAccepted(n) => {
								verifier.count_of_un_accepted_submissions = verifier
									.count_of_un_accepted_submissions
									.checked_add(n.into())
									.ok_or(ArithmeticError::Overflow)?;

								// waive penalty if accuracy is higher than the waiver threshold
								if parameters.penalty_waiver_score > accuracy {
									let penalty_amount =
										FixedU128::from_inner(parameters.penalty_amount)
											.checked_sub(
												&FixedU128::from_inner(parameters.penalty_amount)
													.checked_mul(&update_data.incentive_factor)
													.ok_or(ArithmeticError::Overflow)?,
											)
											.ok_or(ArithmeticError::Overflow)?;

									let incentive_amount: BalanceOf<T> =
										penalty_amount.into_inner().saturated_into();

									verifier.balance = verifier
										.balance
										.checked_sub(&incentive_amount)
										.ok_or(ArithmeticError::Overflow)?;

									// let _ = T::Currency::transfer(
									// 	&verifier.account_id,
									// 	&Self::account_id(),
									// 	incentive_amount,
									// 	ExistenceRequirement::KeepAlive,
									// );

									// InActivate if balance goes bellow limit
									if verifier.balance <
										parameters
											.minimum_deposit_for_being_active
											.saturated_into()
									{
										verifier.state = VerifierState::InActive;
									}
								}
								// check if accuracy goes bellow the threshold
								if verifier.accuracy() < parameters.threshold_accuracy_score &&
									verifier.threshold_breach_at.is_none()
								{
									// let it remain active as per new thought
									// record the breach time
									verifier.threshold_breach_at = Some(current_block);
								}
							},
							Increment::NotCompleted(n) => {
								verifier.count_of_incompleted_processes = verifier
									.count_of_incompleted_processes
									.checked_add(n.into())
									.ok_or(ArithmeticError::Overflow)?;

								// waive penalty if accuracy is higher than the waiver threshold
								if parameters.penalty_waiver_score > accuracy {
									let penalty_amount = FixedU128::from_inner(
										parameters.penalty_amount_not_completed,
									)
									.checked_sub(
										&FixedU128::from_inner(
											parameters.penalty_amount_not_completed,
										)
										.checked_mul(&update_data.incentive_factor)
										.ok_or(ArithmeticError::Overflow)?,
									)
									.ok_or(ArithmeticError::Overflow)?;

									let incentive_amount: BalanceOf<T> =
										penalty_amount.into_inner().saturated_into();

									verifier.balance = verifier
										.balance
										.checked_sub(&incentive_amount)
										.ok_or(ArithmeticError::Overflow)?;

									// let _ = T::Currency::transfer(
									// 	&verifier.account_id,
									// 	&Self::account_id(),
									// 	incentive_amount,
									// 	ExistenceRequirement::KeepAlive,
									// );
									// InActivate if balance goes bellow limit
									if verifier.balance <
										parameters
											.minimum_deposit_for_being_active
											.saturated_into()
									{
										verifier.state = VerifierState::InActive;
									}
								}
								// check if accuracy goes bellow the threshold
								if verifier.accuracy() < parameters.threshold_accuracy_score &&
									verifier.threshold_breach_at.is_none()
								{
									// let it remain active as per new thought
									// record the breach time
									verifier.threshold_breach_at = Some(current_block);
								}
							},
						}
					} else {
						log::error!("+++++++++++++verifier not found+++++++++++++++++");
					}
					Ok(())
				})?;
			}
			Ok(())
		}
	}
}
