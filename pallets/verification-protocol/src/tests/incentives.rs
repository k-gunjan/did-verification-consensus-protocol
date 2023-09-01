use super::mock::*;
use sp_runtime::FixedU128;
use verifiers::{
	types::{Verifier, VerifierState},
	VerifiersProvider,
};

// pub struct VerifierUpdateData {
// 	// account_id: A,
// 	pub incentive_factor: FixedU128,
// 	pub increment: Increment,
// }
// pub enum Increment {
// 	Accepted(u8),
// 	UnAccepted(u8),
// 	NotCompleted(u8),
// }

#[test]
fn test_register_verifier_success() {
	let initial_balance = 1000_000_000_000_000;
	let deposit = 100u128 * 10u128.pow(12);
	ExtBuilder::default()
		.balances(&[(account_key("//Bob"), initial_balance)])
		.build()
		.execute_with(|| {
			let bob = account_key("//Bob");
			let pallet_account = Verifiers::account_id();

			assert_eq!(Balances::free_balance(&bob), initial_balance);
			assert_eq!(Balances::free_balance(&pallet_account), 0);
			// Register a verifier
			assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit));

			assert_eq!(Balances::free_balance(&bob), initial_balance - deposit);
			assert_eq!(Balances::free_balance(&pallet_account), deposit);

			assert!(matches!(
				Verifiers::verifiers(bob),
				Some(Verifier {
					balance: 100_000_000_000_000,
					count_of_accepted_submissions: 0,
					count_of_un_accepted_submissions: 0,
					count_of_incompleted_processes: 0,
					threshold_breach_at: None,
					..
				})
			))
		})
}

#[test]
fn test_register_verifier_error_insufficient_balance() {
	let initial_balance = 10_000_000_000_000;
	let deposit = 100u128 * 10u128.pow(12);

	ExtBuilder::default()
		.balances(&[(account_key("//Bob"), initial_balance)])
		.build()
		.execute_with(|| {
			let bob = account_key("//Bob");

			// Register a verifier
			assert_noop!(
				Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit),
				pallet_balances::Error::<Test, _>::InsufficientBalance
			);
		})
}

#[test]
fn test_register_verifier_success_verifier_state_higher_deposit() {
	let initial_balance = 1000_000_000_000_000;
	let deposit = 100u128 * 10u128.pow(12);

	ExtBuilder::default()
		.balances(&[(account_key("//Bob"), initial_balance)])
		.build()
		.execute_with(|| {
			let bob = account_key("//Bob");
			let verifier_protocol_parameters = Verifiers::protocol_parameters();

			assert!(deposit >= verifier_protocol_parameters.minimum_deposit_for_being_active);
			// Register a verifier
			assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit),);

			assert!(matches!(
				Verifiers::verifiers(bob),
				Some(Verifier { state: VerifierState::Active, .. })
			));
		})
}

#[test]
fn test_register_verifier_success_verifier_state_low_deposit() {
	let initial_balance = 1000_000_000_000_000;
	let deposit = 99u128 * 10u128.pow(12);

	ExtBuilder::default()
		.balances(&[(account_key("//Bob"), initial_balance)])
		.build()
		.execute_with(|| {
			let bob = account_key("//Bob");
			let verifier_protocol_parameters = Verifiers::protocol_parameters();

			assert!(deposit < verifier_protocol_parameters.minimum_deposit_for_being_active);
			// Register a verifier
			assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit),);

			assert!(matches!(
				Verifiers::verifiers(bob),
				Some(Verifier { state: VerifierState::Pending, balance: 99_000_000_000_000, .. })
			));
		})
}

