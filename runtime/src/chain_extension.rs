use crate::{
	sp_api_hidden_includes_construct_runtime::hidden_include::traits::Get, DidModule, Encode,
	Runtime,
};
use frame_support::{
	inherent::Vec,
	log::{error, trace},
	weights::{constants::RocksDbWeight, Weight},
};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use pallet_did::did::Did;

use sp_core::crypto::UncheckedFrom;
use sp_runtime::{traits::StaticLookup, AccountId32, DispatchError};
#[derive(Default)]
pub struct DidChainExtension;

impl ChainExtension<Runtime> for DidChainExtension
// where
// T: SysConfig + pallet_contracts::Config + pallet_did::Config, /* add the pallets needed to
//                                                                * interact with the contract */
// <T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
{
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		// E: Ext<T = T>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		let func_id = env.func_id();
		let mut env = env.buf_in_buf_out();

		// Match on function id assigned in the contract
		match func_id {
			1 => {
				trace!("contract called pallet. id {:?})", func_id);
				let name: Vec<u8> = "VERIFIED".as_bytes().to_vec();
				let did: AccountId32 = env.read_as()?;
				let validity = match DidModule::read_attribute(&did, &name) {
					Some(a) => a.validity,
					_ => 0,
				};
				let result = validity.encode();
				trace!("chain_extension return value: {:?})", &result);
				env.write(&result, false, None)
					.map_err(|_| "Encountered an error when retrieving runtime storage value.")?;
			},
			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"))
			},
		}
		// No error, return status code `0`, indicating `Ok(())`
		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
