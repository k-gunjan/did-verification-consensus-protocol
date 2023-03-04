use codec::{Decode, Encode, EncodeLike};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use crate::Config;
use frame_support::inherent::Vec;
use frame_support::pallet_prelude::ConstU32;
use frame_support::pallet_prelude::Get;
use frame_support::BoundedVec;
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

impl<T: Config> VerificationRequest<T> {
	/// Required number of accept has been received
	/// and  closing the new acceptence
	fn act_on_fulfilled_ack(&mut self) {
		self.state.ack.state = false;
	}
	/// Required number of verification parameter has been received
	/// and  closing the new submissions
	fn act_on_fulfilled_submit_vp(&mut self) {
		self.state.submit_vp.state = false;
	}
	/// Required number of reveal has been received
	/// and  closing the new reveals
	fn act_on_fulfilled_reveal(&mut self) {
		self.state.reveal.state = false;
	}
}

/// Enumurates the possible Did Creation Statuses
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct StateAttributes<BlockNumber> {
	pub done_count_of_verifiers: u8,
	pub pending_count_of_verifiers: u8,
	pub round_number: u8,
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
// #[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Debug)]
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[scale_info(skip_type_params(T))]
pub enum VerificationParameter {
	Reject,
	Accept(H256),
}

/// Struct to hold the process data of verifier for every verification request
// #[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Debug, EncodeLike)]
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct VerificationProcessData<T: Config> {
	pub verifier_account_id: T::AccountId,
	/// at blocknumber
	pub allotted_at: Option<T::BlockNumber>,
	/// ack with(blocknumber, confidence_score)
	pub acknowledged: Option<(T::BlockNumber, u8)>,
	pub data: Option<(T::BlockNumber, VerificationParameter)>,
	pub revealed_data: Option<(T::BlockNumber, ConsumerDetails)>,
	pub is_valid: Option<bool>,
}

/// Struct of consumer personal data
/// hash1 may be hash of (name + DOB + Fathers name)
// #[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Debug)]
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
// #[scale_info(skip_type_params(T))]
pub struct ConsumerDetails {
	pub hash1: H256,
	pub hash2: H256,
	pub hash3: H256,
}

/// Struct of verification protocol parameters
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct ProtocolParameterValues {
	pub max_length_list_of_documents: u16,
	pub min_count_at_vp_reveal_stage: u16,
	pub min_count_at_allot_stage: u8,
	pub min_count_at_ack_accept_stage: u8,
	pub min_count_at_submit_vp_stage: u8,
	pub min_count_at_reveal_stage: u8,
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
