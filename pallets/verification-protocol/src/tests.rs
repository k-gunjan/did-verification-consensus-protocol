use crate::{
	mock::*,
	types::{IdDocument, IdType},
	Error,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_whitelist_id_type() {
	new_test_ext().execute_with(|| {
		// An ID Document Type to be whitelisted
		let id_type =
			IdType::build(b"PASSPORT".to_vec(), b"GOVTOFINDIA".to_vec(), b"INDIA".to_vec())
				.expect("error in creatign id type");
		assert_eq!(VerificationProtocol::whitelisted_countries().len(), 0);
		assert_eq!(VerificationProtocol::whitelisted_id_types(&id_type.country).len(), 0);

		let account = account_key("//Alice");

		assert_ok!(VerificationProtocol::whitelist_id_type(
			RuntimeOrigin::signed(account),
			id_type.clone()
		));
		assert_eq!(VerificationProtocol::whitelisted_countries().len(), 1);
		assert_eq!(VerificationProtocol::whitelisted_id_types(&id_type.country).len(), 1);

		// Test for duplicate entry
		// It returns Tx success instead of returning error
		assert_ok!(VerificationProtocol::whitelist_id_type(
			RuntimeOrigin::signed(account),
			id_type.clone()
		));

		// Counts should remain same
		assert_eq!(VerificationProtocol::whitelisted_countries().len(), 1);
		assert_eq!(VerificationProtocol::whitelisted_id_types(&id_type.country).len(), 1);
	});
}

#[test]
fn test_remove_id_type() {
	new_test_ext().execute_with(|| {
		// An ID Document Type to be whitelisted
		let id_type =
			IdType::build(b"PASSPORT".to_vec(), b"GOVTOFINDIA".to_vec(), b"INDIA".to_vec())
				.expect("error in creatign id type");
		let account = account_key("//Alice");

		assert_ok!(VerificationProtocol::whitelist_id_type(
			RuntimeOrigin::signed(account),
			id_type.clone()
		));
		assert_eq!(VerificationProtocol::whitelisted_countries().len(), 1);
		assert_eq!(VerificationProtocol::whitelisted_id_types(&id_type.country).len(), 1);

		// Test for remove
		// It returns Tx success instead of returning error
		assert_ok!(VerificationProtocol::remove_id_type(
			RuntimeOrigin::signed(account),
			id_type.clone()
		));

		// Counts should reduce
		assert_eq!(VerificationProtocol::whitelisted_countries().len(), 0);
		assert_eq!(VerificationProtocol::whitelisted_id_types(&id_type.country).len(), 0);

		// Try to remove non-whitelisted IdType
		assert_noop!(
			VerificationProtocol::remove_id_type(RuntimeOrigin::signed(account), id_type),
			Error::<Test>::IdTypeNotDefined
		);
	});
}

#[test]
fn test_whitelisted_countries() {
	new_test_ext().execute_with(|| {
		// Country should be removed from the whitelist only when all the entries corresponding
		//to it has be removed from the whitelisted_id_types entry

		// An ID Document Type to be whitelisted
		let id_type_1 =
			IdType::build(b"PASSPORT".to_vec(), b"GOVTOFINDIA".to_vec(), b"INDIA".to_vec())
				.expect("error in creatign id type");
		let id_type_2 =
			IdType::build(b"VOTERID".to_vec(), b"GOVTOFINDIA".to_vec(), b"INDIA".to_vec())
				.expect("error in creatign id type");

		let account = account_key("//Alice");

		// Insert first Id Type
		assert_ok!(VerificationProtocol::whitelist_id_type(
			RuntimeOrigin::signed(account),
			id_type_1.clone()
		));
		// Insert second Id Type
		assert_ok!(VerificationProtocol::whitelist_id_type(
			RuntimeOrigin::signed(account),
			id_type_2.clone()
		));
		assert_eq!(VerificationProtocol::whitelisted_countries().len(), 1);
		assert_eq!(VerificationProtocol::whitelisted_id_types(&id_type_1.country).len(), 2);

		// Remove id_type_1
		assert_ok!(VerificationProtocol::remove_id_type(
			RuntimeOrigin::signed(account),
			id_type_1.clone()
		));

		// Count of countries should not reduce
		assert_eq!(VerificationProtocol::whitelisted_countries().len(), 1);
		assert_eq!(VerificationProtocol::whitelisted_id_types(&id_type_1.country).len(), 1);

		// Remove id_type_2
		assert_ok!(VerificationProtocol::remove_id_type(
			RuntimeOrigin::signed(account),
			id_type_2.clone()
		));

		// Count of countries should reduce
		assert_eq!(VerificationProtocol::whitelisted_countries().len(), 0);
		assert_eq!(VerificationProtocol::whitelisted_id_types(&id_type_1.country).len(), 0);
	});
}
