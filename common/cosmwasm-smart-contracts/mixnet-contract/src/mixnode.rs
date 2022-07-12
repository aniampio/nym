// due to code generated by JsonSchema
#![allow(clippy::field_reassign_with_default)]

use crate::error::MixnetContractError;
use crate::reward_params::RewardParams;
use crate::{Delegation, IdentityKey, SphinxKey};
use crate::{ONE, U128};
use az::CheckedCast;
use cosmwasm_std::{coin, Addr, Coin, Uint128};
use log::error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::cmp::Ordering;
use std::fmt::Display;

#[cfg_attr(feature = "generate-ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "generate-ts",
    ts(export_to = "ts-packages/types/src/types/rust/RewardedSetNodeStatus.ts")
)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub enum RewardedSetNodeStatus {
    Active,
    Standby,
}

impl RewardedSetNodeStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, RewardedSetNodeStatus::Active)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum DelegationEvent {
    Delegate(Delegation),
    Undelegate(PendingUndelegate),
}

impl DelegationEvent {
    pub fn delegation_amount(&self) -> Option<Coin> {
        match self {
            DelegationEvent::Delegate(delegation) => Some(delegation.amount.clone()),
            // I think it would be nice to also expose an amount here to know how much we're undelegating
            DelegationEvent::Undelegate(_) => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct PendingUndelegate {
    mix_identity: IdentityKey,
    delegate: Addr,
    proxy: Option<Addr>,
    block_height: u64,
}

impl PendingUndelegate {
    pub fn new(
        mix_identity: IdentityKey,
        delegate: Addr,
        proxy: Option<Addr>,
        block_height: u64,
    ) -> Self {
        Self {
            mix_identity,
            delegate,
            proxy,
            block_height,
        }
    }

    pub fn mix_identity(&self) -> IdentityKey {
        self.mix_identity.clone()
    }

    pub fn delegate(&self) -> Addr {
        self.delegate.clone()
    }

    pub fn proxy(&self) -> Option<Addr> {
        self.proxy.clone()
    }

    pub fn block_height(&self) -> u64 {
        self.block_height
    }

    pub fn proxy_storage_key(&self) -> Vec<u8> {
        if let Some(proxy) = &self.proxy {
            self.delegate()
                .as_bytes()
                .iter()
                .zip(proxy.as_bytes())
                .map(|(x, y)| x ^ y)
                .collect()
        } else {
            self.delegate().as_bytes().to_vec()
        }
    }

    pub fn storage_key(&self) -> (IdentityKey, Vec<u8>) {
        (self.mix_identity(), self.proxy_storage_key())
    }

    pub fn delegation_key(&self, block_height: u64) -> (IdentityKey, Vec<u8>, u64) {
        (self.mix_identity(), self.proxy_storage_key(), block_height)
    }

    pub fn event_storage_key(&self) -> (Vec<u8>, u64, IdentityKey) {
        (
            self.proxy_storage_key(),
            self.block_height(),
            self.mix_identity(),
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize, JsonSchema)]
pub struct MixNode {
    pub host: String,
    pub mix_port: u16,
    pub verloc_port: u16,
    pub http_api_port: u16,
    pub sphinx_key: SphinxKey,
    /// Base58 encoded ed25519 EdDSA public key.
    pub identity_key: IdentityKey,
    pub version: String,
    pub profit_margin_percent: u8,
}

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize_repr,
    Deserialize_repr,
    JsonSchema,
)]
#[repr(u8)]
pub enum Layer {
    Gateway = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

impl From<Layer> for String {
    fn from(layer: Layer) -> Self {
        if layer == Layer::Gateway {
            "gateway".to_string()
        } else {
            (layer as u8).to_string()
        }
    }
}

// cosmwasm's limited serde doesn't work with U128 directly
#[allow(non_snake_case)]
pub mod fixed_U128_as_string {
    use super::U128;
    use serde::de::Error;
    use serde::Deserialize;
    use std::str::FromStr;

    pub fn serialize<S>(val: &U128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = (*val).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U128, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        U128::from_str(&s).map_err(|err| {
            D::Error::custom(format!(
                "failed to deserialize U128 with its string representation - {}",
                err
            ))
        })
    }
}

// everything required to reward delegator of given mixnode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct DelegatorRewardParams {
    reward_params: RewardParams,

    // to be completely honest I don't understand all consequences of using `#[schemars(with = "String")]`
    // for U128 here, but it seems that CosmWasm is using the same attribute for their Uint128
    #[schemars(with = "String")]
    #[serde(with = "fixed_U128_as_string")]
    sigma: U128,
    #[schemars(with = "String")]
    #[serde(with = "fixed_U128_as_string")]
    profit_margin: U128,
    #[schemars(with = "String")]
    #[serde(with = "fixed_U128_as_string")]
    node_profit: U128,
}

impl DelegatorRewardParams {
    pub fn new(
        sigma: U128,
        profit_margin: U128,
        node_profit: U128,
        reward_params: RewardParams,
    ) -> Self {
        DelegatorRewardParams {
            sigma,
            profit_margin,
            node_profit,
            reward_params,
        }
    }

    pub fn determine_delegation_reward(&self, delegation_amount: Uint128) -> u128 {
        if self.sigma == 0 {
            return 0;
        }

        // change all values into their fixed representations
        let delegation_amount = U128::from_num(delegation_amount.u128());
        let staking_supply = U128::from_num(self.reward_params.staking_supply());

        let scaled_delegation_amount = delegation_amount / staking_supply;

        // Div by zero checked above
        let delegator_reward =
            (ONE - self.profit_margin) * (scaled_delegation_amount / self.sigma) * self.node_profit;

        let reward = delegator_reward.max(U128::ZERO);
        if let Some(int_reward) = reward.checked_cast() {
            int_reward
        } else {
            error!(
                "Could not cast delegator reward ({}) to u128, returning 0",
                reward,
            );
            0u128
        }
    }

    pub fn node_reward_params(&self) -> RewardParams {
        self.reward_params
    }
}

#[derive(Debug, Clone, JsonSchema, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct StoredNodeRewardResult {
    reward: Uint128,

    #[schemars(with = "String")]
    #[serde(with = "fixed_U128_as_string")]
    lambda: U128,

    #[schemars(with = "String")]
    #[serde(with = "fixed_U128_as_string")]
    sigma: U128,
}

impl StoredNodeRewardResult {
    pub fn reward(&self) -> Uint128 {
        self.reward
    }

