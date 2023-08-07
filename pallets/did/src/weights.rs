
//! Autogenerated weights for `pallet_did`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-07-12, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `kali`, CPU: `Intel(R) Core(TM) i7-10870H CPU @ 2.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/felidae-node
// benchmark
// pallet
// --chain
// dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// pallet_did
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --output
// pallets/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_transaction.
pub trait WeightInfo {
	fn add_attribute() -> Weight;
	fn get_attribute() -> Weight;
}

/// Weight functions for `pallet_did`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: unknown `0xc6ac8c4e605bee5cbf6880ef91a4a9393c49e1811cda81f0f584a46ebd8ba262` (r:1 w:0)
	/// Proof Skipped: unknown `0xc6ac8c4e605bee5cbf6880ef91a4a9393c49e1811cda81f0f584a46ebd8ba262` (r:1 w:0)
	/// Storage: unknown `0xc6ac8c4e605bee5cbf6880ef91a4a93965d26e0531d8633d3382c8703ca0cf39` (r:1 w:1)
	/// Proof Skipped: unknown `0xc6ac8c4e605bee5cbf6880ef91a4a93965d26e0531d8633d3382c8703ca0cf39` (r:1 w:1)
	/// Storage: unknown `0xc6ac8c4e605bee5cbf6880ef91a4a93924433d9e568e64cc1fcf83850a536c97` (r:1 w:1)
	/// Proof Skipped: unknown `0xc6ac8c4e605bee5cbf6880ef91a4a93924433d9e568e64cc1fcf83850a536c97` (r:1 w:1)
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: unknown `0xc6ac8c4e605bee5cbf6880ef91a4a93916cad8927bcd207b526f16775c96863b` (r:0 w:1)
	/// Proof Skipped: unknown `0xc6ac8c4e605bee5cbf6880ef91a4a93916cad8927bcd207b526f16775c96863b` (r:0 w:1)
	fn add_attribute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `45`
		//  Estimated: `8108`
		// Minimum execution time: 39_567 nanoseconds.
		Weight::from_ref_time(39_935_000)
			.saturating_add(Weight::from_proof_size(8108))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: unknown `0xc6ac8c4e605bee5cbf6880ef91a4a93965d26e0531d8633d3382c8703ca0cf39` (r:1 w:0)
	/// Proof Skipped: unknown `0xc6ac8c4e605bee5cbf6880ef91a4a93965d26e0531d8633d3382c8703ca0cf39` (r:1 w:0)
	/// Storage: unknown `0xc6ac8c4e605bee5cbf6880ef91a4a93924433d9e568e64cc1fcf83850a536c97` (r:1 w:0)
	/// Proof Skipped: unknown `0xc6ac8c4e605bee5cbf6880ef91a4a93924433d9e568e64cc1fcf83850a536c97` (r:1 w:0)
	fn get_attribute() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `298`
		//  Estimated: `5546`
		// Minimum execution time: 28_622 nanoseconds.
		Weight::from_ref_time(29_778_000)
			.saturating_add(Weight::from_proof_size(5546))
			.saturating_add(T::DbWeight::get().reads(2))
	}
}