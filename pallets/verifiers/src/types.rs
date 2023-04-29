use codec::{Decode, Encode};

use scale_info::TypeInfo;

use frame_support::inherent::Vec;
use sp_core::MaxEncodedLen;
use sp_runtime::{traits::Zero, ArithmeticError};

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
	pub threshold_breach_time: Option<BlockNumber>,
	pub reputation_score: u128,
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

	pub fn accuracy_weight(&self) -> Result<f64, ArithmeticError> {
		let result = (self.count_of_accepted_submissions - self.count_of_un_accepted_submissions) /
			(self.count_of_accepted_submissions + self.count_of_un_accepted_submissions);

		Ok(result.into())
	}

	pub fn selection_score(self) -> f64 {
		//update accuracy score
		let accuracy = match self.accuracy_weight() {
			Ok(f) => f,
			Err(e) => {
				log::info!("error in accuracy:{e:?}");
				Zero::zero()
			},
		};
		accuracy
		//TODO implement other weights
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
	pub threshold_accuracy_score: u32,
	pub penalty_waiver_score: u32,
	pub resumption_waiting_period: u32,
	pub decimals: u8,
}

impl Default for ProtocolParameterValues {
	fn default() -> Self {
		ProtocolParameterValues {
			minimum_deposite_for_being_active: 1000_000_000_000,
			threshold_accuracy_score: 850000000,
			penalty_waiver_score: 950000000,
			resumption_waiting_period: 100,
			decimals: 9,
		}
	}
}