#[test]
fn test_register_verifier_success_verifier_state_transition_from_pending_to_active() {
	let initial_balance = 1000_000_000_000_000;
	let deposit = 99u128 * 10u128.pow(12);
	let second_deposit = 5u128 * 10u128.pow(12);

	ExtBuilder::default()
		.balances(&[(account_key("//Bob"), initial_balance)])
		.build()
		.execute_with(|| {
			let bob = account_key("//Bob");
			let verifier_protocol_parameters = Verifiers::protocol_parameters();

			assert!(deposit < verifier_protocol_parameters.minimum_deposit_for_being_active);
			// Register a verifier
			assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit),);

			assert!(matches!(
				Verifiers::verifiers(bob),
				Some(Verifier { state: VerifierState::Pending, .. })
			));

			run_to_block(2);
			//Deposit more fund
			assert_ok!(Verifiers::verifier_deposit(RuntimeOrigin::signed(bob), second_deposit),);
			run_to_block(3);
			// Confirm the new state
			assert!(matches!(
				Verifiers::verifiers(bob),
				Some(Verifier { state: VerifierState::Active, balance: 104_000_000_000_000, .. })
			));
			println!("{:?}", Verifiers::verifiers(bob));
		})
}

#[test]
fn test_verifier_submits_accepted_data_success_counter_increments() {
	let initial_balance = 1000_000_000_000_000;
	let deposit = 100u128 * 10u128.pow(12);
	let num_of_tasks = 5;
	let update_data = VerifierUpdateData {
		incentive_factor: 1.into(),
		increment: Increment::Accepted(num_of_tasks),
	};

	ExtBuilder::default()
		.balances(&[(account_key("//Bob"), initial_balance)])
		.build()
		.execute_with(|| {
			let bob = account_key("//Bob");

			// Register a verifier
			assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit),);

			// Confirm the counter is zero
			assert!(matches!(
				Verifiers::verifiers(bob),
				Some(Verifier {count_of_accepted_submissions: 0, .. })
			));

			let mut data = Vec::new();
			data.push((bob, update_data));

			run_to_block(2);
			//Deposit an action data with 5 accepted
			assert_ok!(<Test as pallet_verification_protocol::Config>::VerifiersProvider::update_verifier_profiles(data, 2));


			// Confirm the new state
			assert!(matches!(
				Verifiers::verifiers(bob),
				Some(Verifier { count_of_accepted_submissions: 5, .. })
			));

		})
}

#[test]
fn test_verifier_submits_un_accepted_data_success_counter_increments() {
	let initial_balance = 1000_000_000_000_000;
	let deposit = 100u128 * 10u128.pow(12);
	let num_of_tasks = 5;
	let update_data = VerifierUpdateData {
		incentive_factor: 1.into(),
		increment: Increment::UnAccepted(num_of_tasks),
	};

	ExtBuilder::default()
		.balances(&[(account_key("//Bob"), initial_balance)])
		.build()
		.execute_with(|| {
			let bob = account_key("//Bob");

			// Register a verifier
			assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit),);

			// Confirm the counter is zero
			assert!(matches!(
				Verifiers::verifiers(bob),
				Some(Verifier {count_of_un_accepted_submissions: 0, .. })
			));

			let mut data = Vec::new();
			data.push((bob, update_data));

			run_to_block(2);
			//Deposit an action data with 5 un-accepted
			assert_ok!(<Test as pallet_verification_protocol::Config>::VerifiersProvider::update_verifier_profiles(data, 2));


			// Confirm the new state
			assert!(matches!(
				Verifiers::verifiers(bob),
				Some(Verifier { count_of_un_accepted_submissions: 5, .. })
			));

		})
}

#[test]
fn test_verifier_not_completed_process_success_counter_increments() {
	let initial_balance = 1000_000_000_000_000;
	let deposit = 100u128 * 10u128.pow(12);
	let num_of_tasks = 5;
	let update_data = VerifierUpdateData {
		incentive_factor: 1.into(),
		increment: Increment::NotCompleted(num_of_tasks),
	};

	ExtBuilder::default()
		.balances(&[(account_key("//Bob"), initial_balance)])
		.build()
		.execute_with(|| {
			let bob = account_key("//Bob");

			// Register a verifier
			assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit),);

			// Confirm the counter is zero
			assert!(matches!(
				Verifiers::verifiers(bob),
				Some(Verifier {count_of_incompleted_processes: 0, .. })
			));

			let mut data = Vec::new();
			data.push((bob, update_data));

			run_to_block(2);
			//Deposit an action data with 5 not completed
			assert_ok!(<Test as pallet_verification_protocol::Config>::VerifiersProvider::update_verifier_profiles(data, 2));


			// Confirm the new state
			assert!(matches!(
				Verifiers::verifiers(bob),
				Some(Verifier { count_of_incompleted_processes: 5, .. })
			));

		})
}

