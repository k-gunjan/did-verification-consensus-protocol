use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

// #[test]
// fn it_works_for_default_value() {
// 	new_test_ext().execute_with(|| {
// 		// Dispatch a signed extrinsic.
// 		// assert_ok!(AdoptionModule::do_something(RuntimeOrigin::adoption_event_types_creation("abcdefg"
// .to_vec(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_vec()), 42)); 		// Read pallet
// storage and assert an expected result. 		// assert_eq!(AdoptionModule::something(), Some(42));
// 		assert_ok!(1,1);
// 	});
// }

// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(
// 			AdoptionModule::cause_error(RuntimeOrigin::signed(1)),
// 			Error::<Test>::NoneValue
// 		);
// 	});
// }
