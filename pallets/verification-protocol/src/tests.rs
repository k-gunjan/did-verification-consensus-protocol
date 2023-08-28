use crate::mock::*;
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

#[test]
fn test_did_creation_request() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let doc_url = vec![0u8; 100];
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			vec![0u8; 100]
		));
		let verification_reqeust_data = create_verification_request(alice, doc_url);
		let stored_request = VerificationProtocol::verification_requests(alice);
		assert_eq!(Some(verification_reqeust_data), stored_request);
	});
}

#[test]
fn test_did_creation_request_with_invalid_document_url_length() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		//check with very small url :should fail
		assert_noop!(
			VerificationProtocol::submit_did_creation_request(
				RuntimeOrigin::signed(alice),
				vec![0u8; 2]
			),
			Error::<Test>::ListOfDocsTooShort
		);
		// check with very long url :should fail
		assert_noop!(
			VerificationProtocol::submit_did_creation_request(
				RuntimeOrigin::signed(alice),
				vec![0u8; 200]
			),
			Error::<Test>::ListOfDocsTooLong
		);
	});
}

#[test]
fn test_did_creation_request_re_submission() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");

		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			vec![0u8; 100]
		));
		//upon trying to register again- it should fail
		assert_noop!(
			VerificationProtocol::submit_did_creation_request(
				RuntimeOrigin::signed(alice),
				vec![0u8; 100]
			),
			Error::<Test>::CreationRequestAlreadyRegistered
		);
	});
}

#[test]
fn test_update_verification_protocol_parameters() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));
		let stored_parameters = VerificationProtocol::protocol_parameters();
		assert_eq!(stored_parameters, new_parameters);
	});
}

#[test]
fn test_task_alocation_to_a_verifier() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let mut verification_reqeust_data = create_verification_request(alice, doc_url.clone());
		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register a verifier
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));

		Verifiers::on_finalize(System::block_number());
		VerificationProtocol::on_finalize(System::block_number());

		// Check the verification task entry
		let expected_verification_process_data: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(bob.clone(), System::block_number());
		let stored_data = VerificationProtocol::verrification_process_records(alice, bob);
		assert_eq!(Some(expected_verification_process_data), stored_data);

		// Check the updated status of the verification request
		update_verification_request_on_allot(&mut verification_reqeust_data, 1);
		update_verification_request_start_ack(&mut verification_reqeust_data);
		update_verification_request_start_submit_vp(&mut verification_reqeust_data);

		let stored_request = VerificationProtocol::verification_requests(alice);
		assert_eq!(Some(verification_reqeust_data), stored_request);
	})
}

#[test]
fn test_task_alocation_to_verifiers_as_per_config() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));
		run_to_block(System::block_number() + 1);

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register a verifier
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));
		// Check the verification task entry
		let expected_verification_process_data_alice: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(alice.clone(), System::block_number());
		let expected_verification_process_data_bob: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(bob.clone(), System::block_number());
		let expected_verification_process_data_dave: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(dave.clone(), System::block_number());
		let expected_verification_process_data_charlie: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(charlie.clone(), System::block_number());

		run_to_block(System::block_number() + 1);

		let stored_data_alice = VerificationProtocol::verrification_process_records(alice, alice);
		let stored_data_bob = VerificationProtocol::verrification_process_records(alice, bob);
		let stored_data_dave = VerificationProtocol::verrification_process_records(alice, dave);
		let stored_data_charlie =
			VerificationProtocol::verrification_process_records(alice, charlie);
		assert_eq!(Some(expected_verification_process_data_alice), stored_data_alice);
		assert_eq!(Some(expected_verification_process_data_bob), stored_data_bob);
		assert_eq!(Some(expected_verification_process_data_dave), stored_data_dave);
		assert_eq!(Some(expected_verification_process_data_charlie), stored_data_charlie);
	})
}

