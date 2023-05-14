use codec::{Decode, Encode};

use crate::Config;
pub use verifiers::types::{Increment, VerifierUpdateData};

use frame_support::{pallet_prelude::ConstU32, BoundedVec};

use core::cmp::{Eq, Ordering, PartialEq};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

/// Struct of the did verification request submitted by the consumer
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct VerificationRequest<T: Config> {
	pub consumer_account_id: T::AccountId,
	pub submitted_at: T::BlockNumber,
	pub list_of_documents: BoundedVec<u8, T::MaxLengthListOfDocuments>,
	pub did_creation_status: DidCreationStatus,
	pub state: StateConfig<T::BlockNumber>,
}

/// Macro to update the verification request object on fulfillment of the task by verifiers
/// or the chain itself
// #[macro_use]
#[macro_export]
macro_rules! act_on_fulfilled {
	($stage:ident, $obj:expr, $fulfilled_count:expr, $current_block:expr ) => {
		$obj.state.$stage.pending_count_of_verifiers -= $fulfilled_count;
		$obj.state.$stage.done_count_of_verifiers += $fulfilled_count;
		if $obj.state.$stage.pending_count_of_verifiers == 0 {
			$obj.state.$stage.state = false;
			$obj.state.$stage.ended_at = Some($current_block);
			if stringify!($stage) == "reveal" {
				//start the next stage: evaluation of revealed parameters
				$obj.state.stage = VerificationStages::Eval;
				$obj.state.eval_vp_state = Some(EvalVpState::Pending);
				$obj.state.eval_vp_result = Some(EvalVpResult::Pending);
			}
		}
	};
}
pub use act_on_fulfilled;

/// Macro to start or update the stages of the verification requests
/// it starts the new round with new required number to fulfill, wait duration and the current
/// block number
#[macro_export]
macro_rules! start_stage {
	($stage:ident, $obj:expr, $pending_count_increment:expr, $wait_time:expr, $current_block:expr ) => {
		$obj.state.$stage.state = true;
		$obj.state.$stage.round_number += 1;
		$obj.state.$stage.started_at = $current_block;
		// default value started at zero.
		$obj.state.$stage.pending_count_of_verifiers += $pending_count_increment;
		// state duration to be set not incremented
		$obj.state.$stage.state_duration = $wait_time;

		// update verification request stage to indicate Reveal state
		// and close previous stages if starting reveal stage
		if stringify!($stage) == "reveal" {
			$obj.state.stage = VerificationStages::Reveal;
			$obj.state.allot.state = false;
			$obj.state.ack.state = false;
			$obj.state.submit_vp.state = false;
		}
	};
}
pub use start_stage;

/// Enumurates the possible Did Creation Statuses
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug, Copy)]
pub enum DidCreationStatus {
	/// Pending on submission
	#[default]
	Pending,
	/// Created at the end if accepted eval process
	Created,
	/// Rejected at the end if rejected by eval process
	Rejected,
	/// Eval process successful but same id hashes exists already
	RejectedDuplicate,
	/// Evaluation of verificaiton parameter Failed
	Failed,
}

/// State parameters of different stages of the request
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct StateConfig<BlockNumber> {
	pub allot: StateAttributes<BlockNumber>,
	pub ack: StateAttributes<BlockNumber>,
	pub submit_vp: StateAttributes<BlockNumber>,
	pub reveal: StateAttributes<BlockNumber>,
	pub eval_vp_result: Option<EvalVpResult>,
	pub eval_vp_state: Option<EvalVpState>,
	pub stage: VerificationStages,
}

/// Enumerates stages of verification requests
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug)]
pub enum VerificationStages {
	#[default]
	Submitted,
	/// In One or Many stages among Allot, Ack and Submit-verification-parameters stages
	AllotAckVp,
	/// In the reveal stage
	Reveal,
	/// In the Eval Stage
	Eval,
	/// Completed all stages
	Done,
}

/// Attributes of a particular Stage
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct StateAttributes<BlockNumber> {
	pub done_count_of_verifiers: u16,
	pub pending_count_of_verifiers: u16,
	pub round_number: u16,
	/// whether this is to be fulfilled or Done
	pub state: bool,
	/// Start time of the state true
	pub started_at: BlockNumber,
	/// Completion time of the fullfilment
	pub ended_at: Option<BlockNumber>,
	/// Duration for which state was True
	pub state_duration: u32,
}

/// Enumerates the results of the Evaluation of veri-para  
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug)]
pub enum EvalVpResult {
	/// Eval process started
	#[default]
	Pending,
	/// Accepted and ll proceed to created DID
	Accepted(ConsumerDetails),
	/// Rejected and DID ll not be created
	Rejected,
	/// Eval could not result into a clear Accept or Reject. Consider it Failed
	CantDecideAkaFailed,
}

/// Enumerates states of evaluation process
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug)]
pub enum EvalVpState {
	#[default]
	Pending,
	Done,
}

