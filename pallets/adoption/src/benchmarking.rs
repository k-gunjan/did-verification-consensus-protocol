//! Benchmarking setup for pallet-adoption

use super::*;

#[allow(unused)]
use crate::Pallet as AdoptionEvent;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;

/// Assert that the last event equals the provided one.
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const CALLER_ACCOUNT: &str = "Alice";
const PARTNER_ID: &[u8; 10] = b"pid7hu4210";
const PARTNER_INFO: &[u8] = b"bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
const REFERRER_PARTNER_ID: &[u8; 10] = b"rid7hu7h4k";
const EVENT_TYPE_ID: &[u8; 10] = b"etid7hu110";
const EVENT_DETAILS: &[u8] = b"bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
const EVENT_ID: &[u8; 10] = b"eid7hu7110";
const EVENT_CREATION_DETAILS: &[u8] =
	b"bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
const EVENT_STATE: u32 = 1;
const EVENT_VALUE: u128 = 1_000_000_000_000;

benchmarks! {
	partner_registration {
		let caller : T::AccountId = account(CALLER_ACCOUNT, 0, 0);

	}: _(RawOrigin::Signed(caller.clone()), PARTNER_ID.to_vec(), PARTNER_INFO.to_vec())
	verify {
		assert_last_event::<T>(Event::<T>::NewPartnerAdded(
			PARTNER_ID.to_vec(),
			PARTNER_INFO.to_vec(),
		).into());
	}

	adoption_event_types_creation {
		let caller : T::AccountId = account(CALLER_ACCOUNT, 0, 0);

	}: _(RawOrigin::Signed(caller.clone()), EVENT_TYPE_ID.to_vec(), EVENT_DETAILS.to_vec())
	verify {
		assert_last_event::<T>(Event::<T>::NewEventTypeAdded(
			EVENT_TYPE_ID.to_vec(),
			EVENT_DETAILS.to_vec()).into());
	}

	adoption_event_creation {
		let caller : T::AccountId = account(CALLER_ACCOUNT, 0, 0);

		<AdoptionEvent<T>>::partner_registration
		(
			RawOrigin::Signed(caller.clone()).into(),
			PARTNER_ID.to_vec(),
			PARTNER_INFO.to_vec()
		)?;

		<AdoptionEvent<T>>::partner_registration
		(
			RawOrigin::Signed(caller.clone()).into(),
			REFERRER_PARTNER_ID.to_vec(),
			PARTNER_INFO.to_vec()
		)?;

		<AdoptionEvent<T>>::adoption_event_types_creation
		(
			RawOrigin::Signed(caller.clone()).into(),
			EVENT_TYPE_ID.to_vec(),
			EVENT_DETAILS.to_vec()
		)?;


	}: _(RawOrigin::Signed(caller.clone()),
		EVENT_ID.to_vec(),
		PARTNER_ID.to_vec(),
		EVENT_TYPE_ID.to_vec(),
		EVENT_VALUE,
		caller.clone(),
		REFERRER_PARTNER_ID.to_vec(),
		EVENT_CREATION_DETAILS.to_vec()
	)
	verify {
		assert_last_event::<T>(Event::<T>::NewEventAdded(
			EVENT_ID.to_vec()).into());
	}

	adoption_event_participant_addition {
		let caller : T::AccountId = account(CALLER_ACCOUNT, 0, 0);
		let participant : T::AccountId = account("Bob", 0, 0);

		<AdoptionEvent<T>>::partner_registration
		(
			RawOrigin::Signed(caller.clone()).into(),
			PARTNER_ID.to_vec(),
			PARTNER_INFO.to_vec()
		)?;

		<AdoptionEvent<T>>::partner_registration
		(
			RawOrigin::Signed(caller.clone()).into(),
			REFERRER_PARTNER_ID.to_vec(),
			PARTNER_INFO.to_vec()
		)?;

		<AdoptionEvent<T>>::adoption_event_types_creation
		(
			RawOrigin::Signed(caller.clone()).into(),
			EVENT_TYPE_ID.to_vec(),
			EVENT_DETAILS.to_vec()
		)?;

		<AdoptionEvent<T>>::adoption_event_creation
		(
			RawOrigin::Signed(caller.clone()).into(),
			EVENT_ID.to_vec(),
			PARTNER_ID.to_vec(),
			EVENT_TYPE_ID.to_vec(),
			EVENT_VALUE,
			caller.clone(),
			REFERRER_PARTNER_ID.to_vec(),
			EVENT_CREATION_DETAILS.to_vec()
		)?;

		<AdoptionEvent<T>>::event_set_state
		(
			RawOrigin::Signed(caller.clone()).into(),
			EVENT_ID.to_vec(),
			EVENT_STATE
		)?;

	}: _(RawOrigin::Signed(caller.clone()), participant.clone(), EVENT_ID.to_vec())
	verify {
		assert_last_event::<T>(Event::<T>::NewParticipantAdded(
			EVENT_ID.to_vec(),
			participant.clone()).into());
	}

	mint_event_participants {
		let caller : T::AccountId = account(CALLER_ACCOUNT, 0, 0);
		let participant : T::AccountId = account("Bob", 0, 0);

		<AdoptionEvent<T>>::partner_registration
		(
			RawOrigin::Signed(caller.clone()).into(),
			PARTNER_ID.to_vec(),
			PARTNER_INFO.to_vec()
		)?;

		<AdoptionEvent<T>>::partner_registration
		(
			RawOrigin::Signed(caller.clone()).into(),
			REFERRER_PARTNER_ID.to_vec(),
			PARTNER_INFO.to_vec()
		)?;

		<AdoptionEvent<T>>::adoption_event_types_creation
		(
			RawOrigin::Signed(caller.clone()).into(),
			EVENT_TYPE_ID.to_vec(),
			EVENT_DETAILS.to_vec()
		)?;

		<AdoptionEvent<T>>::adoption_event_creation
		(
			RawOrigin::Signed(caller.clone()).into(),
			EVENT_ID.to_vec(),
			PARTNER_ID.to_vec(),
			EVENT_TYPE_ID.to_vec(),
			EVENT_VALUE,
			caller.clone(),
			REFERRER_PARTNER_ID.to_vec(),
			EVENT_CREATION_DETAILS.to_vec()
		)?;

		<AdoptionEvent<T>>::event_set_state
		(
			RawOrigin::Signed(caller.clone()).into(),
			EVENT_ID.to_vec(),
			EVENT_STATE
		)?;

		<AdoptionEvent<T>>::adoption_event_participant_addition
		(
			RawOrigin::Signed(caller.clone()).into(),
			participant.clone(),
			EVENT_ID.to_vec()
		)?;

	}: _(RawOrigin::Signed(caller.clone()), EVENT_ID.to_vec())
	verify {
		assert_last_event::<T>(Event::<T>::MintedToAccountID(
			participant.clone(),
			EVENT_VALUE).into());
	}

	event_set_state {
		let caller : T::AccountId = account(CALLER_ACCOUNT, 0, 0);

		<AdoptionEvent<T>>::partner_registration
		(
			RawOrigin::Signed(caller.clone()).into(),
			PARTNER_ID.to_vec(),
			PARTNER_INFO.to_vec()
		)?;

		<AdoptionEvent<T>>::partner_registration
		(
			RawOrigin::Signed(caller.clone()).into(),
			REFERRER_PARTNER_ID.to_vec(),
			PARTNER_INFO.to_vec()
		)?;

		<AdoptionEvent<T>>::adoption_event_types_creation
		(
			RawOrigin::Signed(caller.clone()).into(),
			EVENT_TYPE_ID.to_vec(),
			EVENT_DETAILS.to_vec()
		)?;

		<AdoptionEvent<T>>::adoption_event_creation
		(
			RawOrigin::Signed(caller.clone()).into(),
			EVENT_ID.to_vec(),
			PARTNER_ID.to_vec(),
			EVENT_TYPE_ID.to_vec(),
			EVENT_VALUE,
			caller.clone(),
			REFERRER_PARTNER_ID.to_vec(),
			EVENT_CREATION_DETAILS.to_vec()
		)?;

		<AdoptionEvent<T>>::event_set_state
		(
			RawOrigin::Signed(caller.clone()).into(),
			EVENT_ID.to_vec(),
			EVENT_STATE
		)?;

	}: _(RawOrigin::Signed(caller.clone()), EVENT_ID.to_vec(), EVENT_STATE)
	verify {
		assert_last_event::<T>(Event::<T>::EventStateUpdated(
			EVENT_STATE).into());
	}

}

#[cfg(test)]
mod tests {
	use crate::mock;
	use frame_support::sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		mock::new_test_ext()
	}
}

impl_benchmark_test_suite!(
	AdoptionEvent,
	crate::benchmarking::tests::new_test_ext(),
	crate::mock::Test,
);