#[test]
fn test_task_alocation_to_verifiers_as_per_config_in_multiple_blocks() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");

		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));
		run_to_block(System::block_number() + 1);

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register a verifier
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));

		// Check the verification task entry
		let expected_verification_process_data_alice: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(alice.clone(), System::block_number());
		let expected_verification_process_data_bob: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(bob.clone(), System::block_number());

		run_to_block(System::block_number() + 1);

		let stored_data_alice = VerificationProtocol::verrification_process_records(alice, alice);
		let stored_data_bob = VerificationProtocol::verrification_process_records(alice, bob);

		assert_eq!(Some(expected_verification_process_data_alice), stored_data_alice);
		assert_eq!(Some(expected_verification_process_data_bob), stored_data_bob);

		run_to_block(System::block_number() + 1);
		// as tast is still to be allotted more verifiers so dave and charlie should get the task
		// upon registration
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		let expected_verification_process_data_dave: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(dave.clone(), System::block_number());
		let expected_verification_process_data_charlie: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(charlie.clone(), System::block_number());
		run_to_block(System::block_number() + 1);

		let stored_data_dave = VerificationProtocol::verrification_process_records(alice, dave);
		let stored_data_charlie =
			VerificationProtocol::verrification_process_records(alice, charlie);
		assert_eq!(Some(expected_verification_process_data_dave), stored_data_dave);
		assert_eq!(Some(expected_verification_process_data_charlie), stored_data_charlie);
	})
}

#[test]
fn test_task_accept_by_authorized_verifier() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let mut verification_reqeust_data = create_verification_request(alice, doc_url);
		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			vec![0u8; 100]
		));
		// Register a verifier
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));

		// Check the updated status of the verification request
		update_verification_request_on_allot(&mut verification_reqeust_data, 1);
		update_verification_request_start_ack(&mut verification_reqeust_data);
		update_verification_request_start_submit_vp(&mut verification_reqeust_data);

		// Check the verification task entry
		let mut expected_verification_process_data: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(bob.clone(), System::block_number());

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));

		expected_verification_process_data.acknowledged = Some((System::block_number(), 5));

		verification_reqeust_data.state.ack.done_count_of_verifiers += 1;

		// Check the state of verification process record
		let stored_data = VerificationProtocol::verrification_process_records(alice, bob);
		assert_eq!(Some(expected_verification_process_data), stored_data);

		// Check the state of verification request
		let stored_request = VerificationProtocol::verification_requests(alice);
		assert_eq!(Some(verification_reqeust_data), stored_request);
	})
}

#[test]
fn test_task_accept_by_un_authorized_verifier() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register a verifier
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));

		run_to_block(System::block_number() + 1);

		// Confirm dave has not been allotted the task of alice
		let stored_data = VerificationProtocol::verrification_process_records(alice, dave);
		assert_eq!(stored_data, None);
		// Trying to acknowledge should fail
		assert_noop!(
			VerificationProtocol::accept_verification_task(RuntimeOrigin::signed(dave), alice, 5),
			Error::<Test>::NotAllowed
		);
	})
}

#[test]
fn test_submit_verification_data_reject_by_verifier() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let mut verification_reqeust_data = create_verification_request(alice, doc_url.clone());
		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register a verifier
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));

		// Check the updated status of the verification request
		update_verification_request_on_allot(&mut verification_reqeust_data, 1);
		update_verification_request_start_ack(&mut verification_reqeust_data);
		update_verification_request_start_submit_vp(&mut verification_reqeust_data);

		// Check the verification task entry
		let mut expected_verification_process_data: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(bob.clone(), System::block_number());

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));

		expected_verification_process_data.acknowledged = Some((System::block_number(), 5));

		verification_reqeust_data.state.ack.done_count_of_verifiers += 1;

		let clear_reject = b"REJECT".to_vec();
		let secret = b"123".to_vec();
		let combined_clear_secret = [&clear_reject[..], &secret[..]].concat();
		let hash_combined_clear_vec = keccak_256(&combined_clear_secret);
		let h256_hash_combined_clear_vec = H256::from_slice(&hash_combined_clear_vec);

		// Submit REJECT for the verification task of alice
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));

		// update the expected verification request and process data
		expected_verification_process_data.data =
			Some((System::block_number(), h256_hash_combined_clear_vec));

		verification_reqeust_data.state.submit_vp.done_count_of_verifiers += 1;

		// Check the state of verification process record
		let stored_data = VerificationProtocol::verrification_process_records(alice, bob);
		assert_eq!(Some(expected_verification_process_data), stored_data);

		// Check the state of verification request
		let stored_request = VerificationProtocol::verification_requests(alice);
		assert_eq!(Some(verification_reqeust_data), stored_request);
	})
}

