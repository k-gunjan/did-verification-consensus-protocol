// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Runtime API definition for DID pallet.

#![cfg_attr(not(feature = "std"), no_std)]

// use sp_std::vec::Vec;
use frame_support::inherent::Vec;
use sp_runtime::codec::Codec;
// add Attribute from did pallet
use pallet_did::types::Attribute;

// The `RuntimeApi` trait is used to define the runtime api.
// add runtimeapi for get_attribute of did pallet
sp_api::decl_runtime_apis! {
	pub trait ReadAttributeApi<AccountId, Blocknumber, Moment> where
		AccountId: Codec,
		Blocknumber: Codec,
		Moment: Codec,
	{
		fn read_attribute(did: AccountId, name: Vec<u8>) -> Option<Attribute<Blocknumber, Moment>>;
		fn get_a_fixed_value(i:u32, j:u32) -> u32;
	}
}
