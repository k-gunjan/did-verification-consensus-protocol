use codec::{Decode, Encode, EncodeLike};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use crate::Config;
use frame_support::inherent::Vec;
use frame_support::pallet_prelude::ConstU32;
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
	pub done_count_of_verifiers: u32,
	pub pending_count_of_verifiers: u32,
	pub round_number: u32,
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
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub enum VerificationParameter {
	Reject,
	Accept(H256),
}

/// Struct to hold the process data of verifier per task
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct VerificationProcessData<AccountId> {
	pub verifier_account_id: AccountId,
    /// at blocknumber
    pub allotted: Option<u128>  
    /// ack with(blocknumber, confidence_score)
    pub acknowledged: Option<(u128, u8)> 
	pub data: Option<(u128, VerificationParameter)>,
	pub revealed_data: Option<(u128, ConsumerDetails)>,
	pub is_valid: Option<bool>,
}

#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct ConsumerDetails {
	pub hash1: H256,
	pub hash2: H256,
	pub hash3: H256,
}
