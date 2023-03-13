use codec::{Decode, Encode};

use crate::Config;

use frame_support::{inherent::Vec, pallet_prelude::ConstU32, BoundedVec};

use scale_info::TypeInfo;
use sp_core::H256;

/// Struct of the did verification request submitted by the consumer
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct VerificationRequest<T: Config> {
	pub consumer_account_id: T::AccountId,
	pub submitted_at: T::BlockNumber,
	pub list_of_documents: BoundedVec<u8, T::MaxLengthListOfDocuments>, //TODO; implement checks
	pub list_of_id_hashes: BoundedVec<u8, T::MaxLengthListOfDocuments>,
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
	};
}
pub use start_stage;

/// Enumurates the possible Did Creation Statuses
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug)]
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
	Accepted,
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
	Wip,
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
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub enum RevealedParameters {
	Reject,
	Accept(ConsumerDetails),
}

/// Struct to hold the process data of verifier for every verification request
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct VerificationProcessData<T: Config> {
	pub verifier_account_id: T::AccountId,
	/// at blocknumber
	pub allotted_at: Option<T::BlockNumber>,
	/// ack with(blocknumber, confidence_score)
	pub acknowledged: Option<(T::BlockNumber, u8)>,
	pub data: Option<(T::BlockNumber, H256)>,
	pub revealed_data: Option<(T::BlockNumber, RevealedParameters)>,
	pub is_valid: Option<bool>,
}

/// allot to a task to a particular verifier
impl<T: Config> VerificationProcessData<T> {
	pub fn allot_to_verifier(verifier: T::AccountId, current_block: T::BlockNumber) -> Self {
		VerificationProcessData {
			verifier_account_id: verifier,
			allotted_at: Some(current_block),
			acknowledged: None.into(),
			data: None.into(),
			revealed_data: None.into(),
			is_valid: None.into(),
		}
	}
}

/// Struct of consumer personal data
/// hash1 may be hash of (name + DOB + Fathers name)
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
pub struct ConsumerDetails {
	pub country: BoundedVec<u8, ConstU32<50>>,
	pub id_issuing_authority: BoundedVec<u8, ConstU32<200>>,
	pub hash1_name_dob_father: H256,
	pub hash2_name_dob_mother: H256,
	pub hash3_name_dob_guardian: H256,
}

/// Struct of verification protocol parameters
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct ProtocolParameterValues {
	pub max_length_list_of_documents: u16,
	pub min_count_at_vp_reveal_stage: u16,
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
			min_count_at_vp_reveal_stage: 5,
			min_count_at_allot_stage: 20,
			min_count_at_ack_accept_stage: 15,
			min_count_at_submit_vp_stage: 10,
			min_count_at_reveal_stage: 5,
			max_waiting_time_at_stages: 50,
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
// #[scale_info(skip_type_params(T))]
pub struct RevealedData {
	pub country: Vec<u8>,
	pub id_issuing_authority: Vec<u8>,
	pub hash1_name_dob_father: Vec<u8>,
	pub hash2_name_dob_mother: Vec<u8>,
	pub hash3_name_dob_guardian: Vec<u8>,
}