#[test]
fn test_submit_verification_data_without_ack_by_verifier() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register a verifier
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));

		run_to_block(System::block_number() + 1);

		let clear_reject = b"REJECT".to_vec();
		let secret = b"123".to_vec();
		let combined_clear_secret = [&clear_reject[..], &secret[..]].concat();
		let hash_combined_clear_vec = keccak_256(&combined_clear_secret);
		let h256_hash_combined_clear_vec = H256::from_slice(&hash_combined_clear_vec);

		// Submit verification data for the verification task before accepting the task should fail
		assert_noop!(
			VerificationProtocol::submit_verification_data(
				RuntimeOrigin::signed(bob),
				alice,
				h256_hash_combined_clear_vec
			),
			Error::<Test>::AcceptPending
		);
	})
}

#[test]
fn test_reveal_before_reveal_stage_is_open_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register a verifier
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));

		let clear_reject = b"REJECT".to_vec();
		let secret = b"123".to_vec();
		let combined_clear_secret = [&clear_reject[..], &secret[..]].concat();
		let hash_combined_clear_vec = keccak_256(&combined_clear_secret);
		let h256_hash_combined_clear_vec = H256::from_slice(&hash_combined_clear_vec);

		// Submit REJECT for the verification task of alice
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));

		//Check that reveal is not allow at this moment
		assert!(matches!(
			VerificationProtocol::verification_requests(alice),
			Some(VerificationRequest {
				state: StateConfig { reveal: StateAttributes { state: false, .. }, .. },
				..
			})
		));

		// Submit REJECT for the verification task of alice
		assert_noop!(
			VerificationProtocol::reveal_data(
				RuntimeOrigin::signed(bob),
				alice,
				clear_reject,
				secret
			),
			Error::<Test>::RevealNotBeingAccepted
		);
	})
}

#[test]
fn test_reveal_starts_after_threshold_no_of_verification_data_submissions() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register verifiers
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(alice),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(dave),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(charlie),
			alice,
			5
		));

		let clear_reject = b"REJECT".to_vec();
		let secret = b"123".to_vec();
		let combined_clear_secret = [&clear_reject[..], &secret[..]].concat();
		let hash_combined_clear_vec = keccak_256(&combined_clear_secret);
		let h256_hash_combined_clear_vec = H256::from_slice(&hash_combined_clear_vec);

		// Submit REJECT for the verification task of alice by three verifiers
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(dave),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(charlie),
			alice,
			h256_hash_combined_clear_vec
		));

		run_to_block(System::block_number() + 1);
		//Check that reveal is allowed at this moment as three verifiers have submitted hashed
		// consumer data
		assert!(matches!(
			VerificationProtocol::verification_requests(alice),
			Some(VerificationRequest {
				state: StateConfig { reveal: StateAttributes { state: true, .. }, .. },
				..
			})
		));
	})
}

