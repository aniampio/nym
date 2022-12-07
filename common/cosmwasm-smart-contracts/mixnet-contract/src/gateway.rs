// Copyright 2021-2022 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

// due to code generated by JsonSchema
#![allow(clippy::field_reassign_with_default)]

use crate::{IdentityKey, SphinxKey};
use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::Display;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize, JsonSchema)]
pub struct Gateway {
    pub host: String,
    pub mix_port: u16,
    pub clients_port: u16,
    pub location: String,
    pub sphinx_key: SphinxKey,
    /// Base58 encoded ed25519 EdDSA public key of the gateway used to derive shared keys with clients
    pub identity_key: IdentityKey,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, JsonSchema)]
pub struct GatewayBond {
    pub pledge_amount: Coin,
    pub owner: Addr,
    pub block_height: u64,
    pub gateway: Gateway,
    pub proxy: Option<Addr>,
}

impl GatewayBond {
    pub fn new(
        pledge_amount: Coin,
        owner: Addr,
        block_height: u64,
        gateway: Gateway,
        proxy: Option<Addr>,
    ) -> Self {
        GatewayBond {
            pledge_amount,
            owner,
            block_height,
            gateway,
            proxy,
        }
    }

    pub fn identity(&self) -> &String {
        &self.gateway.identity_key
    }

    pub fn pledge_amount(&self) -> Coin {
        self.pledge_amount.clone()
    }

    pub fn owner(&self) -> &Addr {
        &self.owner
    }

    pub fn gateway(&self) -> &Gateway {
        &self.gateway
    }
}

impl PartialOrd for GatewayBond {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // first remove invalid cases
        if self.pledge_amount.denom != other.pledge_amount.denom {
            return None;
        }

        // try to order by total pledge
        let pledge_cmp = self
            .pledge_amount
            .amount
            .partial_cmp(&other.pledge_amount.amount)?;
        if pledge_cmp != Ordering::Equal {
            return Some(pledge_cmp);
        }

        // then check block height
        let height_cmp = self.block_height.partial_cmp(&other.block_height)?;
        if height_cmp != Ordering::Equal {
            return Some(height_cmp);
        }

        // finally go by the rest of the fields in order. It doesn't really matter at this point
        // but we should be deterministic.
        let owner_cmp = self.owner.partial_cmp(&other.owner)?;
        if owner_cmp != Ordering::Equal {
            return Some(owner_cmp);
        }

        self.gateway.partial_cmp(&other.gateway)
    }
}

impl Display for GatewayBond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "amount: {} {}, owner: {}, identity: {}",
            self.pledge_amount.amount,
            self.pledge_amount.denom,
            self.owner,
            self.gateway.identity_key
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, JsonSchema)]
pub struct PagedGatewayResponse {
    pub nodes: Vec<GatewayBond>,
    pub per_page: usize,
    pub start_next_after: Option<IdentityKey>,
}

impl PagedGatewayResponse {
    pub fn new(
        nodes: Vec<GatewayBond>,
        per_page: usize,
        start_next_after: Option<IdentityKey>,
    ) -> Self {
        PagedGatewayResponse {
            nodes,
            per_page,
            start_next_after,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, JsonSchema)]
pub struct GatewayOwnershipResponse {
    pub address: Addr,
    pub gateway: Option<GatewayBond>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, JsonSchema)]
pub struct GatewayBondResponse {
    pub identity: IdentityKey,
    pub gateway: Option<GatewayBond>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gateway_fixture() -> Gateway {
        Gateway {
            host: "1.1.1.1".to_string(),
            mix_port: 123,
            clients_port: 456,
            location: "foomplandia".to_string(),
            sphinx_key: "sphinxkey".to_string(),
            identity_key: "identitykey".to_string(),
            version: "0.11.0".to_string(),
        }
    }

    #[test]
    fn gateway_bond_partial_ord() {
        let _150foos = Coin::new(150, "foo");
        let _140foos = Coin::new(140, "foo");
        let _50foos = Coin::new(50, "foo");
        let _0foos = Coin::new(0, "foo");

        let gate1 = GatewayBond {
            pledge_amount: _150foos.clone(),
            owner: Addr::unchecked("foo1"),
            block_height: 100,
            gateway: gateway_fixture(),
            proxy: None,
        };

        let gate2 = GatewayBond {
            pledge_amount: _150foos,
            owner: Addr::unchecked("foo2"),
            block_height: 120,
            gateway: gateway_fixture(),
            proxy: None,
        };

        let gate3 = GatewayBond {
            pledge_amount: _50foos,
            owner: Addr::unchecked("foo3"),
            block_height: 120,
            gateway: gateway_fixture(),
            proxy: None,
        };

        let gate4 = GatewayBond {
            pledge_amount: _140foos,
            owner: Addr::unchecked("foo4"),
            block_height: 120,
            gateway: gateway_fixture(),
            proxy: None,
        };

        let gate5 = GatewayBond {
            pledge_amount: _0foos,
            owner: Addr::unchecked("foo5"),
            block_height: 120,
            gateway: gateway_fixture(),
            proxy: None,
        };

        // summary:
        // gate1: 150bond, foo1, 100
        // gate2: 150bond, foo2, 120
        // gate3: 50bond, foo3, 120
        // gate4: 140bond, foo4, 120
        // gate5: 0bond, foo5, 120

        // highest total bond is used
        // finally just the rest of the fields

        // gate1 has higher total than gate4 or gate5
        assert!(gate1 > gate4);
        assert!(gate1 > gate5);

        // gate1 has the same total as gate3, however, gate1 has more tokens in bond
        assert!(gate1 > gate3);
        // same case for gate4 and gate5
        assert!(gate4 > gate5);

        // same bond and delegation, so it's just ordered by height
        assert!(gate1 < gate2);
    }
}
