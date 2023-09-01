use super::mock::*;

#[test]
fn test_multiple_round_starts_only_if_round_1_deliberated() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 10,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));
		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			vec![0u8; 100]
		));
		let verification_reqeust_data = create_verification_request(alice, doc_url);
		// Going to future beyond waiting time
		// It should not increase the round as round 1 not started by allocating to at least one
		// verifier
		run_to_block(120);
		let stored_request = VerificationProtocol::verification_requests(alice);
		assert_eq!(Some(verification_reqeust_data), stored_request);
	});
}

#[test]
fn test_verification_request_state_at_round_2() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let deposit = 100u128 * 10u128.pow(12);
		// Register a verifier
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 10,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			vec![0u8; 100]
		));
		let mut verification_reqeust_data = create_verification_request(alice, doc_url);
		// Update the status of the verification request
		update_verification_request_on_allot(&mut verification_reqeust_data, 1);
		update_verification_request_start_ack(&mut verification_reqeust_data);
		update_verification_request_start_submit_vp(&mut verification_reqeust_data);

		// Going to future beyond waiting time 1+10=11
		// It should increase the round as round to 2 at the end of 12th block
		run_to_block(13);
		update_verification_request_to_round_2_from_1(&mut verification_reqeust_data);
		let stored_request = VerificationProtocol::verification_requests(alice);
		assert_eq!(Some(verification_reqeust_data), stored_request);
	});
}

#[test]
fn test_verification_request_state_at_round_n() {
	let n = 5;
	ExtBuilder::default().build().execute_with(|| {
		let alice = account_key("//Alice");
		let bob = account_key("//Bob");
		let deposit = 100u128 * 10u128.pow(12);
		// Register a verifier
		assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));
		let doc_url = vec![0u8; 100];
		let new_parameters = VerificationProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 4,
			min_count_at_ack_accept_stage: 4,
			min_count_at_submit_vp_stage: 3,
			min_count_at_reveal_stage: 2,
			max_waiting_time_at_stages: 10,
			threshold_winning_percentage: 66,
		};
		assert_ok!(VerificationProtocol::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));

		assert_ok!(VerificationProtocol::submit_did_creation_request(
			RuntimeOrigin::signed(alice),
			vec![0u8; 100]
		));
		let mut verification_reqeust_data = create_verification_request(alice, doc_url);
		// Update the status of the verification request
		update_verification_request_on_allot(&mut verification_reqeust_data, 1);
		update_verification_request_start_ack(&mut verification_reqeust_data);
		update_verification_request_start_submit_vp(&mut verification_reqeust_data);
		for _ in 0..n {
			update_verification_request_to_go_to_next_round(&mut verification_reqeust_data);
		}
		let to_block = verification_reqeust_data.state.submit_vp.started_at +
			verification_reqeust_data.state.submit_vp.state_duration as u64;

		run_to_block(to_block + 1);
		// update_verification_request_to_round_2_from_1(&mut verification_reqeust_data);
		let stored_request = VerificationProtocol::verification_requests(alice);
		assert_eq!(Some(verification_reqeust_data), stored_request);
		// Confirm that the round number is 5(n) + 1 = 6
		assert!(matches!(stored_request, Some(VerificationRequest { round_number: 6, .. })));
	});
}

#[test]
fn test_stored_result_in_case_verification_process_completed_in_round_3() {
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
		let mut verification_reqeust_data = create_verification_request(alice, doc_url.clone());
		// Update the status of the verification request
		update_verification_request_on_allot(&mut verification_reqeust_data, 1);
		update_verification_request_start_ack(&mut verification_reqeust_data);
		update_verification_request_start_submit_vp(&mut verification_reqeust_data);
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
		// only two out of min 3 required submit vp done.
		//then lets wait for it go into round num 3
		for _ in 0..2 {
			update_verification_request_to_go_to_next_round(&mut verification_reqeust_data);
		}
		// a block where it would be in the round num 3
		let to_block = verification_reqeust_data.state.submit_vp.started_at +
			verification_reqeust_data.state.submit_vp.state_duration as u64;

		run_to_block(to_block + 1);
		// Now third VP is submitted and it should proceed and complete
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
