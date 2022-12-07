// Copyright 2021-2022 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

// due to code generated by JsonSchema
#![allow(clippy::field_reassign_with_default)]

use crate::constants::TOKEN_SUPPLY;
use crate::helpers::IntoBaseDecimal;
use crate::{Addr, MixId};
use cosmwasm_std::{Coin, Decimal, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// just use a string representation of those so that we wouldn't need to bother with decoding bytes
// and trying to figure out whether they're valid, etc
pub type OwnerProxySubKey = String;
pub type StorageKey = (MixId, OwnerProxySubKey);

pub fn generate_owner_storage_subkey(address: &Addr, proxy: Option<&Addr>) -> String {
    if let Some(proxy) = &proxy {
        let key_bytes = address
            .as_bytes()
            .iter()
            .zip(proxy.as_bytes())
            .map(|(x, y)| x ^ y)
            .collect::<Vec<_>>();
        bs58::encode(key_bytes).into_string()
    } else {
        address.clone().into_string()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
pub struct Delegation {
    /// Address of the owner of this delegation.
    pub owner: Addr,

    /// Id of the MixNode that this delegation was performed against.
    pub mix_id: MixId,

    // Note to UI/UX devs: there's absolutely no point in displaying this value to the users,
    // it would serve them no purpose. It's only used for calculating rewards
    /// Value of the "unit delegation" associated with the mixnode at the time of delegation.
    pub cumulative_reward_ratio: Decimal,

    /// Original delegation amount. Note that it is never mutated as delegation accumulates rewards.
    pub amount: Coin,

    /// Block height where this delegation occurred.
    pub height: u64,

    /// Proxy address used to delegate the funds on behalf of another address
    pub proxy: Option<Addr>,
}

impl Delegation {
    pub fn new(
        owner: Addr,
        mix_id: MixId,
        cumulative_reward_ratio: Decimal,
        amount: Coin,
        height: u64,
        proxy: Option<Addr>,
    ) -> Self {
        assert!(
            amount.amount <= TOKEN_SUPPLY,
            "delegation cannot be larger than the token supply"
        );

        Delegation {
            owner,
            mix_id,
            cumulative_reward_ratio,
            amount,
            height,
            proxy,
        }
    }

    pub fn generate_storage_key(
        mix_id: MixId,
        owner_address: &Addr,
        proxy: Option<&Addr>,
    ) -> StorageKey {
        (mix_id, generate_owner_storage_subkey(owner_address, proxy))
    }

    // this function might seem a bit redundant, but I'd rather explicitly keep it around in case
    // some types change in the future
    pub fn generate_storage_key_with_subkey(
        mix_id: MixId,
        owner_proxy_subkey: OwnerProxySubKey,
    ) -> StorageKey {
        (mix_id, owner_proxy_subkey)
    }

    pub fn dec_amount(&self) -> StdResult<Decimal> {
        self.amount.amount.into_base_decimal()
    }

    pub fn proxy_storage_key(&self) -> OwnerProxySubKey {
        generate_owner_storage_subkey(&self.owner, self.proxy.as_ref())
    }

    pub fn storage_key(&self) -> StorageKey {
        Self::generate_storage_key(self.mix_id, &self.owner, self.proxy.as_ref())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, JsonSchema)]
pub struct PagedMixNodeDelegationsResponse {
    pub delegations: Vec<Delegation>,
    pub start_next_after: Option<OwnerProxySubKey>,
}

impl PagedMixNodeDelegationsResponse {
    pub fn new(delegations: Vec<Delegation>, start_next_after: Option<OwnerProxySubKey>) -> Self {
        PagedMixNodeDelegationsResponse {
            delegations,
            start_next_after,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, JsonSchema)]
pub struct PagedDelegatorDelegationsResponse {
    pub delegations: Vec<Delegation>,
    pub start_next_after: Option<(MixId, OwnerProxySubKey)>,
}

impl PagedDelegatorDelegationsResponse {
    pub fn new(
        delegations: Vec<Delegation>,
        start_next_after: Option<(MixId, OwnerProxySubKey)>,
    ) -> Self {
        PagedDelegatorDelegationsResponse {
            delegations,
            start_next_after,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, JsonSchema)]
pub struct MixNodeDelegationResponse {
    pub delegation: Option<Delegation>,
    pub mixnode_still_bonded: bool,
}

impl MixNodeDelegationResponse {
    pub fn new(delegation: Option<Delegation>, mixnode_still_bonded: bool) -> Self {
        MixNodeDelegationResponse {
            delegation,
            mixnode_still_bonded,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, JsonSchema)]
pub struct PagedAllDelegationsResponse {
    pub delegations: Vec<Delegation>,
    pub start_next_after: Option<StorageKey>,
}

impl PagedAllDelegationsResponse {
    pub fn new(delegations: Vec<Delegation>, start_next_after: Option<StorageKey>) -> Self {
        PagedAllDelegationsResponse {
            delegations,
            start_next_after,
        }
    }
}