    pub fn lambda(&self) -> U128 {
        self.lambda
    }

    pub fn sigma(&self) -> U128 {
        self.sigma
    }
}

impl TryFrom<NodeRewardResult> for StoredNodeRewardResult {
    type Error = MixnetContractError;

    fn try_from(node_reward_result: NodeRewardResult) -> Result<Self, Self::Error> {
        Ok(StoredNodeRewardResult {
            reward: Uint128::new(
                node_reward_result
                    .reward()
                    .checked_cast()
                    .ok_or(MixnetContractError::CastError)?,
            ),
            lambda: node_reward_result.lambda(),
            sigma: node_reward_result.sigma(),
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NodeRewardResult {
    reward: U128,
    lambda: U128,
    sigma: U128,
}

impl NodeRewardResult {
    pub fn reward(&self) -> U128 {
        self.reward
    }

    pub fn lambda(&self) -> U128 {
        self.lambda
    }

    pub fn sigma(&self) -> U128 {
        self.sigma
    }
}

pub struct RewardEstimate {
    pub total_node_reward: u64,
    pub operator_reward: u64,
    pub delegators_reward: u64,
    pub node_profit: u64,
    pub operator_cost: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub struct MixNodeBond {
    pub pledge_amount: Coin,
    pub total_delegation: Coin,
    pub owner: Addr,
    pub layer: Layer,
    pub block_height: u64,
    pub mix_node: MixNode,
    pub proxy: Option<Addr>,
    pub accumulated_rewards: Option<Uint128>,
}

impl MixNodeBond {
    pub fn new(
        pledge_amount: Coin,
        owner: Addr,
        layer: Layer,
        block_height: u64,
        mix_node: MixNode,
        proxy: Option<Addr>,
    ) -> Self {
        MixNodeBond {
            total_delegation: coin(0, &pledge_amount.denom),
            pledge_amount,
            owner,
            layer,
            block_height,
            mix_node,
            proxy,
            accumulated_rewards: None,
        }
    }

    pub fn accumulated_rewards(&self) -> Uint128 {
        self.accumulated_rewards.unwrap_or_else(Uint128::zero)
    }

    pub fn profit_margin(&self) -> U128 {
        U128::from_num(self.mix_node.profit_margin_percent) / U128::from_num(100)
    }

    pub fn identity(&self) -> &String {
        &self.mix_node.identity_key
    }

    pub fn pledge_amount(&self) -> Coin {
        self.pledge_amount.clone()
    }

    pub fn owner(&self) -> &Addr {
        &self.owner
    }

    pub fn mix_node(&self) -> &MixNode {
        &self.mix_node
    }

    // Takes into account accumulated rewards as well as current pledge and delegation amounts
    pub fn total_bond(&self) -> Option<u128> {
        if self.pledge_amount.denom != self.total_delegation.denom {
            None
        } else {
            Some(
                self.pledge_amount.amount.u128()
                    + self.total_delegation.amount.u128()
                    + self.accumulated_rewards().u128(),
            )
        }
    }

    pub fn total_delegation(&self) -> Coin {
        self.total_delegation.clone()
    }

    pub fn stake_saturation(&self, staking_supply: u128, rewarded_set_size: u32) -> U128 {
        self.total_bond_to_staking_supply(staking_supply) * U128::from_num(rewarded_set_size)
    }

    // TODO: There is an effect here when adding accumulted rewards to the total bond, ie accumulated rewards will not
    // affect lambda, but will affect sigma, in turn over time, if left unclaimed operator rewards will not compound, but
    // behave similarly to delegations.
    // The question is should this be taken into account when calculating operator rewards?
    pub fn pledge_to_staking_supply(&self, staking_supply: u128) -> U128 {
        U128::from_num(self.pledge_amount().amount.u128()) / U128::from_num(staking_supply)
    }

    pub fn total_bond_to_staking_supply(&self, staking_supply: u128) -> U128 {
        U128::from_num(self.pledge_amount().amount.u128() + self.total_delegation().amount.u128())
            / U128::from_num(staking_supply)
    }

    pub fn lambda_ticked(&self, params: &RewardParams) -> U128 {
        // Ratio of a bond to the token circulating supply
        self.lambda(params).min(params.one_over_k())
    }

    pub fn lambda(&self, params: &RewardParams) -> U128 {
        // Ratio of a bond to the token circulating supply
        self.pledge_to_staking_supply(params.staking_supply())
    }

    pub fn sigma_ticked(&self, params: &RewardParams) -> U128 {
        // Ratio of a delegation to the the token circulating supply
        self.sigma(params).min(params.one_over_k())
    }

    pub fn sigma(&self, params: &RewardParams) -> U128 {
        // Ratio of a delegation to the the token circulating supply
        self.total_bond_to_staking_supply(params.staking_supply())
    }

    pub fn estimate_reward(
        &self,
        base_operator_cost: u64,
        params: &RewardParams,
    ) -> Result<RewardEstimate, MixnetContractError> {
        let total_node_reward = self
            .reward(params)
            .reward()
            .checked_to_num::<u128>()
            .unwrap_or_default();
        let node_profit = self
            .node_profit(params, base_operator_cost)
            .checked_to_num::<u128>()
            .unwrap_or_default();
        let operator_cost = params
            .node
            .operator_cost(base_operator_cost)
            .checked_to_num::<u128>()
            .unwrap_or_default();
        let operator_reward = self.operator_reward(params, base_operator_cost);
        // Total reward has to be the sum of operator and delegator rewards
        let delegators_reward = node_profit.saturating_sub(operator_reward);

        Ok(RewardEstimate {
            total_node_reward: total_node_reward.try_into()?,
            operator_reward: operator_reward.try_into()?,
            delegators_reward: delegators_reward.try_into()?,
            node_profit: node_profit.try_into()?,
            operator_cost: operator_cost.try_into()?,
        })
    }

    // keybase://chat/nymtech#dev-core/14473
    pub fn reward(&self, params: &RewardParams) -> NodeRewardResult {
        let lambda_ticked = self.lambda_ticked(params);
        let sigma_ticked = self.sigma_ticked(params);

        let reward = params.performance()
            * params.epoch_reward_pool()
            * (sigma_ticked * params.omega()
                + params.alpha() * lambda_ticked * sigma_ticked * params.rewarded_set_size())
            / (ONE + params.alpha());

        // we only need regular lambda and sigma to calculate operator and delegator rewards
        NodeRewardResult {
            reward,
            lambda: self.lambda(params),
            sigma: self.sigma(params),
        }
    }

    pub fn node_profit(&self, params: &RewardParams, base_operator_cost: u64) -> U128 {
        self.reward(params)
            .reward()
            .saturating_sub(params.node.operator_cost(base_operator_cost))
    }

    pub fn operator_reward(&self, params: &RewardParams, base_operator_cost: u64) -> u128 {
        let reward = self.reward(params);
        if reward.sigma == 0u128 {
            return 0;
        }

        let profit = reward
            .reward
            .saturating_sub(params.node.operator_cost(base_operator_cost));

        let operator_base_reward = reward
            .reward
            .min(params.node.operator_cost(base_operator_cost));
        // Div by zero checked above
        let operator_reward = (self.profit_margin()
            + (ONE - self.profit_margin()) * reward.lambda / reward.sigma)
            * profit;

        let reward = (operator_reward + operator_base_reward).max(U128::from_num(0));

        if let Some(int_reward) = reward.checked_cast() {
            int_reward
        } else {
            error!(
                "Could not cast reward ({}) to u128, returning 0 - mixnode {}",
                reward,
                self.identity()
            );
            0u128
        }
    }

    pub fn sigma_ratio(&self, params: &RewardParams) -> U128 {
        if self.total_bond_to_staking_supply(params.staking_supply()) < params.one_over_k() {
            self.total_bond_to_staking_supply(params.staking_supply())
        } else {
            params.one_over_k()
        }
    }

    pub fn reward_delegation(
        &self,
        delegation_amount: Uint128,
        params: &RewardParams,
        base_operator_cost: u64,
    ) -> u128 {
        let reward_params = DelegatorRewardParams::new(
            self.sigma(params),
            self.profit_margin(),
            self.node_profit(params, base_operator_cost),
            params.to_owned(),
        );
        reward_params.determine_delegation_reward(delegation_amount)
    }
}

impl PartialOrd for MixNodeBond {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // first remove invalid cases
        if self.pledge_amount.denom != self.total_delegation.denom {
            return None;
        }

        if other.pledge_amount.denom != other.total_delegation.denom {
            return None;
        }

        if self.pledge_amount.denom != other.pledge_amount.denom {
            return None;
        }

        // try to order by total bond + delegation
        let total_cmp = (self.pledge_amount.amount + self.total_delegation.amount)
            .partial_cmp(&(self.pledge_amount.amount + self.total_delegation.amount))?;

        if total_cmp != Ordering::Equal {
            return Some(total_cmp);
        }

        // then if those are equal, prefer higher bond over delegation
        let pledge_cmp = self
            .pledge_amount
            .amount
            .partial_cmp(&other.pledge_amount.amount)?;
        if pledge_cmp != Ordering::Equal {
            return Some(pledge_cmp);
        }

        // then look at delegation (I'm not sure we can get here, but better safe than sorry)
        let delegation_cmp = self
            .total_delegation
            .amount
            .partial_cmp(&other.total_delegation.amount)?;
        if delegation_cmp != Ordering::Equal {
            return Some(delegation_cmp);
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

        let layer_cmp = self.layer.partial_cmp(&other.layer)?;
        if layer_cmp != Ordering::Equal {
            return Some(layer_cmp);
        }

        self.mix_node.partial_cmp(&other.mix_node)
    }
}

impl Display for MixNodeBond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "amount: {} {}, owner: {}, identity: {}",
            self.pledge_amount.amount,
            self.pledge_amount.denom,
            self.owner,
            self.mix_node.identity_key
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub struct PagedMixnodeResponse {
    pub nodes: Vec<MixNodeBond>,
    pub per_page: usize,
    pub start_next_after: Option<IdentityKey>,
}

impl PagedMixnodeResponse {
    pub fn new(
        nodes: Vec<MixNodeBond>,
        per_page: usize,
        start_next_after: Option<IdentityKey>,
    ) -> Self {
        PagedMixnodeResponse {
            nodes,
            per_page,
            start_next_after,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub struct MixOwnershipResponse {
    pub address: Addr,
    pub mixnode: Option<MixNodeBond>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, JsonSchema)]
pub struct MixnodeBondResponse {
    pub identity: IdentityKey,
    pub mixnode: Option<MixNodeBond>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mixnode_fixture() -> MixNode {
        MixNode {
            host: "1.1.1.1".to_string(),
            mix_port: 123,
            verloc_port: 456,
            http_api_port: 789,
            sphinx_key: "sphinxkey".to_string(),
            identity_key: "identitykey".to_string(),
            version: "0.11.0".to_string(),
            profit_margin_percent: 10,
        }
    }

    #[test]
    fn mixnode_bond_partial_ord() {
        let _150foos = Coin::new(150, "foo");
        let _50foos = Coin::new(50, "foo");
        let _0foos = Coin::new(0, "foo");

        let mix1 = MixNodeBond {
            pledge_amount: _150foos.clone(),
            total_delegation: _50foos.clone(),
            owner: Addr::unchecked("foo1"),
            layer: Layer::One,
            block_height: 100,
            mix_node: mixnode_fixture(),
            proxy: None,
            accumulated_rewards: Some(Uint128::zero()),
        };

        let mix2 = MixNodeBond {
            pledge_amount: _150foos.clone(),
            total_delegation: _50foos.clone(),
            owner: Addr::unchecked("foo2"),
            layer: Layer::One,
            block_height: 120,
            mix_node: mixnode_fixture(),
            proxy: None,
            accumulated_rewards: Some(Uint128::zero()),
        };

        let mix3 = MixNodeBond {
            pledge_amount: _50foos,
            total_delegation: _150foos.clone(),
            owner: Addr::unchecked("foo3"),
            layer: Layer::One,
            block_height: 120,
            mix_node: mixnode_fixture(),
            proxy: None,
            accumulated_rewards: Some(Uint128::zero()),
        };

        let mix4 = MixNodeBond {
            pledge_amount: _150foos.clone(),
            total_delegation: _0foos.clone(),
            owner: Addr::unchecked("foo4"),
            layer: Layer::One,
            block_height: 120,
            mix_node: mixnode_fixture(),
            proxy: None,
            accumulated_rewards: Some(Uint128::zero()),
        };

        let mix5 = MixNodeBond {
            pledge_amount: _0foos,
            total_delegation: _150foos,
            owner: Addr::unchecked("foo5"),
            layer: Layer::One,
            block_height: 120,
            mix_node: mixnode_fixture(),
            proxy: None,
            accumulated_rewards: Some(Uint128::zero()),
        };

        // summary:
        // mix1: 150bond + 50delegation, foo1, 100
        // mix2: 150bond + 50delegation, foo2, 120
        // mix3: 50bond + 150delegation, foo3, 120
        // mix4: 150bond + 0delegation, foo4, 120
        // mix5: 0bond + 150delegation, foo5, 120

        // highest total bond+delegation is used
        // then bond followed by delegation
        // finally just the rest of the fields

        // mix1 has higher total than mix4 or mix5
        assert!(mix1 > mix4);
        assert!(mix1 > mix5);

        // mix1 has the same total as mix3, however, mix1 has more tokens in bond
        assert!(mix1 > mix3);
        // same case for mix4 and mix5
        assert!(mix4 > mix5);

        // same bond and delegation, so it's just ordered by height
        assert!(mix1 < mix2);
    }
}
