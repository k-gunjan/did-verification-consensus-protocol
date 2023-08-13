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

#[test]
fn state_check_on_register_with_greater_deposit_amount() {
	new_test_ext().execute_with(|| {
		let account = account_key("//Alice");
		let deposit = 100u128 * 10u128.pow(12) + 1u128;
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(account), deposit));
		// assert that the state is Active for deposit amount more that threshold
		let v = Verifiers::verifiers(account);
		assert!(matches!(v, Some(Verifier { state: VerifierState::Active, .. })))
	})
}

#[test]
fn state_check_on_register_with_less_deposit_amount() {
	new_test_ext().execute_with(|| {
		let account = account_key("//Alice");
		let deposit = 100u128 * 10u128.pow(12) - 1u128;
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(account), deposit));
		// assert that the state is Pending
		let v = Verifiers::verifiers(account);
		assert!(matches!(v, Some(Verifier { state: VerifierState::Pending, .. })))
	})
}

#[test]
fn state_check_on_additional_deposit_amount_to_become_active() {
	new_test_ext().execute_with(|| {
		let account = account_key("//Alice");
		let deposit = 100u128 * 10u128.pow(12) - 1u128;
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(account), deposit));
		// assert that the state is Pending
		let v = Verifiers::verifiers(account);
		assert!(matches!(v, Some(Verifier { state: VerifierState::Pending, .. })));

		assert_ok!(Verifiers::verifier_deposit(RuntimeOrigin::signed(account), 1u128));
		let v = Verifiers::verifiers(account);
		assert!(matches!(v, Some(Verifier { state: VerifierState::Active, .. })));
	})
}