#[test]
fn test_reveal() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		let mut verification_reqeust_data = create_verification_request(alice, doc_url.clone());
		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register verifiers
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		// Check the updated status of the verification request
		update_verification_request_on_allot(&mut verification_reqeust_data, 4);
		update_verification_request_start_ack(&mut verification_reqeust_data);
		update_verification_request_start_submit_vp(&mut verification_reqeust_data);

		// Check the verification task entry
		let mut expected_verification_process_data_bob: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(bob.clone(), System::block_number());

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(alice),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(dave),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(charlie),
			alice,
			5
		));

		expected_verification_process_data_bob.acknowledged = Some((System::block_number(), 5));

		verification_reqeust_data.state.ack.done_count_of_verifiers += 4;

		let clear_reject = b"REJECT".to_vec();
		let secret = b"123".to_vec();
		let combined_clear_secret = [&clear_reject[..], &secret[..]].concat();
		let hash_combined_clear_vec = keccak_256(&combined_clear_secret);
		let h256_hash_combined_clear_vec = H256::from_slice(&hash_combined_clear_vec);

		// Submit REJECT for the verification task of alice by three verifiers
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(dave),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(charlie),
			alice,
			h256_hash_combined_clear_vec
		));

		// update the expected verification request and process data
		expected_verification_process_data_bob.data =
			Some((System::block_number(), h256_hash_combined_clear_vec));
		verification_reqeust_data.state.submit_vp.done_count_of_verifiers += 3;
		//Reveal starts as minimum at submit stage reached
		update_verification_request_start_reveal(&mut verification_reqeust_data);

		run_to_block(System::block_number() + 1);
		//Check that reveal is allowed at this moment
		assert!(matches!(
			VerificationProtocol::verification_requests(alice),
			Some(VerificationRequest {
				state: StateConfig { reveal: StateAttributes { state: true, .. }, .. },
				..
			})
		));

		// Submit REJECT for the verification task of alice
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(bob),
			alice,
			clear_reject.clone(),
			secret.clone()
		));
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(dave),
			alice,
			clear_reject,
			secret
		));
		// update the expected verification request and process data after reveal
		expected_verification_process_data_bob.revealed_data =
			Some((System::block_number(), RevealedParameters::Reject));
		update_verification_request_on_reveal(&mut verification_reqeust_data, 2, 2);
		// Check the state of verification process record
		let stored_data = VerificationProtocol::verrification_process_records(alice, bob);
		assert_eq!(Some(expected_verification_process_data_bob), stored_data);

		// Check the state of verification request
		let stored_request = VerificationProtocol::verification_requests(alice);
		assert_eq!(Some(verification_reqeust_data), stored_request);
	})
}

#[test]
fn test_reveal_fails_without_submitting_hash_data_beforehand() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register verifiers
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(alice),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(dave),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(charlie),
			alice,
			5
		));

		let clear_reject = b"REJECT".to_vec();
		let secret = b"123".to_vec();
		let combined_clear_secret = [&clear_reject[..], &secret[..]].concat();
		let hash_combined_clear_vec = keccak_256(&combined_clear_secret);
		let h256_hash_combined_clear_vec = H256::from_slice(&hash_combined_clear_vec);

		// Submit REJECT for the verification task of alice by three verifiers
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(dave),
			alice,
			h256_hash_combined_clear_vec
		));

		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(alice),
			alice,
			h256_hash_combined_clear_vec
		));

		run_to_block(System::block_number() + 1);
		//Check that reveal is allowed at this moment
		assert!(matches!(
			VerificationProtocol::verification_requests(alice),
			Some(VerificationRequest {
				state: StateConfig { reveal: StateAttributes { state: true, .. }, .. },
				..
			})
		));

		// Submit clear data for the verification task of alice by charlie should fail
		assert_noop!(
			VerificationProtocol::reveal_data(
				RuntimeOrigin::signed(charlie),
				alice,
				clear_reject,
				secret
			),
			Error::<Test>::SubmitVpPending
		);
	})
}

#[test]
fn test_deletion_of_request_and_submissions_from_storage_at_the_end_of_process() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register verifiers
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(alice),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(dave),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(charlie),
			alice,
			5
		));

		let clear_reject = b"REJECT".to_vec();
		let secret = b"123".to_vec();
		let combined_clear_secret = [&clear_reject[..], &secret[..]].concat();
		let hash_combined_clear_vec = keccak_256(&combined_clear_secret);
		let h256_hash_combined_clear_vec = H256::from_slice(&hash_combined_clear_vec);

		// Submit REJECT for the verification task of alice by three verifiers
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(dave),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(charlie),
			alice,
			h256_hash_combined_clear_vec
		));

		run_to_block(System::block_number() + 1);

		// Submit REJECT for the verification task of alice
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(bob),
			alice,
			clear_reject.clone(),
			secret.clone()
		));
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(dave),
			alice,
			clear_reject,
			secret
		));

		run_to_block(System::block_number() + 1);
		// after the eval completion request should be deleted from the state
		// Check the state of verification request
		let stored_request = VerificationProtocol::verification_requests(alice);
		assert_eq!(stored_request, None);
		//Check deletion of verification records submitted by verifiers
		assert_eq!(VerificationProtocol::verrification_process_records(alice, alice), None);
		assert_eq!(VerificationProtocol::verrification_process_records(alice, bob), None);
		assert_eq!(VerificationProtocol::verrification_process_records(alice, dave), None);
		assert_eq!(VerificationProtocol::verrification_process_records(alice, charlie), None);
	})
}

