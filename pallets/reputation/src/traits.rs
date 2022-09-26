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

use crate::{
   ReputationUnit,
   CredibilityUnit,
   Score,
};
use frame_support::inherent::Vec;

/// Trait used to handle the reputation of a system.
/// Opinionated so that the user must submit some for of credibility rating.
/// This should be used to weigh the votes of a consumer against their credibility.
 pub trait ReputationHandler<T: frame_system::Config> {

   /// Calculate the new reputation of a voter based of a new score given.
   fn calculate_reputation<N, P>(item: &N, score: &Vec<Score>) -> ReputationUnit
   where N: HasCredibility + HasReputation + HasAccountId<T>,
         P: Scored;

   /// Calculate the new credibility of the voter, it is used to determine how to weigh the votes.
   /// Must return a value between 0 and 1000 higher is better
   fn calculate_credibility<N: HasCredibility>(item: &N, score: &Vec<Score>) -> u16;

 }

pub trait HasReputation {

   /// Return the reputation for a given struct.
   fn get_reputation(&self) -> ReputationUnit;
}

pub trait HasCredibility {

   /// Return the credibility for a given struct.
   fn get_credibility(&self) -> CredibilityUnit;
}

pub trait HasAccountId<T: frame_system::Config> {
   fn get_account_id(&self) -> &T::AccountId;
}