#[test]
fn test_verifier_not_completed_process_success_deposit_slashes() {
	let initial_balance = 1000_000_000_000_000;
	let deposit = 100u128 * 10u128.pow(12);
	let num_of_tasks = 1;
	let confidence_score: u128 = 1;
	let update_data = VerifierUpdateData {
		incentive_factor: FixedU128::from_rational(confidence_score, 100u128),
		increment: Increment::NotCompleted(num_of_tasks),
	};
	let update_data_dummy =
		VerifierUpdateData { incentive_factor: 1.into(), increment: Increment::UnAccepted(5) };
	let verifier_protocol_parameters = VerifierProtocolParameterValues {
		minimum_deposit_for_being_active: 100_000_000_000_000,
		threshold_accuracy_score: FixedI64::from_inner(85),
		penalty_waiver_score: FixedI64::from_inner(95),
		// resemption period in number of blocks
		resumption_waiting_period: 100,
		reward_amount: 10_000_000_000_000,
		penalty_amount: 10_000_000_000_000,
		penalty_amount_not_completed: 15_000_000_000_000,
		accuracy_weight: FixedI64::from_inner(1),
		reputation_score_weight: FixedI64::from_inner(1),
	};

	ExtBuilder::default()
		.balances(&[(account_key("//Bob"), initial_balance)])
		.build()
		.execute_with(|| {
			let alice = account_key("//Alice");
			assert_ok!(Verifiers::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			verifier_protocol_parameters.clone()
		));

			let bob = account_key("//Bob");
			// Register a verifier
			assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit),);

			let mut data_dummy = Vec::new();
			data_dummy.push((bob.clone(), update_data_dummy));

			//Deposit an action data with 5 not completed to degrade the profile so that penalty can be applied
			assert_ok!(<Test as pallet_verification_protocol::Config>::VerifiersProvider::update_verifier_profiles(data_dummy, 1));
			assert!(match Verifiers::verifiers(bob) {
				Some(v @ Verifier {
					balance: 100_000_000_000_000,
					count_of_accepted_submissions:0,
					count_of_un_accepted_submissions:5,
					count_of_incompleted_processes: 0,
					 .. }) => v.accuracy()< verifier_protocol_parameters.penalty_waiver_score,
				_=> false

			});

			let mut data = Vec::new();
			data.push((bob, update_data.clone()));

			run_to_block(2);
			//Deposit an action data with 1 not completed
			assert_ok!(<Test as pallet_verification_protocol::Config>::VerifiersProvider::update_verifier_profiles(data, 2));

			println!("verifier: {:?}", Verifiers::verifiers(bob));

			// Confirm the new state with reduced deposit value by 15*(1 + 1%)
			assert!(match  Verifiers::verifiers(bob) {

				Some(Verifier { balance , .. })  => 84_850_000_000_000 == balance,
				_ => false
			}
			);

		})
}