#[test]
fn test_stored_result_upon_evaluate_verification_submissions_reject_case() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url.clone()
		));
		// Register verifiers
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(alice),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(dave),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(charlie),
			alice,
			5
		));

		let clear_reject = b"REJECT".to_vec();
		let secret = b"123".to_vec();
		let combined_clear_secret = [&clear_reject[..], &secret[..]].concat();
		let hash_combined_clear_vec = keccak_256(&combined_clear_secret);
		let h256_hash_combined_clear_vec = H256::from_slice(&hash_combined_clear_vec);

		// Submit valid consumer datails for the verification task of alice by three verifiers
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(dave),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(charlie),
			alice,
			h256_hash_combined_clear_vec
		));

		run_to_block(System::block_number() + 1);

		// Submit consumber details in clear format for the verification task of alice
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(bob),
			alice,
			clear_reject.clone(),
			secret.clone()
		));
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(dave),
			alice,
			clear_reject,
			secret
		));

		//verification result data after eval
		let result: VerificationResult<Test> = VerificationResult {
			consumer_account_id: alice,
			submitted_at: 1,
			completed_at: System::block_number(),
			list_of_documents: doc_url.try_into().unwrap(),
			did_creation_status: DidCreationStatus::Rejected,
			result: EvalVpResult::Rejected,
			stage: VerificationStages::Done,
		};

		run_to_block(System::block_number() + 1);

		// after the eval completion result should be created

		let stored_result = VerificationProtocol::verification_results(alice);
		assert_eq!(Some(result), stored_result);
	})
}

#[test]
fn test_reveal_fails_with_not_whitelisted_id_type() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];

		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url.clone()
		));
		// Register verifiers
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(alice),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(dave),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(charlie),
			alice,
			5
		));

		let (h256_hash_combined_clear_vec, clear_with_indian_passport, secret, _consumer_details) =
			generate_consumer_data_indian_passport();

		// Submit valid data for the verification task of alice by three verifiers
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(dave),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(charlie),
			alice,
			h256_hash_combined_clear_vec
		));

		run_to_block(System::block_number() + 1);

		// Submit consumer details in the clear text for the verification task of alice
		// It should fail as the ID document is not whitelisted
		assert_noop!(
			VerificationProtocol::reveal_data(
				RuntimeOrigin::signed(bob),
				alice,
				clear_with_indian_passport.clone(),
				secret.clone()
			),
			Error::<Test>::IdTypeNotDefined
		);
	})
}

#[test]
fn test_stored_result_upon_evaluate_verification_submissions_success_case() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		// Whitelist Indian Passport as a valid ID document
		let id_type =
			IdType::build(b"PASSPORT".to_vec(), b"GOVTOFINDIA".to_vec(), b"INDIA".to_vec())
				.expect("error in creatign id type");

		assert_ok!(VerificationProtocol::whitelist_id_type(
			RuntimeOrigin::signed(alice),
			id_type.clone()
		));
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url.clone()
		));
		// Register verifiers
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(alice),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(dave),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(charlie),
			alice,
			5
		));

		let (h256_hash_combined_clear_vec, clear_with_indian_passport, secret, consumer_details) =
			generate_consumer_data_indian_passport();

		// Submit valid data for the verification task of alice by three verifiers
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(dave),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(charlie),
			alice,
			h256_hash_combined_clear_vec
		));

		run_to_block(System::block_number() + 1);

		// Submit consumber details in clear format for the verification task of alice
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(bob),
			alice,
			clear_with_indian_passport.clone(),
			secret.clone()
		));
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(dave),
			alice,
			clear_with_indian_passport,
			secret
		));

		//verification result data after eval
		let result: VerificationResult<Test> = VerificationResult {
			consumer_account_id: alice,
			submitted_at: 1,
			completed_at: System::block_number(),
			list_of_documents: doc_url.try_into().unwrap(),
			did_creation_status: DidCreationStatus::Created,
			result: EvalVpResult::Accepted(consumer_details),
			stage: VerificationStages::Done,
		};

		run_to_block(System::block_number() + 1);

		// after the eval completion result should be created

		let stored_result = VerificationProtocol::verification_results(alice);
		assert_eq!(Some(result), stored_result);
	})
}

