// This file is part of Substrate.

// Copyright UNIVERSALDOT FOUNDATION
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

//! Benchmarking setup for pallet-task

use super::*;

#[allow(unused)]
use crate::Pallet as PalletTask;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller, vec, Vec};
use frame_system::RawOrigin;
use frame_support::traits::{Currency};
use pallet_profile::Pallet as PalletProfile;

const SEED: u32 = 0;

// Helper function to assert event thrown during verification
fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

// This creates and returns a `Task` object.
fn create_task_info<T: Config>(_num_fields: u32) -> Task<T> {

	// Populate with worst case scenario
	let mut data = Vec::new();
	data.push(u8::MAX);

	// Populate data fields
	let initiator: T::AccountId = whitelisted_caller();
	let volunteer: T::AccountId = whitelisted_caller();
	let owner: T::AccountId = whitelisted_caller();
	let balance = <T as pallet::Config>::Currency::total_balance(&initiator);
	let deadline = u64::MAX;
	let status: TaskStatus = TaskStatus::InProgress;

	// Create object
	let info = Task {
		title: data.clone(),
		specification: data.clone(),
		initiator: initiator,
		volunteer: volunteer,
		current_owner: owner,
		status: status,
		budget: balance,
		deadline: deadline,
	};

	return info
}

// Helper function to create a profile
fn create_profile<T: Config>(){

	let username = Vec::new();
	let interests = Vec::new();

	let caller: T::AccountId = whitelisted_caller();
	let _profile = PalletProfile::<T>::create_profile(RawOrigin::Signed(caller).into(), username, interests);

}


