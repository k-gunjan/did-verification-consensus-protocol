use crate::types::*;
use crate::Config;
use frame_support::dispatch::DispatchResult;
use frame_support::inherent::Vec;
use frame_support::pallet_prelude::ConstU32;
use frame_support::traits::ConstU8;
use frame_support::BoundedVec;

/// Traits of overall verification process
pub trait VerificationProcess<C: Config> {
	/// Creates a DID verification request
	fn create_verification_request(
		who: &C::AccountId,
		list_of_documents: BoundedVec<u8, C::MaxLengthListOfDocuments>,
	) -> DispatchResult;

	/// alot tasks to verifiers
	fn allot_verification_task(
		verifiers: Vec<C::AccountId>,
		verification_reuests: Vec<(&C::AccountId, u32)>,
	) -> DispatchResult;

	fn ack_verification_task(
		_who: &C::AccountId,
		consumer_account_id: &C::AccountId,
		confidence_score: Option<BoundedVec<u8, ConstU32<10>>>,
	) -> DispatchResult;

	fn is_verifier_allowed_ack(
		_who: &C::AccountId,
		consumer_account_id: &C::AccountId,
	) -> DispatchResult;

	fn submit_verification_parameter(
		_who: &C::AccountId,
		consumer_account_id: &C::AccountId,
		verification_parameters: VerificationParameter,
	) -> DispatchResult;

	fn is_verifier_allowed_vp(
		_who: &C::AccountId,
		consumer_account_id: &C::AccountId,
	) -> DispatchResult;

	fn reveal_verification_parameter(
		_who: &C::AccountId,
		consumer_account_id: &C::AccountId,
		verification_parameters: Vec<C::Hash>,
		secret: BoundedVec<u8, ConstU8<20>>,
	) -> DispatchResult;

	fn is_verifier_allowed_reveal(
		_who: &C::AccountId,
		consumer_account_id: &C::AccountId,
	) -> DispatchResult;
}