#[test]
fn test_evaluate_verification_submissions_fails_as_upon_resubmission_of_the_same_document() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		// Whitelist Indian Passport as a valid ID document
		let id_type =
			IdType::build(b"PASSPORT".to_vec(), b"GOVTOFINDIA".to_vec(), b"INDIA".to_vec())
				.expect("error in creatign id type");

		assert_ok!(VerificationProtocol::whitelist_id_type(
			RuntimeOrigin::signed(alice),
			id_type.clone()
		));
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};

		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(bob),
			doc_url.clone()
		));

		// Register verifiers
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(alice),
			bob,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			bob,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(dave),
			bob,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(charlie),
			bob,
			5
		));

		let (h256_hash_combined_clear_vec, clear_with_indian_passport, secret, consumer_details) =
			generate_consumer_data_indian_passport();
		// Dummy insert the details of alice
		for hash in consumer_details.hashes().iter() {
			ConsumerHashes::<Test>::insert(&hash, (alice.clone(), 10));
		}

		// Submit valid data for the verification task of alice by three verifiers
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			bob,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(dave),
			bob,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(charlie),
			bob,
			h256_hash_combined_clear_vec
		));

		run_to_block(System::block_number() + 1);

		// Submit consumber details in clear format for the verification task of bob
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(bob),
			bob,
			clear_with_indian_passport.clone(),
			secret.clone()
		));
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(dave),
			bob,
			clear_with_indian_passport,
			secret
		));
		//verification result data after eval
		// Evaluates to valid result but fails at Didcreation because some one else(alice) already
		// has used the document to verify. so bob can not use it
		let result: VerificationResult<Test> = VerificationResult {
			consumer_account_id: bob,
			submitted_at: 1,
			completed_at: System::block_number(),
			list_of_documents: doc_url.try_into().unwrap(),
			did_creation_status: DidCreationStatus::RejectedDuplicate,
			result: EvalVpResult::Accepted(consumer_details),
			stage: VerificationStages::Done,
		};

		run_to_block(System::block_number() + 1);

		// after the eval completion result should be created

		let stored_result = VerificationProtocol::verification_results(bob);
		assert_eq!(Some(result), stored_result);
	})
}

#[test]
fn test_resubmission_of_verification_request_after_rejected_duplicate() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let doc_url = vec![0u8; 100];

		let (.., consumer_details) = generate_consumer_data_indian_passport();

		//Dummy verification result data after eval
		let result: VerificationResult<Test> = VerificationResult {
			consumer_account_id: alice,
			submitted_at: 1,
			completed_at: 2,
			list_of_documents: doc_url.clone().try_into().unwrap(),
			did_creation_status: DidCreationStatus::RejectedDuplicate,
			result: EvalVpResult::Accepted(consumer_details),
			stage: VerificationStages::Done,
		};
		//Dummy insert the rejected result
		VerificationResults::<Test>::insert(alice, result);

		run_to_block(System::block_number() + 1);

		// Register agian a verification request- should succeed
		// cooling period to be inserted
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url.clone()
		));
	})
}

#[test]
fn test_resubmission_of_verification_request_in_case_of_already_verified() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let doc_url = vec![0u8; 100];

		//Insert dummy Did Verified status
		assert_ok!(<Test as pallet_verification_protocol::Config>::DidProvider::creat_new_did(
			&alice
		));
		let (.., consumer_details) = generate_consumer_data_indian_passport();
		// Dummy insert the details of alice
		for hash in consumer_details.hashes().iter() {
			ConsumerHashes::<Test>::insert(&hash, (alice.clone(), 10));
		}

		//verification result data after successful eval
		let result: VerificationResult<Test> = VerificationResult {
			consumer_account_id: alice,
			submitted_at: 1,
			completed_at: System::block_number(),
			list_of_documents: doc_url.clone().try_into().unwrap(),
			did_creation_status: DidCreationStatus::Created,
			result: EvalVpResult::Accepted(consumer_details),
			stage: VerificationStages::Done,
		};

		VerificationResults::<Test>::insert(alice, result);
		run_to_block(System::block_number() + 1);

		// Register agian a verification request fails
		assert_noop!(
			VerificationProtocol::submit_did_creation_request(
				RuntimeOrigin::signed(alice),
				doc_url.clone()
			),
			Error::<Test>::AlreadyCreated4Account
		);
	})
}

