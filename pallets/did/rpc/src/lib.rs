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

//! RPC interface for the transaction payment pallet.
// #![cfg_attr(not(feature = "std"), no_std)]
use std::sync::Arc;

use codec::Codec;
use jsonrpsee::{
	core::RpcResult,
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
// use sp_rpc::number::NumberOrHex;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use pallet_did::types::Attribute;
pub use pallet_did_rpc_runtime_api::ReadAttributeApi as ReadAttributeRuntimeApi;

use scale_info::TypeInfo;
use sp_runtime::codec::{Decode, Encode};

use node_primitives::{AccountId, BlockNumber, Moment};

/// Attributes or properties that make an DID reply.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, TypeInfo)]
pub struct RpcDidAttribute<BlockNumber, Moment> {
	pub name: Vec<u8>,
	pub value: Vec<u8>,
	pub validity: BlockNumber,
	pub creation: Moment,
}

impl From<Attribute<BlockNumber, Moment>> for RpcDidAttribute<BlockNumber, Moment> {
	fn from(attribute: Attribute<BlockNumber, Moment>) -> Self {
		Self {
			name: attribute.name.into(),
			value: attribute.value.into(),
			validity: attribute.validity,
			creation: attribute.creation,
		}
	}
}

#[rpc(client, server)]
// pub trait TransactionPaymentApi<BlockHash, ResponseType> {
pub trait ReadAttributeApi<BlockHash, AccountId, Blocknumber, Moment> {
	#[method(name = "read_did_attribute")]
	fn get_did_attributes(
		&self,
		did: AccountId,
		name: Bytes,
		at: Option<BlockHash>,
	) -> RpcResult<Option<Attribute<BlockNumber, Moment>>>;
}

/// Provides RPC methods to query a dispatchable's class, weight and fee.
pub struct Did<C, P> {
	/// Shared reference to the client.
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>,
}

impl<C, P> Did<C, P> {
	/// Creates a new instance of the DID Rpc helper.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

impl<C, Block> ReadAttributeApiServer<<Block as BlockT>::Hash, AccountId, BlockNumber, Moment>
	for Did<C, Block>
where
	Block: BlockT,
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
	C::Api: ReadAttributeRuntimeApi<Block, AccountId, BlockNumber, Moment>,
	AccountId: Codec,
	BlockNumber: Codec,
	Moment: Codec,
{
	fn get_did_attributes(
		&self,
		did: AccountId,
		name: Bytes,
		at: Option<Block::Hash>,
	) -> RpcResult<Option<Attribute<BlockNumber, Moment>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.read(&at, did, name.to_vec()).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to query DID info.",
				Some(e.to_string()),
			))
			.into()
		})
	}
}