#[test]
fn test_verifier_un_accepted_submission_success_deposit_slashes() {
	let initial_balance = 1000_000_000_000_000;
	let deposit = 100u128 * 10u128.pow(12);
	let num_of_tasks = 1;
	let confidence_score: u128 = 1;
	let update_data = VerifierUpdateData {
		incentive_factor: FixedU128::from_rational(confidence_score, 100u128),
		increment: Increment::UnAccepted(num_of_tasks),
	};
	let update_data_dummy =
		VerifierUpdateData { incentive_factor: 1.into(), increment: Increment::UnAccepted(5) };
	let verifier_protocol_parameters = VerifierProtocolParameterValues {
		minimum_deposit_for_being_active: 100_000_000_000_000,
		threshold_accuracy_score: FixedI64::from_inner(85),
		penalty_waiver_score: FixedI64::from_inner(95),
		// resemption period in number of blocks
		resumption_waiting_period: 100,
		reward_amount: 10_000_000_000_000,
		penalty_amount: 10_000_000_000_000,
		penalty_amount_not_completed: 15_000_000_000_000,
		accuracy_weight: FixedI64::from_inner(1),
		reputation_score_weight: FixedI64::from_inner(1),
	};

	ExtBuilder::default()
		.balances(&[(account_key("//Bob"), initial_balance)])
		.build()
		.execute_with(|| {
			let alice = account_key("//Alice");

			// let verifier_protocol_parameters = Verifiers::protocol_parameters();
			assert_ok!(Verifiers::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			verifier_protocol_parameters.clone()
		));

			let bob = account_key("//Bob");
			// Register a verifier
			assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit),);

			let mut data_dummy = Vec::new();
			data_dummy.push((bob.clone(), update_data_dummy));

			//Deposit an action data with 5 not completed to degrade the profile so that penalty can be applied
			assert_ok!(<Test as pallet_verification_protocol::Config>::VerifiersProvider::update_verifier_profiles(data_dummy, 1));
			assert!(match Verifiers::verifiers(bob) {
				Some(v @ Verifier {
					balance: 100_000_000_000_000,
					count_of_accepted_submissions:0,
					count_of_un_accepted_submissions:5,
					count_of_incompleted_processes: 0,
					 .. }) => v.accuracy()< verifier_protocol_parameters.penalty_waiver_score,
				_=> false

			});

			let mut data = Vec::new();
			data.push((bob, update_data.clone()));

			run_to_block(2);
			//Deposit an action data with 1 not completed
			assert_ok!(<Test as pallet_verification_protocol::Config>::VerifiersProvider::update_verifier_profiles(data, 2));

			println!("verifier: {:?}", Verifiers::verifiers(bob));

			// Confirm the new state with reduced deposit value by 10*(1 + 1%)
			assert!(match  Verifiers::verifiers(bob) {

				Some(Verifier { balance , .. })  => 89_900_000_000_000 == balance,
				_ => false
			}
			);

		})
}

#[test]
fn test_verifier_submits_accepted_data_success_reward_check() {
	let initial_balance = 1000_000_000_000_000;
	let deposit = 100u128 * 10u128.pow(12);
	let num_of_tasks = 1;
	let confidence_score: u128 = 1;
	let update_data = VerifierUpdateData {
		incentive_factor: FixedU128::from_rational(confidence_score, 100u128),
		increment: Increment::Accepted(num_of_tasks),
	};
	let verifier_protocol_parameters = VerifierProtocolParameterValues {
		minimum_deposit_for_being_active: 100_000_000_000_000,
		threshold_accuracy_score: FixedI64::from_inner(85),
		penalty_waiver_score: FixedI64::from_inner(95),
		// resemption period in number of blocks
		resumption_waiting_period: 100,
		reward_amount: 10_000_000_000_000,
		penalty_amount: 10_000_000_000_000,
		penalty_amount_not_completed: 15_000_000_000_000,
		accuracy_weight: FixedI64::from_inner(1),
		reputation_score_weight: FixedI64::from_inner(1),
	};

	ExtBuilder::default()
		.balances(&[(account_key("//Bob"), initial_balance)])
		.build()
		.execute_with(|| {
			let alice = account_key("//Alice");

			// let verifier_protocol_parameters = Verifiers::protocol_parameters();
			assert_ok!(Verifiers::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			verifier_protocol_parameters.clone()
		));

			let bob = account_key("//Bob");

			// Register a verifier
			assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit),);

			// Confirm the counter is zero and balance is deposit
			assert!(matches!(
				Verifiers::verifiers(bob),
				Some(Verifier {count_of_incompleted_processes: 0, count_of_accepted_submissions:0, count_of_un_accepted_submissions:0, balance: 100_000_000_000_000, .. })
			));

			let mut data = Vec::new();
			data.push((bob, update_data.clone()));

			run_to_block(2);
			//Deposit an action data with 1 accepted
			assert_ok!(<Test as pallet_verification_protocol::Config>::VerifiersProvider::update_verifier_profiles(data, 2));

			println!("verifier: {:?}", Verifiers::verifiers(bob));

			// Confirm the new state
			assert!(match  Verifiers::verifiers(bob) {

				Some(Verifier { count_of_accepted_submissions: 1, balance , .. })  => 110_100_000_000_000 == balance,
				_ => false
			}
			);

		})
}