// threshold wining percentage set to 66%
// one out of four submits Reject (25%)
// Three out of four submits accept with same data (75%)
#[test]
fn test_threshold_winning_percentage_clear_majority_case() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		// Whitelist Indian Passport as a valid ID document
		let id_type =
			IdType::build(b"PASSPORT".to_vec(), b"GOVTOFINDIA".to_vec(), b"INDIA".to_vec())
				.expect("error in creatign id type");

		assert_ok!(VerificationProtocol::whitelist_id_type(
			RuntimeOrigin::signed(alice),
			id_type.clone()
		));
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 4,
			min_count_at_reveal_stage: 4,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url.clone()
		));
		// Register verifiers
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(alice),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(dave),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(charlie),
			alice,
			5
		));

		let (h256_hash_combined_clear_vec, clear_with_indian_passport, secret, consumer_details) =
			generate_consumer_data_indian_passport();
		let clear_reject = b"REJECT".to_vec();
		let combined_clear_reject_with_secret = [&clear_reject[..], &secret[..]].concat();
		let hash_combined_clear_reject_with_secret = keccak_256(&combined_clear_reject_with_secret);
		let h256_combined_clear_reject_with_secret =
			H256::from_slice(&hash_combined_clear_reject_with_secret);

		// Alice submits Reject
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(alice),
			alice,
			h256_combined_clear_reject_with_secret
		));
		// Submit valid data for the verification task of alice by three verifiers
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(dave),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(charlie),
			alice,
			h256_hash_combined_clear_vec
		));

		run_to_block(System::block_number() + 1);

		// submit clear Reject data
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(alice),
			alice,
			clear_reject.clone(),
			secret.clone()
		));
		// Submit consumber details in clear format for the verification task of alice
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(bob),
			alice,
			clear_with_indian_passport.clone(),
			secret.clone()
		));
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(dave),
			alice,
			clear_with_indian_passport.clone(),
			secret.clone()
		));
		assert_ok!(VerificationProtocol::reveal_data(
			RuntimeOrigin::signed(charlie),
			alice,
			clear_with_indian_passport,
			secret
		));

		//verification result data after eval should be following as 75% is above threshold 66%
		let result: VerificationResult<Test> = VerificationResult {
			consumer_account_id: alice,
			submitted_at: 1,
			completed_at: System::block_number(),
			list_of_documents: doc_url.try_into().unwrap(),
			did_creation_status: DidCreationStatus::Created,
			result: EvalVpResult::Accepted(consumer_details),
			stage: VerificationStages::Done,
		};

		run_to_block(System::block_number() + 1);

		// after the eval completion result should be created

		let stored_result = VerificationProtocol::verification_results(alice);
		assert_eq!(Some(result), stored_result);
	})
}

#[test]
fn test_after_reveal_start_ack_can_not_be_done() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register verifiers
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(dave),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(charlie),
			alice,
			5
		));

		let clear_reject = b"REJECT".to_vec();
		let secret = b"123".to_vec();
		let combined_clear_secret = [&clear_reject[..], &secret[..]].concat();
		let hash_combined_clear_vec = keccak_256(&combined_clear_secret);
		let h256_hash_combined_clear_vec = H256::from_slice(&hash_combined_clear_vec);

		// Submit REJECT for the verification task of alice by three verifiers
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(dave),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(charlie),
			alice,
			h256_hash_combined_clear_vec
		));

		run_to_block(System::block_number() + 1);
		//Confirm that reveal has started at this moment
		assert!(matches!(
			VerificationProtocol::verification_requests(alice),
			Some(VerificationRequest {
				state: StateConfig { reveal: StateAttributes { state: true, .. }, .. },
				..
			})
		));
		//Trying to accept the task now should fail
		assert_noop!(
			VerificationProtocol::accept_verification_task(RuntimeOrigin::signed(alice), alice, 5),
			Error::<Test>::AckNotBeingAccepted
		);
	})
}

