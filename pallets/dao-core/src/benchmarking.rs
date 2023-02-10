//
// Build Instructions:
// > cargo build --release --features runtime-benchmarks --features local-node
//
// Weight Creation:
// > ./target/release/genesis-dao benchmark pallet --chain dev --pallet pallet_dao_core --extrinsic '*' --steps 20 --repeat 10 --output pallets/dao-core/src/weights.rs --template ./benchmarking/frame-weight-template.hbs
//
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

use crate::Pallet as DaoCore;

/// Creates a DAO for the given caller
/// - `caller`: AccountId of the dao creator
fn setup_dao<T: Config>(caller: T::AccountId) -> Vec<u8>{
	let dao_id: Vec<u8> = b"GDAO".to_vec();

	DaoCore::<T>::create_dao(
		RawOrigin::Signed(caller).into(),
		dao_id.clone(),
		b"Genesis DAO".to_vec()
	).expect("error on dao creation");
	dao_id
}

/// Setups a whitelisted caller to interact with the pallet,
/// we'll inject 1_000_000_000_000_000_000x the min balance into it - 1 full unit
fn setup_caller<T: Config>() -> T::AccountId {
	let caller: T::AccountId = whitelisted_caller();
	let units: u32 = 1_000_000_000u32;
	<T as Config>::Currency::issue(<T as Config>::Currency::minimum_balance() * units.into() * 1_000_000u32.into());
	<T as Config>::Currency::make_free_balance_be(&caller, <T as Config>::Currency::minimum_balance() * units.into() * 1_000_000u32.into());
	caller
}

/// Helper func to validate the benchmark flow by last event
/// - `generic_event`: Any runtime event that we want to equal to the last event emitted
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
  	create_dao {
		let caller = setup_caller::<T>();
		let dao_id: Vec<u8> = b"GDAO".to_vec();
	}: _(RawOrigin::Signed(caller.clone()), dao_id.clone(), b"Genesis DAO".to_vec())
  	verify {
		let bounded_dao_id: BoundedVec<_, _> = dao_id.clone().try_into().expect("unbounded");
		assert_last_event::<T>(Event::DaoCreated {
			owner: caller.clone(),  dao_id: bounded_dao_id.clone()
		}.into());
	}

	destroy_dao {
		let caller = setup_caller::<T>();
		let dao_id = setup_dao::<T>(caller.clone());
	}: _(RawOrigin::Signed(caller.clone()), dao_id.clone())
  	verify {
		let bounded_dao_id: BoundedVec<_, _> = dao_id.clone().try_into().expect("unbounded");
		assert_last_event::<T>(Event::DaoDestroyed { dao_id: bounded_dao_id.clone() }.into());
	}

	issue_token {
		let caller = setup_caller::<T>();
		let dao_id = setup_dao::<T>(caller.clone());
		let supply:  T::Balance = 1000u32.into();
	}: _(RawOrigin::Signed(caller.clone()), dao_id.clone(), supply)
  	verify {
		let bounded_dao_id: BoundedVec<_, _> = dao_id.clone().try_into().expect("unbounded");
		assert_last_event::<T>(Event::DaoTokenIssued {
			dao_id: bounded_dao_id.clone(),
			supply: supply,
			asset_id: DaoCore::<T>::load_dao(dao_id.clone()).unwrap().asset_id.unwrap()
		}.into());
	}

	set_metadata {
		let caller = setup_caller::<T>();
		let dao_id = setup_dao::<T>(caller.clone());
		let metadata = b"http://my.cool.dao".to_vec();
		let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
	}: _(RawOrigin::Signed(caller.clone()), dao_id.clone(), metadata.clone(), hash.clone())
	verify {
		let bounded_dao_id: BoundedVec<_, _> = dao_id.clone().try_into().expect("unbounded");
		assert_last_event::<T>(Event::DaoMetadataSet {
			dao_id: bounded_dao_id.clone()
		}.into());
	}

	impl_benchmark_test_suite!(DaoCore, crate::mock::new_test_ext(), crate::mock::Test)
}