/// Verifier agents struct
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct Verifier<AccountId> {
	pub account_id: AccountId,
	pub score: u32,
	pub state: VerifierState,
	pub count_of_accepted_submissions: u128,
	pub count_of_rejected_submissions: u128,
	pub count_of_incompleted_processes: u128,
}

// enum to hold the state of the verifiers
#[derive(Default, Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
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

/// verification parameter submitted by the verifier
/// Either reject or Accept with the hash of the verification data
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, Ord, Eq, PartialOrd)]
#[scale_info(skip_type_params(T))]
pub enum RevealedParameters {
	Reject,
	Accept(ConsumerDetails),
}

/// Struct to hold the process data of verifier for every verification request
#[derive(Clone, Encode, Decode, Eq, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct VerificationProcessData<T: Config> {
	pub verifier_account_id: T::AccountId,
	/// at blocknumber
	pub allotted_at: Option<T::BlockNumber>,
	/// ack with(blocknumber, confidence_score)
	pub acknowledged: Option<(T::BlockNumber, u8)>,
	pub data: Option<(T::BlockNumber, H256)>,
	pub revealed_data: Option<(T::BlockNumber, RevealedParameters)>,
}

/// allot a task to a particular verifier
impl<T: Config> VerificationProcessData<T> {
	pub fn allot_to_verifier(verifier: T::AccountId, current_block: T::BlockNumber) -> Self {
		VerificationProcessData {
			verifier_account_id: verifier,
			allotted_at: Some(current_block),
			acknowledged: None.into(),
			data: None.into(),
			revealed_data: None.into(),
		}
	}

	// checks if verifier completed the process and returns
	pub fn with_completed(self) -> Option<VerificationProcessDataItem<T>> {
		if let VerificationProcessData {
			verifier_account_id,
			allotted_at: Some(allotted_at),
			acknowledged: Some(acknowledged),
			data: Some(data),
			revealed_data: Some(revealed_data),
		} = self
		{
			Some(VerificationProcessDataItem {
				verifier_account_id,
				allotted_at,
				acknowledged,
				data,
				revealed_data,
			})
		} else {
			None
		}
	}
}

#[derive(Eq, PartialEq, Clone)]
pub struct VerificationProcessDataItem<T: Config> {
	pub verifier_account_id: T::AccountId,
	pub allotted_at: T::BlockNumber,
	/// ack with(blocknumber, confidence_score)
	pub acknowledged: (T::BlockNumber, u8),
	pub data: (T::BlockNumber, H256),
	pub revealed_data: (T::BlockNumber, RevealedParameters),
}

impl<T: Config> VerificationProcessDataItem<T> {
	// duration between ack and submitting the verification data
	pub fn timetaken(&self) -> T::BlockNumber {
		let VerificationProcessDataItem { acknowledged, data, .. } = self;
		acknowledged.0 - data.0
	}
	pub fn is_submission_is_result(&self, s: &EvalVpResult) -> bool {
		let VerificationProcessDataItem { revealed_data: (_, revealed_parameters), .. } = self;
		match s {
			EvalVpResult::Rejected =>
				if *revealed_parameters == RevealedParameters::Reject {
					true
				} else {
					false
				},
			EvalVpResult::Accepted(c) =>
				if *revealed_parameters == RevealedParameters::Accept(c.clone()) {
					true
				} else {
					false
				},
			_ => false, // matches to no one
		}
	}
}

impl<T: Config> PartialOrd for VerificationProcessDataItem<T> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(&other))
	}
}

impl<T: Config> Ord for VerificationProcessDataItem<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		let VerificationProcessDataItem { acknowledged: (ack_at, _), data: (submit_at, _), .. } =
			self;

		let VerificationProcessDataItem {
			acknowledged: (ack_at_other, _),
			data: (submit_at_other, _),
			..
		} = other;
		(*submit_at - *ack_at).cmp(&(*submit_at_other - *ack_at_other))
	}
}

#[derive(Clone)]
pub enum Incentive<Balance> {
	Reward(Balance),
	Penalty(Balance),
}

// #[derive(Clone, Debug)]
// pub enum Increment {
// 	Accepted(u8),
// 	UnAccepted(u8),
// 	NotCompleted(u8),
// }

// #[derive(Debug)]
// pub struct VerifierUpdateData {
// 	// account_id: A,
// 	incentive_factor: f64,
// 	increment: Increment,
// }