#[test]
fn test_verifier_submits_accepted_data_success_reward_check_while_in_cooling_period() {
	let initial_balance = 1000_000_000_000_000;
	let deposit = 100u128 * 10u128.pow(12);
	let num_of_tasks = 1;
	let confidence_score: u128 = 1;
	let update_data = VerifierUpdateData {
		incentive_factor: FixedU128::from_rational(confidence_score, 100u128),
		increment: Increment::Accepted(num_of_tasks),
	};
	let update_data_dummy =
		VerifierUpdateData { incentive_factor: 1.into(), increment: Increment::UnAccepted(1) };
	let update_data_dummy_2 =
		VerifierUpdateData { incentive_factor: 1.into(), increment: Increment::Accepted(10) };
	let verifier_protocol_parameters = VerifierProtocolParameterValues {
		minimum_deposit_for_being_active: 100_000_000_000_000,
		threshold_accuracy_score: FixedI64::from_inner(85),
		penalty_waiver_score: FixedI64::from_inner(95),
		// resemption period in number of blocks
		resumption_waiting_period: 100,
		reward_amount: 10_000_000_000_000,
		penalty_amount: 10_000_000_000_000,
		penalty_amount_not_completed: 15_000_000_000_000,
		accuracy_weight: FixedI64::from_inner(1),
		reputation_score_weight: FixedI64::from_inner(1),
	};

	ExtBuilder::default()
		.balances(&[(account_key("//Bob"), initial_balance)])
		.build()
		.execute_with(|| {
			let alice = account_key("//Alice");

			// let verifier_protocol_parameters = Verifiers::protocol_parameters();
			assert_ok!(Verifiers::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			verifier_protocol_parameters.clone()
		));

			let bob = account_key("//Bob");

			// Register a verifier
			assert_ok!(Verifiers::register_verifier(RuntimeOrigin::signed(bob), deposit),);

			let mut data_dummy = Vec::new();
			data_dummy.push((bob.clone(), update_data_dummy));

			//Deposit an action data with 1 not accepted to degrade the profile so that penalty can be applied
			assert_ok!(<Test as pallet_verification_protocol::Config>::VerifiersProvider::update_verifier_profiles(data_dummy, 1));
			println!("verifier: {:?}", Verifiers::verifiers(bob));

			assert!(match Verifiers::verifiers(bob) {
				Some(v @ Verifier {
					balance: 100_000_000_000_000,
					threshold_breach_at: Some(1),
					 .. }) => v.accuracy() < verifier_protocol_parameters.threshold_accuracy_score,
				_=> false

			});

			run_to_block(2);
			let mut data_dummy_2 = Vec::new();
			data_dummy_2.push((bob.clone(), update_data_dummy_2));

			//Deposit an action data with 10 accepted to enhance the  profile verifier is elligible to receive reward accuracy wise 
			assert_ok!(<Test as pallet_verification_protocol::Config>::VerifiersProvider::update_verifier_profiles(data_dummy_2, 2));
			println!("verifier: {:?}", Verifiers::verifiers(bob));

			run_to_block(3);
			assert!(match Verifiers::verifiers(bob) {
				Some(v @ Verifier {
					balance: 100_000_000_000_000,
					threshold_breach_at: Some(1),
					 .. }) => v.accuracy() > verifier_protocol_parameters.threshold_accuracy_score,
				_=> false

			});

			// Now on next activity even verifier is elligible for reward accuracy wise, it wont receive as its in cooling period until block 101
		    	let mut data = Vec::new();
			data.push((bob, update_data.clone()));

			//Deposit an action data with 1 completed
			assert_ok!(<Test as pallet_verification_protocol::Config>::VerifiersProvider::update_verifier_profiles(data, 3));

			println!("verifier: {:?}", Verifiers::verifiers(bob));
			// Assert that accuracy score is above threshold and in cooling period
			assert!(match Verifiers::verifiers(bob) {
				Some(v @ Verifier {
					 balance,
					threshold_breach_at: Some(1),// cooling period
					 .. }) if v.accuracy() > verifier_protocol_parameters.threshold_accuracy_score => 100_000_000_000_000 == balance,
				_=> false

			});


		})
}
