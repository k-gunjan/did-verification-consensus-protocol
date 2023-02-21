use crate::{did::Did, mock::*, Error};

use frame_support::{assert_noop, assert_ok};
use hex_literal::hex;

#[test]
fn add_attribute_test() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let acct = "Felidae";
		let acct2 = "Felidae2";
		let origin = account_key(acct);
		let did_account = account_key(acct2);
		let name = b"id";
		let attribute = b"did:pan:1234567890";

		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(origin),
			did_account,
			name.to_vec(),
			attribute.to_vec(),
			None
		));

		// Test for duplicate entry
		assert_noop!(
			DidModule::add_attribute(
				RuntimeOrigin::signed(origin),
				did_account,
				name.to_vec(),
				attribute.to_vec(),
				None
			),
			Error::<Test>::AttributeAlreadyExist
		);

		// Test update did attribute with invalid validity
		assert_noop!(
			DidModule::add_attribute(
				RuntimeOrigin::signed(origin),
				did_account,
				b"name".to_vec(),
				attribute.to_vec(),
				Some(u64::max_value()),
			),
			Error::<Test>::MaxBlockNumberExceeded
		);
	});
}

#[test]
fn update_attribute_test() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let acct = "Felidae";
		let acct2 = "Felidae2";
		let acct3 = "Fake";
		let origin = account_key(acct);
		let did_account = account_key(acct2);
		let fake_origin = account_key(acct3);
		let name = b"id";
		let attribute = b"did:pan:1234567890";

		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(origin),
			did_account,
			name.to_vec(),
			attribute.to_vec(),
			None
		));

		// Test update owner did attribute
		assert_ok!(DidModule::update_attribute(
			RuntimeOrigin::signed(origin),
			did_account,
			name.to_vec(),
			attribute.to_vec(),
			None,
		));

		// Test update did attribute with invalid validity
		assert_noop!(
			DidModule::update_attribute(
				RuntimeOrigin::signed(origin),
				did_account,
				name.to_vec(),
				attribute.to_vec(),
				Some(u64::max_value()),
			),
			Error::<Test>::MaxBlockNumberExceeded
		);

		// Test update another owner did attribute
		assert_noop!(
			DidModule::update_attribute(
				RuntimeOrigin::signed(fake_origin),
				did_account,
				name.to_vec(),
				attribute.to_vec(),
				None,
			),
			Error::<Test>::AttributeAuthorizationFailed
		);

		// Test update non-existing attribute
		assert_noop!(
			DidModule::update_attribute(
				RuntimeOrigin::signed(origin),
				did_account,
				b"name".to_vec(),
				attribute.to_vec(),
				None,
			),
			Error::<Test>::AttributeNotFound
		);
	});
}

#[test]
fn read_attribute_test() {
	new_test_ext().execute_with(|| {
		let acct = "Felidae";
		let acct2 = "Felidae2";
		let origin = account_key(acct);
		let did_account = account_key(acct2);
		let name = b"id";
		let attribute = b"did:pan:1234567890";

		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(origin),
			did_account,
			name.to_vec(),
			attribute.to_vec(),
			None
		));

		// Test read existing attribute
		assert_ok!(DidModule::read_attribute(
			RuntimeOrigin::signed(origin),
			did_account,
			name.to_vec()
		));

		// Test read non-existing attribute
		assert_noop!(
			DidModule::read_attribute(
				RuntimeOrigin::signed(origin),
				account_key("invalid"),
				name.to_vec()
			),
			Error::<Test>::AttributeNotFound
		);
	});
}

#[test]
fn remove_attribute_test() {
	new_test_ext().execute_with(|| {
		let acct = "Felidae";
		let acct2 = "Felidae2";
		let acct3 = "Fake";
		let origin = account_key(acct);
		let did_account = account_key(acct2);
		let fake_origin = account_key(acct3);
		let name = b"id";
		let attribute = b"did:pan:1234567890";

		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(origin),
			did_account,
			name.to_vec(),
			attribute.to_vec(),
			None
		));

		// Test remove owner did attribute
		assert_ok!(DidModule::remove_attribute(
			RuntimeOrigin::signed(origin),
			did_account,
			name.to_vec()
		));

		// Test remove another owner did attribute
		assert_noop!(
			DidModule::remove_attribute(
				RuntimeOrigin::signed(fake_origin),
				did_account,
				name.to_vec()
			),
			Error::<Test>::AttributeAuthorizationFailed
		);

		// Test remove non-existing attribute
		assert_noop!(
			DidModule::remove_attribute(
				RuntimeOrigin::signed(origin),
				did_account,
				b"name".to_vec()
			),
			Error::<Test>::AttributeNotFound
		);
	});
}

#[test]
fn hashed_key_correctness_test() {
	new_test_ext().execute_with(|| {
		let did_account = sp_core::sr25519::Public::from_raw(hex!(
			"d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
		));
		let name = b"id";
		let expected_result =
			hex!("9fe4704db64fc812b35abc8a53d0e4183f0621d159f5241edd94d8310f92c409");

		assert_eq!(DidModule::get_hashed_key_for_attr(&did_account, &name[..]), expected_result)
	});
}
