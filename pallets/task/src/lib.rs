// This file is part of Substrate.

// Copyright (C) 2022 UNIVERSALDOT FOUNDATION.
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


//! # Task Pallet
//! 
//! ## Version: 0.7.0
//!
//! - [`Config`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! The Task Pallet creates a way for users to interact with one another.
//!
//! There are two types of Users who can interact with tasks. We call them
//! Initiators and Volunteers.
//!
//! Initiators are people who have the permission to Create and Accept Tasks.
//! Volunteers are people who have the permission to Start and Complete Tasks.
//!
//! Anybody can become an Initiator or Volunteer. In other words,
//! one doesn't need permission to become an Initiator or Volunteer.
//! 
//! Budget funds reserved using ReservableCurrency.
//! Funds are unreserved and sent to volunteer when a task is completed or removed.
//!
//! Tasks with expired deadline are automatically removed from storage.
//!
//! ## Interface
//!
//! ### Public Functions
//!
//! - `create_task` - Function used to create a new task.
//! 	Inputs:
//! 		- title: BoundedVec,
//! 		- specification: BoundedVec,
//! 		- budget: BalanceOf<T>,
//! 		- deadline: u64
//! 		- attachments: BoundedVec,
//! 		- keywords: BoundedVec
//! 		- organization: Option<OrganizationIdOf<T>>
//!
//! - `update_task` - Function used to update already existing task.
//! 	Inputs:
//! 		- task_id: T::Hash,
//! 		- title: Vec<u8>,
//! 		- specification: Vec<u8>,
//! 		- budget: BalanceOf<T>,
//! 		- deadline: u64,
//! 		- attachments, BoundedVec
//! 		- keywords: BoundedVec,
//! 		- organization: Option<OrganizationIdOf<T>>
//! 	Only the creator of the task has the update rights.
//!
//! - `remove_task` - Function used to remove an already existing task.
//! 	Inputs:
//! 		- task_id: T::Hash,
//!
//! - `start_task` - Function used to start already existing task.
//! 	Inputs:
//! 		- task_id: T::Hash,
//!
//! - `complete_task` - Function used to complete a task.
//! 	Inputs:
//! 		- task_id: T::Hash,
//!
//! - `accept_task` - Function used to accept completed task.
//! 	Inputs:
//! 		- task_id: T::Hash,
//! 	After the task is accepted, its data is removed from storage.
//!
//! - `reject_task` - Function used to reject an already completed task.
//! 	Inputs:
//! 	- task_id: T::Hash,
//! 	- feedback : BoundedVec
//! 
//! Storage Items:
//! 	Tasks: Stores Task related information
//! 	TaskCount: Counts the total number of Tasks in the ecosystem
//! 	TasksOwned: Keeps track of how many tasks are owned per account
//! 
//!
//! ## Related Modules
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub mod traits;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime, PalletId};
	use frame_system::pallet_prelude::*;
	use frame_support::{
		sp_runtime::traits::{Hash, SaturatedConversion, AccountIdConversion},
		traits::{Currency, ReservableCurrency, tokens::ExistenceRequirement},
		transactional};
	use scale_info::TypeInfo;
	use sp_std::vec::Vec;
	use core::time::Duration;
	use crate::{
		weights::WeightInfo,
		TaskStatus::Created,
		traits::Organization,
		traits
	};

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	// Use AccountId from frame_system
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type OrganizationIdOf<T> = <T as frame_system::Config>::Hash;

	// Struct for holding Task information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Task<T: Config> {
		pub title: BoundedVec<u8, T::MaxTitleLen>,
		pub specification: BoundedVec<u8, T::MaxSpecificationLen>,
		pub initiator: AccountOf<T>,
		pub volunteer: AccountOf<T>,
		pub current_owner: AccountOf<T>,
		pub status: TaskStatus,
		pub budget: BalanceOf<T>,
		pub deadline: u64,
		pub attachments: BoundedVec<u8, T::MaxAttachmentsLen>,
		pub keywords: BoundedVec<u8, T::MaxKeywordsLen>,
		pub feedback: Option<BoundedVec<u8, T::MaxFeedbackLen>>,
		pub created_at: <T as frame_system::Config>::BlockNumber,
		pub updated_at:<T as frame_system::Config>::BlockNumber,
		pub completed_at: <T as frame_system::Config>::BlockNumber,
		/// The organization to which the task belongs.
		pub organization: Option<OrganizationIdOf<T>>
	}

	// Set TaskStatus enum.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
  	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
  	pub enum TaskStatus {
    	Created,
    	InProgress,
		Completed,
		Accepted,
  	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_profile::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Currency type that is linked with AccountID
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Organization type used to verify organization existence
		type Organization: traits::Organization<Self::Hash>;

		/// Time provider type
		type Time: UnixTime;

		/// The maximum amount of tasks a single account can own.
		#[pallet::constant]
		type MaxTasksOwned: Get<u32>;

		#[pallet::constant]
		type MaxTitleLen: Get<u32> + MaxEncodedLen + TypeInfo;

		#[pallet::constant]
		type MaxSpecificationLen: Get<u32> + MaxEncodedLen + TypeInfo;

		#[pallet::constant]
		type MaxAttachmentsLen: Get<u32> + MaxEncodedLen + TypeInfo;

		#[pallet::constant]
		type MaxFeedbackLen: Get<u32> + MaxEncodedLen + TypeInfo;

		#[pallet::constant]
		type MaxKeywordsLen: Get<u32> + MaxEncodedLen + TypeInfo;

		/// WeightInfo provider.
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn task_count)]
	/// TaskCount: Get total number of Tasks in the system
	pub(super) type TaskCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tasks)]
	/// Tasks: Store Tasks in a  Storage Map where [key: hash, value: Task]
	pub(super) type Tasks<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Task<T>>;

	#[pallet::storage]
	#[pallet::getter(fn tasks_owned)]
	/// Keeps track of which Accounts own which Tasks.
	pub(super) type TasksOwned<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<T::Hash, T::MaxTasksOwned>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event for creation of task [AccountID, hash id]
		TaskCreated(T::AccountId, T::Hash),

		/// Event for updating existing task [AccountID, hash id]
		TaskUpdated(T::AccountId, T::Hash),

		/// Task assigned to new account [AccountID, hash id]
		TaskAssigned(T::AccountId, T::Hash),

		/// Task completed by assigned account [AccountID, hash id]
		TaskCompleted(T::AccountId, T::Hash),

		/// Task accepted by owner [AccountID, hash id]
		TaskAccepted(T::AccountId, T::Hash),

		/// Task rejected by owner [AccountID, hash id]
		TaskRejected(T::AccountId, T::Hash),

		/// Task deleted by owner [AccountID, hash id]
		TaskRemoved(T::AccountId, T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Reached maximum number of tasks.
		TaskCountOverflow,
		/// The given task doesn't exists. Try again
		TaskNotExist,
		/// Only the initiator of task has the rights to remove task
		OnlyInitiatorAcceptsTask,
		/// Not enough balance to pay
		NotEnoughBalance,
		/// Exceed maximum tasks owned
		ExceedMaxTasksOwned,
		/// You are not allowed to complete this task
		NoPermissionToComplete,
		/// You are not allowed to update this task. Task is already in progress.
		NoPermissionToUpdate,
		/// You don't have permission to start a task that you have created.
		NoPermissionToStart,
		/// You don't have permission to delete this task.
		NoPermissionToRemove,
		/// Only completed tasks can be rejected.
		OnlyCompletedTaskAreRejected,
		/// This account has no Profile yet.
		NoProfile,
		/// Provided deadline value can not be accepted.
		IncorrectDeadlineTimestamp,
		/// Only Task creator can update the task.
		OnlyInitiatorUpdatesTask,
		/// The provided organization identifier does not exist.
		InvalidOrganization
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Function call that creates tasks.  [origin, title, specification, budget, deadline, attachments, keywords, organization]
		#[pallet::weight(<T as Config>::WeightInfo::create_task(0,0))]
		pub fn create_task(origin: OriginFor<T>, title: BoundedVec<u8, T::MaxTitleLen>, specification: BoundedVec<u8, T::MaxSpecificationLen>, budget: BalanceOf<T>,
			deadline: u64, attachments: BoundedVec<u8, T::MaxAttachmentsLen>, keywords: BoundedVec<u8, T::MaxKeywordsLen>, organization: Option<OrganizationIdOf<T>>) -> DispatchResultWithPostInfo {

			// Check that the extrinsic was signed and get the signer.
			let signer = ensure_signed(origin)?;

			// Verify the organization (if provided)
			if let Some(organization) = organization {
				ensure!(T::Organization::exists(&organization), Error::<T>::InvalidOrganization);
			}

			ensure!(<T as self::Config>::Currency::can_reserve(&signer, budget), Error::<T>::NotEnoughBalance);
			
			// Update storage.
<<<<<<< HEAD
			let task_id = Self::new_task(&signer, title, specification, &budget, deadline, attachments, keywords, organization)?;

=======
			let task_id = Self::new_task(&signer, title, specification, &budget, deadline, attachments, keywords)?;
>>>>>>> f7f284c (fixed another check before writing failed test in tasks)
			// Reserve currency of the task creator.
			<T as self::Config>::Currency::reserve(&signer, budget.into()).expect("can_reserve has been called; qed");

			// Emit a Task Created Event.
			Self::deposit_event(Event::TaskCreated(signer, task_id));

			Ok(().into())
		}

		/// Function call that updates a created task.  [origin, task, title, specification, budget, deadline, attachments, keywords, organization]
		//	todo: minimum change amount?
		#[pallet::weight(<T as Config>::WeightInfo::update_task(0,0))]
		pub fn update_task(origin: OriginFor<T>, task_id: T::Hash, title: BoundedVec<u8, T::MaxTitleLen>, specification: BoundedVec<u8, T::MaxSpecificationLen>,
			budget: BalanceOf<T>, deadline: u64, attachments: BoundedVec<u8, T::MaxAttachmentsLen>, keywords: BoundedVec<u8, T::MaxKeywordsLen>, organization: Option<OrganizationIdOf<T>>) -> DispatchResultWithPostInfo {

			// Check that the extrinsic was signed and get the signer.
			let signer = ensure_signed(origin)?;

			// Verify the organization (if provided)
			if let Some(organization) = organization {
				ensure!(T::Organization::exists(&organization), Error::<T>::InvalidOrganization);
			}

			// Check if task exists
			let old_task = Self::tasks(&task_id).ok_or(<Error<T>>::TaskNotExist)?;

			// Check if the owner is the one who created task
			ensure!(Self::is_task_initiator(&task_id, &signer)?, <Error<T>>::OnlyInitiatorUpdatesTask);

			// Ensure user has a profile before creating a task
			ensure!(pallet_profile::Pallet::<T>::has_profile(&signer).unwrap(), <Error<T>>::NoProfile);

			// Check if task is in created status. Tasks can be updated only before work has been started.
			ensure!(TaskStatus::Created == old_task.status, <Error<T>>::NoPermissionToUpdate);

			// Ensure deadline is in the future
			let deadline_duration = Duration::from_millis(old_task.deadline.saturated_into::<u64>());
			ensure!(T::Time::now() < deadline_duration, Error::<T>::IncorrectDeadlineTimestamp);

			if old_task.budget != budget {
				// Check that sender can reserve.
				// Reserve difference if the budget has increased.
				if budget > old_task.budget {
					let diff = budget - old_task.budget;
					ensure!(<T as self::Config>::Currency::can_reserve(&signer, diff), Error::<T>::NotEnoughBalance);
					<T as self::Config>::Currency::reserve(&signer, diff).expect("can_reserve has been called; qed");

				// Unreserve difference if the budget has decreased.
				} else {
					let diff = old_task.budget - budget;
					<T as self::Config>::Currency::unreserve(&signer, diff);
				}
			}

			// Update storage after as we need to check if sender can reserve new amount.
			let _task_id = Self::update_created_task(old_task, &task_id, title, specification, &budget, deadline, attachments, keywords, organization)?;

			// Emit a Task Updated Event.
			Self::deposit_event(Event::TaskUpdated(signer, task_id));

			Ok(().into())
		}

		/// Function that removes a task by task owner. [origin, task_id]
		#[pallet::weight(<T as Config>::WeightInfo::remove_task(0,0))]
		pub fn remove_task(origin: OriginFor<T>, task_id: T::Hash) -> DispatchResult {

			// Check that the extrinsic was signed and get the signer.
			let signer = ensure_signed(origin)?;

			// Delete task from storage.
			Self::delete_task(&signer, &task_id)?;

			// Emit a Task Removed Event.
			Self::deposit_event(Event::TaskRemoved(signer, task_id));

			Ok(())
		}


		/// Function call that starts a task by assigning new task owner. [origin, task_id]
		#[pallet::weight(<T as Config>::WeightInfo::start_task(0,0))]
		pub fn start_task(origin: OriginFor<T>, task_id: T::Hash) -> DispatchResult {

			// Check that the extrinsic was signed and get the signer.
			let signer = ensure_signed(origin)?;

			// Assign task and update storage.
			Self::assign_task(&signer, &task_id)?;

			// Emit a Task Assigned Event.
			Self::deposit_event(Event::TaskAssigned(signer, task_id));

			Ok(())
		}

		/// Function that completes a task [origin, task_id]
		#[pallet::weight(<T as Config>::WeightInfo::complete_task(0,0))]
		pub fn complete_task(origin: OriginFor<T>, task_id: T::Hash) -> DispatchResult {

			// Check that the extrinsic was signed and get the signer.
			let signer = ensure_signed(origin)?;

			// Complete task and update storage.
			Self::mark_finished(&signer, &task_id)?;

			// Emit a Task Completed Event.
			Self::deposit_event(Event::TaskCompleted(signer, task_id));

			Ok(())
		}

		/// Function to accept a completed task. [origin, task_id]
		#[transactional]
		#[pallet::weight(<T as Config>::WeightInfo::accept_task(0,0))]
		pub fn accept_task(origin: OriginFor<T>, task_id: T::Hash) -> DispatchResult {

			// Check that the extrinsic was signed and get the signer.
			let signer = ensure_signed(origin)?;

			// Check if task exists
			let mut task = Self::tasks(&task_id).ok_or(<Error<T>>::TaskNotExist)?;

			// Ensure owner
			ensure!(&task.current_owner == &signer, Error::<T>::OnlyInitiatorAcceptsTask);

			// Transfer reserved funds of task amount to volunteer.
			<T as self::Config>::Currency::unreserve(&signer, task.budget);
			<T as self::Config>::Currency::transfer(&signer, &task.volunteer, task.budget, ExistenceRequirement::AllowDeath)?;

			// Accept task and update storage.
			Self::accept_completed_task(&signer, &mut task, &task_id)?;

			// Add task to completed tasks list of volunteer's profile.
			pallet_profile::Pallet::<T>::add_task_to_completed_tasks(&task.volunteer, task_id)?;

			// Emit a Task Removed Event.
			Self::deposit_event(Event::TaskAccepted(signer, task_id));

			Ok(())
		}

		/// Function to reject a completed task. [origin, task_id]
		#[pallet::weight(<T as Config>::WeightInfo::reject_task(0,0))]
		pub fn reject_task(origin: OriginFor<T>, task_id: T::Hash, feedback: BoundedVec<u8, T::MaxFeedbackLen>) -> DispatchResult {

			// Check that the extrinsic was signed and get the signer.
			let signer = ensure_signed(origin)?;

			// Reject task and update storage.
			Self::reject_completed_task(&signer, &task_id, feedback)?;

			// Emit a Task Rejected Event.
			Self::deposit_event(Event::TaskRejected(signer, task_id));

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T:Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(_n: T::BlockNumber) -> frame_support::weights::Weight {
			// Remove tasks which have not been started, and have passed the deadline
			let mut weight = 0;
			let current_timestamp = T::Time::now();
			let task_hashes : Vec<T::Hash> = Tasks::<T>::iter_keys().collect();
			for th in task_hashes {
				let task = Tasks::<T>::get(th);
				if let Some(task) = task {
					let deadline_duration = Duration::from_millis(task.deadline.saturated_into::<u64>());
					if deadline_duration < current_timestamp {
						if let Ok(()) = Self::delete_task(&task.initiator, &th) {
							weight += 10_000;
						}
					}
				}
			}
			weight
		}
	}

	// *** Helper functions *** //
	impl<T:Config> Pallet<T> {

		fn new_task(from_initiator: &T::AccountId, title: BoundedVec<u8, T::MaxTitleLen>, specification: BoundedVec<u8, T::MaxSpecificationLen>, budget: &BalanceOf<T>,
			 deadline: u64, attachments: BoundedVec<u8, T::MaxAttachmentsLen>, keywords: BoundedVec<u8, T::MaxKeywordsLen>, organization: Option<OrganizationIdOf<T>>) -> Result<T::Hash, DispatchError> {

			// Ensure user has a profile before creating a task
			ensure!(pallet_profile::Pallet::<T>::has_profile(from_initiator).unwrap(), <Error<T>>::NoProfile);
			let deadline_duration = Duration::from_millis(deadline.saturated_into::<u64>());
			ensure!(T::Time::now() < deadline_duration, Error::<T>::IncorrectDeadlineTimestamp);

			// Init Task Object
			let task = Task::<T> {
				title,
				specification,
				initiator: from_initiator.clone(),
				volunteer: from_initiator.clone(),
				status: Created,
				budget: *budget,
				current_owner: from_initiator.clone(),
				deadline,
				attachments,
				keywords,
				feedback: None, // Only used when task is rejected
				organization: organization,
				created_at: <frame_system::Pallet<T>>::block_number(),
				updated_at: Default::default(),
				completed_at: Default::default(),
			};

			// Create hash of task
			let task_id = T::Hashing::hash_of(&task);

			// Performs this operation first because as it may fail
			<TasksOwned<T>>::try_mutate(&from_initiator, |tasks_vec| {
				tasks_vec.try_push(task_id)
			}).map_err(|_| <Error<T>>::ExceedMaxTasksOwned)?;

			// Insert task into Hashmap
			<Tasks<T>>::insert(task_id, task);

			// Increase task count
			let new_count = Self::task_count().checked_add(1).ok_or(<Error<T>>::TaskCountOverflow)?;
			<TaskCount<T>>::put(new_count);

			Ok(task_id)
		}

		// Task can be updated only after it has been created. Task that is already in progress can't be updated.
		//  Private helper function.
		fn update_created_task(old_task:Task<T>, task_id: &T::Hash, new_title: BoundedVec<u8, T::MaxTitleLen>, new_specification: BoundedVec<u8, T::MaxSpecificationLen>, new_budget: &BalanceOf<T>,
			new_deadline: u64, attachments: BoundedVec<u8, T::MaxAttachmentsLen>, keywords: BoundedVec<u8, T::MaxKeywordsLen>, organization: Option<OrganizationIdOf<T>>) -> Result<(), DispatchError> {

			let mut new_task: Task<T> = old_task;
			// Init Task Object
			new_task.title = new_title.clone();
			new_task.specification = new_specification.clone();
			new_task.budget = *new_budget;
			new_task.deadline = new_deadline;
			new_task.attachments = attachments.clone();
			new_task.keywords = keywords.clone();
			new_task.organization = organization;
			new_task.updated_at = <frame_system::Pallet<T>>::block_number();

			// Insert task into Hashmap
			<Tasks<T>>::insert(task_id, new_task);

			Ok(())
		}

		fn assign_task(volunteer: &T::AccountId, task_id: &T::Hash) -> Result<(), DispatchError> {

			// Check if task exists
			let mut task = Self::tasks(&task_id).ok_or(<Error<T>>::TaskNotExist)?;

			// Volunteer must be different than task Initiator
			ensure!(!Self::is_task_initiator(task_id, volunteer)?, <Error<T>>::NoPermissionToStart);

			// Ensure that only Created Task can be started
			ensure!(TaskStatus::Created == task.status, <Error<T>>::NoPermissionToStart);

			// Remove task ownership from previous owner
			let prev_owner = task.current_owner.clone();
			<TasksOwned<T>>::try_mutate(&prev_owner, |owned| {
				if let Some(index) = owned.iter().position(|&id| id == *task_id) {
					owned.swap_remove(index);
					return Ok(());
				}
				Err(())
			}).map_err(|_| <Error<T>>::TaskNotExist)?;

			// Change task properties and insert
			task.current_owner = volunteer.clone();
			task.volunteer = volunteer.clone();
			task.status = TaskStatus::InProgress;
			<Tasks<T>>::insert(task_id, task);

			// Assign task to volunteer
			<TasksOwned<T>>::try_mutate(volunteer, |vec| {
				vec.try_push(*task_id)
			}).map_err(|_| <Error<T>>::ExceedMaxTasksOwned)?;

			Ok(())
		}

		fn mark_finished(to: &T::AccountId, task_id: &T::Hash) -> Result<(), DispatchError> {

			// Check if task exists
			let mut task = Self::tasks(&task_id).ok_or(<Error<T>>::TaskNotExist)?;

			// Check if task is in progress before closing
			ensure!(TaskStatus::InProgress == task.status, <Error<T>>::NoPermissionToComplete);

			// Check if the volunteer is the one who finished task
			ensure!(to == &task.volunteer, <Error<T>>::NoPermissionToComplete);

			// Remove task ownership from current signer
			<TasksOwned<T>>::try_mutate(&to, |owned| {
				if let Some(index) = owned.iter().position(|&id| id == *task_id) {
					owned.swap_remove(index);
					return Ok(());
				}
				Err(())
			}).map_err(|_| <Error<T>>::TaskNotExist)?;

			// Set current owner to initiator
			task.current_owner = task.initiator.clone();
			task.status = TaskStatus::Completed;
			task.completed_at = <frame_system::Pallet<T>>::block_number();
			let task_initiator = task.initiator.clone();

			// Insert into update task
			<Tasks<T>>::insert(task_id, task);

			// Assign task to new owner (original initiator)
			<TasksOwned<T>>::try_mutate(task_initiator, |vec| {
				vec.try_push(*task_id)
			}).map_err(|_| <Error<T>>::ExceedMaxTasksOwned)?;

			Ok(())
		}

		// Internal helper function, checks Must be called before calling this function.
		fn accept_completed_task(task_initiator: &T::AccountId, task: &mut Task<T>, task_id: &T::Hash) -> Result<(), DispatchError> {

			// Remove from ownership
			<TasksOwned<T>>::try_mutate(&task_initiator, |owned| {
				if let Some(index) = owned.iter().position(|&id| id == *task_id) {
					owned.swap_remove(index);
					return Ok(());
				}
				Err(())
			}).map_err(|_| <Error<T>>::TaskNotExist)?;

			// Update task state
			task.status = TaskStatus::Accepted;
			<Tasks<T>>::insert(task_id, task);

			// Reward reputation points to profiles who created/completed a task
			Self::handle_reputation(task_id)?;

			// remove task once accepted
			<Tasks<T>>::remove(task_id);

			// Reduce task count
			let new_count = Self::task_count().saturating_sub(1);
			<TaskCount<T>>::put(new_count);

			Ok(())
		}

		// Task can be rejected by the creator, which places the task back into progress.
		fn reject_completed_task(task_initiator: &T::AccountId, task_id: &T::Hash, feedback: BoundedVec<u8, T::MaxFeedbackLen>) -> Result<(), DispatchError> {

			// Check if task exists
			let mut task = Self::tasks(&task_id).ok_or(<Error<T>>::TaskNotExist)?;

			// Check if the owner is the one who created task
			ensure!(Self::is_task_initiator(task_id, task_initiator)?, <Error<T>>::OnlyInitiatorAcceptsTask);

			// Check if task is Completed before rejecting it
			ensure!(TaskStatus::Completed == task.status, <Error<T>>::OnlyCompletedTaskAreRejected);

			// Remove from ownership of initiator
			<TasksOwned<T>>::try_mutate(&task_initiator, |owned| {
				if let Some(index) = owned.iter().position(|&id| id == *task_id) {
					owned.swap_remove(index);
					return Ok(());
				}
				Err(())
			}).map_err(|_| <Error<T>>::TaskNotExist)?;

			// Set current owner back to volunteer
			task.current_owner = task.volunteer.clone();
			task.status = TaskStatus::InProgress;
			task.feedback = Some(feedback.clone());
			let task_volunteer = task.volunteer.clone();

			// Insert task
			<Tasks<T>>::insert(task_id, task);

			// Assign task to new owner (original volunteer)
			<TasksOwned<T>>::try_mutate(task_volunteer, |vec| {
				vec.try_push(*task_id)
			}).map_err(|_| <Error<T>>::ExceedMaxTasksOwned)?;

			Ok(())
		}

		fn delete_task(task_initiator: &T::AccountId, task_id: &T::Hash) -> Result<(), DispatchError> {

			// Check if task exists
			let task = Self::tasks(&task_id).ok_or(<Error<T>>::TaskNotExist)?;

			// Check if the owner is the one who created task
			ensure!(Self::is_task_initiator(task_id, task_initiator)?, <Error<T>>::NoPermissionToRemove);

			// Ensure that only Created Task can be deleted
			ensure!(TaskStatus::Created == task.status, <Error<T>>::NoPermissionToRemove);

			// remove task from storage
			<Tasks<T>>::remove(task_id);

			// Unreserve balance amount from task creator
			<T as self::Config>::Currency::unreserve(&task_initiator, task.budget);

			// Reduce task count
			let new_count = Self::task_count().saturating_sub(1);
			<TaskCount<T>>::put(new_count);

			Ok(())
		}

		// Function to check if the current signer is the task_initiator
		fn is_task_initiator(task_id: &T::Hash, task_acceptor: &T::AccountId) -> Result<bool, DispatchError> {
			match Self::tasks(task_id) {
				Some(task) => Ok(task.initiator == *task_acceptor),
				None => Err(<Error<T>>::TaskNotExist.into())
			}
		}

		// Function that generates escrow account based on TaskID
		// todo: ensure that usage of into_account_truncating is correct
		// See: https://paritytech.github.io/substrate/master/sp_runtime/traits/trait.AccountIdConversion.html#tymethod.into_sub_account_truncating
		pub(crate) fn account_id(task_id: &T::Hash) -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(task_id)
		}

		// Handles reputation update for profiles
		fn handle_reputation(task_id: &T::Hash) -> Result<(), DispatchError> {

			// Check if task exists
			let task = Self::tasks(&task_id).ok_or(<Error<T>>::TaskNotExist)?;

			// Ensure that reputation is added only when task is in status Accepted
			if task.status == TaskStatus::Accepted {
				pallet_profile::Pallet::<T>::add_reputation(&task.initiator)?;
				pallet_profile::Pallet::<T>::add_reputation(&task.volunteer)?;
			}

			Ok(())
		}
	}
}
