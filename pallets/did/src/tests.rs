use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
// use frame_system::Origin;

#[test]
fn add_attribute() {
	new_test_ext().execute_with(|| {
		// test for adding attribute
		//it should succeed if the attribute is not already present for the DID

		//set blocknumber to 1
		System::set_block_number(1);
		//set timestamp to 1
		Timestamp::set_timestamp(1);

		let name = "key".as_bytes().to_vec();
		let value = "value".as_bytes().to_vec();

		// DidModule::add_attribute(origin, did, name, value, None)
		// test for attribute added
		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(10),
			10,
			name.clone(),
			value.clone(),
			Some(100)
		));
		// assert add_attribute with none block number
		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(10),
			10,
			"key2".as_bytes().to_vec(),
			"value2".as_bytes().to_vec(),
			None
		));
	});
}

#[test]
fn correct_error_for_none_existing_key() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no such key is present.
		// let name = "key".as_bytes().to_vec();
		// let value = "value".as_bytes().to_vec();

		// // DidModule::add_attribute(origin, did, name, value, None)
		// // add an attribute to the did
		// assert_ok!(DidModule::add_attribute(RuntimeOrigin::signed(10), 10, name.clone(),
		// value.clone(), Some(100))); test to read undefined attribute
		// it should fail because the attribute is not defined
		assert_noop!(
			DidModule::get_attribute(RuntimeOrigin::signed(10), 10, "key5".as_bytes().to_vec()),
			Error::<Test>::AttributeNotFound
		);
	});
}

#[test]
fn correct_error_for_existing_key_addition() {
	new_test_ext().execute_with(|| {
		// test for adding attribute which already exists. Operation by the owner
		let name = "key".as_bytes().to_vec();
		let value = "value".as_bytes().to_vec();

		// DidModule::add_attribute(origin, did, name, value, None)
		// test for attribute added
		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(10),
			10,
			name.clone(),
			value.clone(),
			Some(100)
		));
		// test for duplicate attribute
		assert_noop!(
			DidModule::add_attribute(
				RuntimeOrigin::signed(10),
				10,
				name.clone(),
				value.clone(),
				Some(100)
			),
			Error::<Test>::DuplicateNotNeeded
		);
	});
}

#[test]
fn unauthorized_addition() {
	new_test_ext().execute_with(|| {
		// test for adding attribute of did by unauthorized user
		let name = "key".as_bytes().to_vec();
		let value = "value".as_bytes().to_vec();

		// DidModule::add_attribute(origin, did, name, value, None)
		// test for duplicate attribute
		assert_noop!(
			DidModule::add_attribute(
				RuntimeOrigin::signed(11),
				10,
				name.clone(),
				value.clone(),
				Some(100)
			),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn read_attribute_by_owner() {
	new_test_ext().execute_with(|| {
		// test for readin attribute
		let name = "key".as_bytes().to_vec();
		let value = "value".as_bytes().to_vec();

		// DidModule::add_attribute(origin, did, name, value, None)
		// add attribute
		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(10),
			10,
			name.clone(),
			value.clone(),
			None
		));

		// test for read attribute read by the owner
		assert_ok!(DidModule::get_attribute(RuntimeOrigin::signed(10), 10, name.clone()));
	});
}

#[test]
fn read_attribute_by_non_owner() {
	new_test_ext().execute_with(|| {
		// test for readin attribute
		let name = "key".as_bytes().to_vec();
		let value = "value".as_bytes().to_vec();

		// DidModule::add_attribute(origin, did, name, value, None)
		// add attribute
		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(10),
			10,
			name.clone(),
			value.clone(),
			None
		));

		// test for read attribute read by the not-owner user
		// it should pass because the attribute read is public
		assert_ok!(DidModule::get_attribute(RuntimeOrigin::signed(11), 10, name.clone()));
	});
}
