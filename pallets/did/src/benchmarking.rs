//! Benchmarking setup for did

use super::*;
#[allow(unused)]
use crate::Pallet as DID;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;

/// Assert that the last event equals the provided one.
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const IDENTITY_STR: &str = "Alice";
const CALLER_ACCOUNT_STR: &str = "Bob";
const NAME_BYTES: &[u8; 2] = b"id";
const ATTRITUBE_BYTES: &[u8; 17] = b"did:fn:1234567890";

benchmarks! {
	add_attribute {
		let identity : T::AccountId = account(IDENTITY_STR, 0, 0);

	}: _(RawOrigin::Signed(identity.clone()), identity.clone(), NAME_BYTES.to_vec(), ATTRITUBE_BYTES.to_vec(), None)
	verify {
		assert_last_event::<T>(Event::<T>::AttributeAdded(
			identity,
			NAME_BYTES.to_vec(),
			ATTRITUBE_BYTES.to_vec(),
			None,
		).into());
	}

	get_attribute {
		let caller : T::AccountId = account(CALLER_ACCOUNT_STR, 0, 0);
		let identity : T::AccountId = account(IDENTITY_STR, 0, 0);

		<DID<T>>::add_attribute(
			RawOrigin::Signed(identity.clone()).into(),
			identity.clone(),
			NAME_BYTES.to_vec(),
			ATTRITUBE_BYTES.to_vec(),
			None)?;
	}: _(RawOrigin::Signed(caller.clone()), identity.clone(), NAME_BYTES.to_vec())
	verify {
		assert_last_event::<T>(Event::<T>::AttributeFetched(
			caller,
			identity,
			NAME_BYTES.to_vec()).into());
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

impl_benchmark_test_suite!(DID, crate::benchmarking::tests::new_test_ext(), crate::mock::Test,);
