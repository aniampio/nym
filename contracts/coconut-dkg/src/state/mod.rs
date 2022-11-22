// Copyright 2022 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use cw4::Cw4Contract;
use cw_controllers::Admin;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// unique items
pub const STATE: Item<State> = Item::new("state");
pub const ADMIN: Admin = Admin::new("admin");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct State {
    pub mix_denom: String,
    pub group_addr: Cw4Contract,
}