impl<T: Config> VerificationProcessData<T> {
	// evaluation the winner submission by majority wins
	// takes the list of the Self
	// returns : result=> submission with clear majority, list of (verifier_account,
	// update_data_of_verifier)
	pub fn eval_incentive(
		submissions: Vec<Self>,
	) -> (EvalVpResult, Vec<(T::AccountId, VerifierUpdateData)>) {
		let mut data: Vec<(T::AccountId, VerifierUpdateData)> = Vec::new();
		let mut completed_subs: Vec<VerificationProcessDataItem<T>> = Vec::new();
		submissions.into_iter().for_each(|vpr| {
			let verifier = vpr.verifier_account_id.clone();
			if let Some(rd) = vpr.with_completed() {
				completed_subs.push(rd)
			} else {
				data.push((
					verifier,
					VerifierUpdateData {
						increment: Increment::NotCompleted(1),
						incentive_factor: 0.0,
					},
				))
			}
		});

		let (result, partial_incentive_data) =
			VerificationProcessDataItem::eval_incentive_on_completed(completed_subs);
		data.extend(partial_incentive_data);
		(result, data)
	}
}

impl<T: Config> VerificationProcessDataItem<T> {
	// evaluation the winner submission by majority wins
	// takes the list of the Self
	// returns : result=> submission with clear majority
	fn eval_result(params: &[RevealedParameters]) -> EvalVpResult {
		let mut counts = BTreeMap::new();
		for p in params {
			let count = counts.entry(p).or_insert(0);
			*count += 1;
		}
		// counts
		let mut max_count = 0;
		let mut max_variant = None;
		let mut max_count_2 = 0;
		let mut max_variant_2 = None;
		for (discr, count) in &counts {
			if *count > max_count {
				max_count = *count;
				max_variant = Some(discr);
			} else if *count > max_count_2 {
				max_count_2 = *count;
				max_variant_2 = Some(discr);
			}
		}

		if max_variant.is_some() {
			if max_variant_2.is_some() {
				if max_count == max_count_2 {
					// there is a tie and no clear majority
					max_variant = None;
				}
			}
		}
		let result: EvalVpResult = match max_variant {
			Some(variant) => match variant {
				RevealedParameters::Reject => EvalVpResult::Rejected,
				RevealedParameters::Accept(d) => EvalVpResult::Accepted(d.clone()),
			},
			None => EvalVpResult::CantDecideAkaFailed,
		};
		result
	}

	// evaluation the winner submission by majority wins
	// takes the list of the Self where verification process has been completed by the verifier by
	// revealing the data returns : result=> submission with clear majority, list of
	// (verifier_account, update_data_of_verifier)
	fn eval_incentive_on_completed(
		submissions: Vec<Self>,
	) -> (EvalVpResult, Vec<(T::AccountId, VerifierUpdateData)>) {
		// let mut data: BTreeMap<K, V> = BTreeMap::new();
		let entries: Vec<RevealedParameters> = submissions
			.iter()
			.map(|vpr| {
				let (_, rd) = vpr.revealed_data.clone();
				rd
			})
			.collect();
		let result: EvalVpResult = Self::eval_result(&entries[..]);
		// let fastest = submissions.iter().min().ok_or(())?;
		// let slowest = submissions.iter().min().ok_or(())?;
		// let denominator = fastest.timetaken() - slowest.timetaken();
		let r = submissions
			.iter()
			.map(|x| {
				let increment = if x.is_submission_is_result(&result) {
					Increment::Accepted(1)
				} else {
					Increment::UnAccepted(1)
				};
				(
					x.verifier_account_id.clone(),
					VerifierUpdateData { increment, incentive_factor: 1.0 },
				)
			})
			.collect::<Vec<_>>();

		(result, r)
	}
}

/// Struct of consumer personal data
/// hash1 may be hash of (name + DOB + Fathers name)
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, Ord, Eq, PartialOrd)]
pub struct ConsumerDetails {
	pub country: BoundedVec<u8, ConstU32<50>>,
	pub id_issuing_authority: BoundedVec<u8, ConstU32<200>>,
	pub type_of_id: BoundedVec<u8, ConstU32<200>>,
	pub hash1_name_dob_father: Option<H256>,
	pub hash2_name_dob_mother: Option<H256>,
	pub hash3_name_dob_guardian: Option<H256>,
}

impl ConsumerDetails {
	pub fn hashes(&self) -> Vec<H256> {
		let mut result = Vec::new();
		if let Some(hash) = self.hash1_name_dob_father {
			result.push(hash);
		}
		if let Some(hash) = self.hash2_name_dob_mother {
			result.push(hash);
		}
		if let Some(hash) = self.hash3_name_dob_guardian {
			result.push(hash);
		}
		result
	}
}

/// Struct of verification protocol parameters
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct ProtocolParameterValues {
	pub max_length_list_of_documents: u16,
	pub min_count_at_allot_stage: u16,
	pub min_count_at_ack_accept_stage: u16,
	pub min_count_at_submit_vp_stage: u16,
	pub min_count_at_reveal_stage: u16,
	pub max_waiting_time_at_stages: u32,
}

impl Default for ProtocolParameterValues {
	fn default() -> Self {
		ProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 4,
			min_count_at_reveal_stage: 4,
			max_waiting_time_at_stages: 50,
		}
	}
}