#[test]
fn test_after_reveal_start_submit_vp_can_not_be_done() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register verifiers
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(alice),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(dave),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(charlie),
			alice,
			5
		));

		let clear_reject = b"REJECT".to_vec();
		let secret = b"123".to_vec();
		let combined_clear_secret = [&clear_reject[..], &secret[..]].concat();
		let hash_combined_clear_vec = keccak_256(&combined_clear_secret);
		let h256_hash_combined_clear_vec = H256::from_slice(&hash_combined_clear_vec);

		// Submit REJECT for the verification task of alice by three verifiers
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(dave),
			alice,
			h256_hash_combined_clear_vec
		));
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(charlie),
			alice,
			h256_hash_combined_clear_vec
		));

		run_to_block(System::block_number() + 1);
		//Confirm that reveal has started at this moment
		assert!(matches!(
			VerificationProtocol::verification_requests(alice),
			Some(VerificationRequest {
				state: StateConfig { reveal: StateAttributes { state: true, .. }, .. },
				..
			})
		));

		// Trying to submit hashed data now should fail
		assert_noop!(
			VerificationProtocol::submit_verification_data(
				RuntimeOrigin::signed(alice),
				alice,
				h256_hash_combined_clear_vec
			),
			Error::<Test>::VpNotBeingAccepted
		);
	})
}

#[test]
fn test_verification_request_state_upon_allot_to_verifiers_at_different_blocks() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let dave = account_key("//Dave");
		let charlie = account_key("//Charlie");
		let deposit = 100u128 * 10u128.pow(12);
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		let mut verification_reqeust_data = create_verification_request(alice, doc_url.clone());
		// Register a verification request
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			doc_url
		));
		// Register verifiers
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(alice), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));

		// Check the updated status of the verification request
		update_verification_request_on_allot(&mut verification_reqeust_data, 2);
		update_verification_request_start_ack(&mut verification_reqeust_data);
		update_verification_request_start_submit_vp(&mut verification_reqeust_data);

		// Check the verification task entry
		let mut expected_verification_process_data_bob: VerificationProcessData<Test> =
			VerificationProcessData::allot_to_verifier(bob.clone(), System::block_number());

		run_to_block(System::block_number() + 1);

		// Accept the verification task
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(alice),
			alice,
			5
		));
		assert_ok!(VerificationProtocol::accept_verification_task(
			RuntimeOrigin::signed(bob),
			alice,
			5
		));

		expected_verification_process_data_bob.acknowledged = Some((System::block_number(), 5));

		verification_reqeust_data.state.ack.done_count_of_verifiers += 2;

		let clear_reject = b"REJECT".to_vec();
		let secret = b"123".to_vec();
		let combined_clear_secret = [&clear_reject[..], &secret[..]].concat();
		let hash_combined_clear_vec = keccak_256(&combined_clear_secret);
		let h256_hash_combined_clear_vec = H256::from_slice(&hash_combined_clear_vec);

		// Submit REJECT for the verification task of alice by three verifiers
		assert_ok!(VerificationProtocol::submit_verification_data(
			RuntimeOrigin::signed(bob),
			alice,
			h256_hash_combined_clear_vec
		));

		// update the expected verification request and process data
		expected_verification_process_data_bob.data =
			Some((System::block_number(), h256_hash_combined_clear_vec));
		verification_reqeust_data.state.submit_vp.done_count_of_verifiers += 1;

		run_to_block(System::block_number() + 1);
		// Now two more verifiers register and they will get the task

		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(dave), deposit));
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(charlie), deposit));

		// updated status of the verification request
		update_verification_request_on_allot(&mut verification_reqeust_data, 2);

		run_to_block(System::block_number() + 1);

		// Check the state of verification process record
		let stored_data = VerificationProtocol::verrification_process_records(alice, bob);
		assert_eq!(Some(expected_verification_process_data_bob), stored_data);

		// Check the state of verification request
		let stored_request = VerificationProtocol::verification_requests(alice);
		assert_eq!(Some(verification_reqeust_data), stored_request);
	})
}
