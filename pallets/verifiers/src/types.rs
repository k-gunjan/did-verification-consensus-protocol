use codec::{Decode, Encode};

use scale_info::TypeInfo;

use frame_support::inherent::Vec;
use sp_core::MaxEncodedLen;
use sp_runtime::{
	traits::{Bounded, CheckedAdd, CheckedDiv, CheckedSub},
	FixedI64,
};

#[derive(Clone, Debug)]
pub enum Increment {
	Accepted(u8),
	UnAccepted(u8),
	NotCompleted(u8),
}

#[derive(Debug)]
pub struct VerifierUpdateData {
	// account_id: A,
	pub incentive_factor: f64,
	pub increment: Increment,
}

/// Verifier agents struct
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Verifier<AccountId, BlockNumber, Balance> {
	pub account_id: AccountId,
	pub balance: Balance,
	pub selection_score: u128,
	pub state: VerifierState,
	pub count_of_accepted_submissions: u32,
	pub count_of_un_accepted_submissions: u32,
	pub count_of_incompleted_processes: u32,
	// Thime time when accuracy score went bellow threshold
	pub threshold_breach_at: Option<BlockNumber>,
	pub reputation_score: FixedI64,
}

// enum to hold the state of the verifiers
#[derive(Default, Clone, Encode, Decode, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub enum VerifierState {
	Active,
	InActive,
	Submitted,
	#[default]
	Pending,
	Deactivated,
	Other,
}

impl<AccountId, BlockNumber, Balance> Verifier<AccountId, BlockNumber, Balance> {
	pub fn is_active(&self) -> bool {
		matches!(self.state, VerifierState::Active)
	}

	pub fn accuracy(&self) -> FixedI64 {
		let count_of_accepted_submissions = FixedI64::from_u32(self.count_of_accepted_submissions);
		let count_of_un_accepted_submissions =
			FixedI64::from_u32(self.count_of_un_accepted_submissions);
		let nominator = count_of_accepted_submissions
			.checked_sub(&count_of_un_accepted_submissions)
			.unwrap_or_else(|| FixedI64::min_value());

		let denominator = count_of_accepted_submissions
			.checked_add(&count_of_un_accepted_submissions)
			.unwrap_or_else(|| FixedI64::min_value());

		let result = nominator.checked_div(&denominator).unwrap_or_else(|| FixedI64::min_value());

		result
	}

	// Evaluate selection score of a verifier by taking into account the accuracy and reputation
	// score
	pub fn selection_score(self) -> FixedI64 {
		//update accuracy score
		let accuracy = self.accuracy();
		let selection_score = accuracy + self.reputation_score;
		//TODO implement other weights
		selection_score
	}
}

pub trait VerifierOperations<AccountId, BlockNumber, Balance> {
	fn sort_on_selection_score(
		list_of_verifiers: Vec<Verifier<AccountId, BlockNumber, Balance>>,
	) -> Vec<Verifier<AccountId, BlockNumber, Balance>> {
		list_of_verifiers
	}
}

/// Struct of verification protocol parameters
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ProtocolParameterValues {
	pub minimum_deposite_for_being_active: u128,
	pub threshold_accuracy_score: FixedI64,
	pub penalty_waiver_score: FixedI64,
	pub resumption_waiting_period: u32,
	pub reward_amount: u128,
	pub penalty_amount: u128,
	pub penalty_amount_not_completed: u128,
	pub accuracy_weight: FixedI64,
	pub reputation_score_weight: FixedI64,
}

impl Default for ProtocolParameterValues {
	fn default() -> Self {
		ProtocolParameterValues {
			minimum_deposite_for_being_active: 100_000_000_000_000,
			threshold_accuracy_score: FixedI64::from_inner(85),
			penalty_waiver_score: FixedI64::from_inner(95),
			// resemption period in number of blocks
			resumption_waiting_period: 100,
			reward_amount: 1_000_000_000_000,
			penalty_amount: 1_000_000_000_000,
			penalty_amount_not_completed: 5_000_000_000_000,
			accuracy_weight: FixedI64::from_inner(1),
			reputation_score_weight: FixedI64::from_inner(1),
		}
	}
}