benchmarks! {
	create_task {
		/* setup initial state */
		let caller: T::AccountId = whitelisted_caller();

		// Populate data fields
		let s in 1 .. u8::MAX.into(); // max bytes for specification
		let x in 1 .. 2000;
		let title = vec![0u8, s as u8];
		let specification = vec![0u8, s as u8];
		let budget = <T as pallet::Config>::Currency::total_balance(&caller);

		// Create profile before creating a task
		create_profile::<T>();
		create_task_info::<T>(1);

	}:
	/* the code to be benchmarked */
	create_task(RawOrigin::Signed(caller.clone()), title, specification, budget, x.into())

	verify {
		/* verifying final state */
		let caller: T::AccountId = whitelisted_caller();
		let hash = PalletTask::<T>::tasks_owned(&caller)[0];

		assert_last_event::<T>(Event::<T>::TaskCreated(caller, hash).into());
	}

	update_task {
		/* setup initial state */
		let caller: T::AccountId = whitelisted_caller();

		// Populate data fields
		let s in 1 .. u8::MAX.into(); // max bytes for specification
		let x in 1 .. 2000;
		let title = vec![0u8, s as u8];
		let specification = vec![0u8, s as u8];
		let budget = <T as pallet::Config>::Currency::total_balance(&caller);

		// Create profile before creating a task
		create_profile::<T>();
		create_task_info::<T>(1);
		let _ = PalletTask::<T>::create_task(RawOrigin::Signed(caller.clone()).into(), title.clone(), specification.clone(), budget, x.into());
		let hash_task = PalletTask::<T>::tasks_owned(&caller)[0];

	}:
	/* the code to be benchmarked */
	update_task(RawOrigin::Signed(caller.clone()), hash_task, title, specification, budget, x.into())

	verify {
		/* verifying final state */
		let caller: T::AccountId = whitelisted_caller();
		let hash = PalletTask::<T>::tasks_owned(&caller)[0];

		assert_last_event::<T>(Event::<T>::TaskUpdated(caller, hash).into());
	}

	start_task {
		/* setup initial state */
		let task_creator: T::AccountId = whitelisted_caller();
		let volunteer: T::AccountId = account("volunteer", 0, SEED);

		// Populate data fields
		let s in 1 .. u8::MAX.into(); // max bytes for specification
		let x in 1 .. 2000;
		let title = vec![0u8, s as u8];
		let specification = vec![0u8, s as u8];
		let budget = <T as pallet::Config>::Currency::total_balance(&task_creator);

		// Create profile before creating a task
		create_profile::<T>();
		let _ = PalletTask::<T>::create_task(RawOrigin::Signed(task_creator.clone()).into(), title, specification, budget, x.into());
		let hash_task = PalletTask::<T>::tasks_owned(&task_creator)[0];

	}: start_task(RawOrigin::Signed(volunteer.clone()), hash_task)
		/* the code to be benchmarked */

	verify {
		/* verifying final state */
		assert_last_event::<T>(Event::<T>::TaskAssigned(volunteer, hash_task).into());
	}

	remove_task {
		/* setup initial state */
		let task_creator: T::AccountId = whitelisted_caller();
		let volunteer: T::AccountId = account("volunteer", 0, SEED);

		// Populate data fields
		let s in 1 .. u8::MAX.into(); // max bytes for specification
		let x in 1 .. 2000;
		let title = vec![0u8, s as u8];
		let specification = vec![0u8, s as u8];
		let budget = <T as pallet::Config>::Currency::total_balance(&task_creator);

		// Create profile before creating a task
		create_profile::<T>();
		let _ = PalletTask::<T>::create_task(RawOrigin::Signed(task_creator.clone()).into(), title, specification, budget, x.into());
		let hash_task = PalletTask::<T>::tasks_owned(&task_creator)[0];

	}: remove_task(RawOrigin::Signed(task_creator.clone()), hash_task)
		/* the code to be benchmarked */

	verify {
		/* verifying final state */
		assert_last_event::<T>(Event::<T>::TaskRemoved(task_creator, hash_task).into());
	}

	complete_task {
		/* setup initial state */
		let task_creator: T::AccountId = whitelisted_caller();
		let volunteer: T::AccountId = account("volunteer", 0, SEED);

		// Populate data fields
		let s in 1 .. u8::MAX.into(); // max bytes for specification
		let x in 1 .. 2000;
		let title = vec![0u8, s as u8];
		let specification = vec![0u8, s as u8];
		let budget = <T as pallet::Config>::Currency::total_balance(&task_creator);

		// Create profile before creating a task
		create_profile::<T>();
		let _ = PalletTask::<T>::create_task(RawOrigin::Signed(task_creator.clone()).into(), title, specification, budget, x.into());
		let hash_task = PalletTask::<T>::tasks_owned(&task_creator)[0];
		let _ = PalletTask::<T>::start_task(RawOrigin::Signed(volunteer.clone()).into(), hash_task.clone());

	}: complete_task(RawOrigin::Signed(volunteer.clone()), hash_task)
		/* the code to be benchmarked */

	verify {
		/* verifying final state */
		assert_last_event::<T>(Event::<T>::TaskCompleted(volunteer, hash_task).into());
	}

	accept_task {
		/* setup initial state */
		let task_creator: T::AccountId = whitelisted_caller();
		let volunteer: T::AccountId = whitelisted_caller();

		// Populate data fields
		let s in 1 .. u8::MAX.into(); // max bytes for specification
		let x in 1 .. 4000;
		let title = vec![0u8, s as u8];
		let specification = vec![0u8, s as u8];
		let budget = <T as pallet::Config>::Currency::total_balance(&task_creator);

		// Create profile before creating a task
		create_profile::<T>();
		let _ = PalletTask::<T>::create_task(RawOrigin::Signed(task_creator.clone()).into(), title, specification, budget, x.into());
		let hash_task = PalletTask::<T>::tasks_owned(&task_creator)[0];
		let _ = PalletTask::<T>::start_task(RawOrigin::Signed(volunteer.clone()).into(), hash_task.clone());
		let _ = PalletTask::<T>::complete_task(RawOrigin::Signed(volunteer.clone()).into(), hash_task.clone());

	}: accept_task(RawOrigin::Signed(task_creator.clone()), hash_task)
		/* the code to be benchmarked */

	verify {
		/* verifying final state */
		assert_last_event::<T>(Event::<T>::TaskAccepted(task_creator, hash_task).into());
	}

	reject_task {
		/* setup initial state */
		let task_creator: T::AccountId = whitelisted_caller();
		let volunteer: T::AccountId = account("volunteer", 0, SEED);

		// Populate data fields
		let s in 1 .. u8::MAX.into(); // max bytes for specification
		let x in 1 .. 4000;
		let title = vec![0u8, s as u8];
		let specification = vec![0u8, s as u8];
		let budget = <T as pallet::Config>::Currency::total_balance(&task_creator);

		// Create profile before creating a task
		create_profile::<T>();
		let _ = PalletTask::<T>::create_task(RawOrigin::Signed(task_creator.clone()).into(), title, specification, budget, x.into());
		let hash_task = PalletTask::<T>::tasks_owned(&task_creator)[0];
		let _ = PalletTask::<T>::start_task(RawOrigin::Signed(volunteer.clone()).into(), hash_task.clone());
		let _ = PalletTask::<T>::complete_task(RawOrigin::Signed(volunteer.clone()).into(), hash_task.clone());

	}: reject_task(RawOrigin::Signed(task_creator.clone()), hash_task)
		/* the code to be benchmarked */

	verify {
		/* verifying final state */
		assert_last_event::<T>(Event::<T>::TaskRejected(task_creator, hash_task).into());
	}
}

impl_benchmark_test_suite!(PalletTask, crate::mock::new_test_ext(), crate::mock::Test,);
