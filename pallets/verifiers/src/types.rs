use codec::{Decode, Encode};

use scale_info::TypeInfo;

use frame_support::inherent::Vec;
use sp_core::MaxEncodedLen;
use sp_runtime::{traits::Zero, ArithmeticError};

/// Verifier agents struct
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Verifier<AccountId, BlockNumber> {
	pub account_id: AccountId,
	pub score: u32,
	pub state: VerifierState,
	pub count_of_accepted_submissions: u32,
	pub count_of_un_accepted_submissions: u32,
	pub count_of_incompleted_processes: u32,
	// Thime time when accuracy score went bellow threshold
	pub threshold_breach_time: Option<BlockNumber>,
	// pub accuracy: f64,
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

impl<AccountId, BlockNumber> Verifier<AccountId, BlockNumber> {
	pub fn is_active(&self) -> bool {
		matches!(self.state, VerifierState::Active)
	}

	pub fn update_accepted_submissions(&mut self, count: u8) -> Result<u32, ArithmeticError> {
		match self.count_of_accepted_submissions.checked_add(count as u32) {
			Some(result) => Ok(result),
			None => Err(ArithmeticError::Overflow),
		}
	}

	pub fn update_un_accepted_submissions(&mut self, count: u8) -> Result<u32, ArithmeticError> {
		match self.count_of_un_accepted_submissions.checked_add(count as u32) {
			Some(result) => Ok(result),
			None => Err(ArithmeticError::Overflow),
		}
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

pub trait VerifierOperations<AccountId, BlockNumber> {
	fn sort_on_selection_score(
		list_of_verifiers: Vec<Verifier<AccountId, BlockNumber>>,
	) -> Vec<Verifier<AccountId, BlockNumber>> {
		list_of_verifiers
	}
}

/// Struct of verification protocol parameters
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ProtocolParameterValues {
	pub threshold_accuracy_score: u32,
	pub penalty_waiver_score: u32,
	pub resumption_waiting_period: u32,
	pub decimals: u8,
}

impl Default for ProtocolParameterValues {
	fn default() -> Self {
		ProtocolParameterValues {
			threshold_accuracy_score: 850000000,
			penalty_waiver_score: 950000000,
			resumption_waiting_period: 100,
			decimals: 9,
		}
	}
}
