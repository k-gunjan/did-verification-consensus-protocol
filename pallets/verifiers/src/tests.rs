use crate::{mock::*, types::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_register_verifier() {
	new_test_ext().execute_with(|| {
		let account = account_key("//Alice");
		let deposit = 100_u128 * 10_u128.pow(12);

		// Register a verifier
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(account), deposit));

		// assert the verifier exists
		assert!(Verifiers::verifiers(account).is_some())
	});
}

#[test]
fn correct_error_on_re_register_verifier() {
	new_test_ext().execute_with(|| {
		let account = account_key("//Alice");
		let deposit = 100_u128 * 10_u128.pow(12);

		// Register a verifier
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(account), deposit));

		// error VerifierAlreadyRegistered upon attempt to re-register
		assert_noop!(
			Verifiers::register_verifier(RuntimeOrigin::signed(account), deposit),
			Error::<Test>::VerifierAlreadyRegistered
		);
	});
}

#[test]
fn correct_error_on_zero_deposit_register_verifier() {
	new_test_ext().execute_with(|| {
		let account = account_key("//Alice");
		let deposit = 0u128;

		// error InvalidDepositeAmount upon attempt to re-register with zero deposit
		assert_noop!(
			Verifiers::register_verifier(RuntimeOrigin::signed(account), deposit),
			Error::<Test>::InvalidDepositeAmount
		);
	});
}
