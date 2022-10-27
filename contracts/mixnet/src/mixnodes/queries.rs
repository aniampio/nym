// Copyright 2021-2022 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use super::storage;
use crate::constants::{
    MIXNODE_BOND_DEFAULT_RETRIEVAL_LIMIT, MIXNODE_BOND_MAX_RETRIEVAL_LIMIT,
    MIXNODE_DETAILS_DEFAULT_RETRIEVAL_LIMIT, MIXNODE_DETAILS_MAX_RETRIEVAL_LIMIT,
    UNBONDED_MIXNODES_DEFAULT_RETRIEVAL_LIMIT, UNBONDED_MIXNODES_MAX_RETRIEVAL_LIMIT,
};
use crate::mixnodes::helpers::{get_mixnode_details_by_id, get_mixnode_details_by_owner};
use crate::rewards::storage as rewards_storage;
use cosmwasm_std::{Deps, Order, StdResult, Storage};
use cw_storage_plus::Bound;
use mixnet_contract_common::mixnode::{
    MixNodeBond, MixNodeDetails, MixnodeRewardingDetailsResponse, PagedMixnodesDetailsResponse,
    PagedUnbondedMixnodesResponse, StakeSaturationResponse, UnbondedMixnodeResponse,
};
use mixnet_contract_common::{
    IdentityKey, LayerDistribution, MixId, MixOwnershipResponse, MixnodeDetailsResponse,
    PagedMixnodeBondsResponse,
};

pub fn query_mixnode_bonds_paged(
    deps: Deps<'_>,
    start_after: Option<MixId>,
    limit: Option<u32>,
) -> StdResult<PagedMixnodeBondsResponse> {
    let limit = limit
        .unwrap_or(MIXNODE_BOND_DEFAULT_RETRIEVAL_LIMIT)
        .min(MIXNODE_BOND_MAX_RETRIEVAL_LIMIT) as usize;

    let start = start_after.map(Bound::exclusive);

    let nodes = storage::mixnode_bonds()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<MixNodeBond>>>()?;

    let start_next_after = nodes.last().map(|node| node.mix_id);

    Ok(PagedMixnodeBondsResponse::new(
        nodes,
        limit,
        start_next_after,
    ))
}

fn attach_rewarding_info(
    storage: &dyn Storage,
    read_bond: StdResult<(MixId, MixNodeBond)>,
) -> StdResult<MixNodeDetails> {
    match read_bond {
        Ok((_, bond)) => {
            // if we managed to read the bond we MUST be able to also read rewarding information.
            // if we fail, this is a hard error and the query should definitely fail and we should investigate
            // the reasons for that.
            let mix_rewarding = rewards_storage::MIXNODE_REWARDING.load(storage, bond.mix_id)?;
            Ok(MixNodeDetails::new(bond, mix_rewarding))
        }
        Err(err) => Err(err),
    }
}

pub fn query_mixnodes_details_paged(
    deps: Deps<'_>,
    start_after: Option<MixId>,
    limit: Option<u32>,
) -> StdResult<PagedMixnodesDetailsResponse> {
    let limit = limit
        .unwrap_or(MIXNODE_DETAILS_DEFAULT_RETRIEVAL_LIMIT)
        .min(MIXNODE_DETAILS_MAX_RETRIEVAL_LIMIT) as usize;

    let start = start_after.map(Bound::exclusive);

    let nodes = storage::mixnode_bonds()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| attach_rewarding_info(deps.storage, res))
        .collect::<StdResult<Vec<MixNodeDetails>>>()?;

    let start_next_after = nodes.last().map(|details| details.mix_id());

    Ok(PagedMixnodesDetailsResponse::new(
        nodes,
        limit,
        start_next_after,
    ))
}

pub fn query_unbonded_mixnodes_paged(
    deps: Deps<'_>,
    start_after: Option<MixId>,
    limit: Option<u32>,
) -> StdResult<PagedUnbondedMixnodesResponse> {
    let limit = limit
        .unwrap_or(UNBONDED_MIXNODES_DEFAULT_RETRIEVAL_LIMIT)
        .min(UNBONDED_MIXNODES_MAX_RETRIEVAL_LIMIT) as usize;

    let start = start_after.map(Bound::exclusive);

    let nodes = storage::unbonded_mixnodes()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    let start_next_after = nodes.last().map(|res| res.0);

    Ok(PagedUnbondedMixnodesResponse::new(
        nodes,
        limit,
        start_next_after,
    ))
}

pub fn query_unbonded_mixnodes_by_owner_paged(
    deps: Deps<'_>,
    owner: String,
    start_after: Option<MixId>,
    limit: Option<u32>,
) -> StdResult<PagedUnbondedMixnodesResponse> {
    let owner = deps.api.addr_validate(&owner)?;

    let limit = limit
        .unwrap_or(UNBONDED_MIXNODES_DEFAULT_RETRIEVAL_LIMIT)
        .min(UNBONDED_MIXNODES_MAX_RETRIEVAL_LIMIT) as usize;

    let start = start_after.map(Bound::exclusive);

    let nodes = storage::unbonded_mixnodes()
        .idx
        .owner
        .prefix(owner)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    let start_next_after = nodes.last().map(|res| res.0);

    Ok(PagedUnbondedMixnodesResponse::new(
        nodes,
        limit,
        start_next_after,
    ))
}

pub fn query_unbonded_mixnodes_by_identity_paged(
    deps: Deps<'_>,
    identity_key: String,
    start_after: Option<MixId>,
    limit: Option<u32>,
) -> StdResult<PagedUnbondedMixnodesResponse> {
    let limit = limit
        .unwrap_or(UNBONDED_MIXNODES_DEFAULT_RETRIEVAL_LIMIT)
        .min(UNBONDED_MIXNODES_MAX_RETRIEVAL_LIMIT) as usize;

    let start = start_after.map(Bound::exclusive);

    let nodes = storage::unbonded_mixnodes()
        .idx
        .identity_key
        .prefix(identity_key)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    let start_next_after = nodes.last().map(|res| res.0);

    Ok(PagedUnbondedMixnodesResponse::new(
        nodes,
        limit,
        start_next_after,
    ))
}

pub fn query_owned_mixnode(deps: Deps<'_>, address: String) -> StdResult<MixOwnershipResponse> {
    let validated_addr = deps.api.addr_validate(&address)?;
    let mixnode_details = get_mixnode_details_by_owner(deps.storage, validated_addr.clone())?;

    Ok(MixOwnershipResponse {
        address: validated_addr,
        mixnode_details,
    })
}

pub fn query_mixnode_details(deps: Deps<'_>, mix_id: MixId) -> StdResult<MixnodeDetailsResponse> {
    let mixnode_details = get_mixnode_details_by_id(deps.storage, mix_id)?;

    Ok(MixnodeDetailsResponse {
        mix_id,
        mixnode_details,
    })
}

pub fn query_mixnode_details_by_identity(
    deps: Deps<'_>,
    mix_identity: IdentityKey,
) -> StdResult<Option<MixNodeDetails>> {
    if let Some(bond_information) = storage::mixnode_bonds()
        .idx
        .identity_key
        .item(deps.storage, mix_identity)?
        .map(|record| record.1)
    {
        // if bond exists, rewarding details MUST also exist
        let rewarding_details =
            rewards_storage::MIXNODE_REWARDING.load(deps.storage, bond_information.mix_id)?;
        Ok(Some(MixNodeDetails::new(
            bond_information,
            rewarding_details,
        )))
    } else {
        Ok(None)
    }
}

pub fn query_mixnode_rewarding_details(
    deps: Deps<'_>,
    mix_id: MixId,
) -> StdResult<MixnodeRewardingDetailsResponse> {
    let rewarding_details = rewards_storage::MIXNODE_REWARDING.may_load(deps.storage, mix_id)?;

    Ok(MixnodeRewardingDetailsResponse {
        mix_id,
        rewarding_details,
    })
}

pub fn query_unbonded_mixnode(deps: Deps<'_>, mix_id: MixId) -> StdResult<UnbondedMixnodeResponse> {
    let unbonded_info = storage::unbonded_mixnodes().may_load(deps.storage, mix_id)?;

    Ok(UnbondedMixnodeResponse {
        mix_id,
        unbonded_info,
    })
}

pub fn query_stake_saturation(deps: Deps<'_>, mix_id: MixId) -> StdResult<StakeSaturationResponse> {
    let mix_rewarding = match rewards_storage::MIXNODE_REWARDING.may_load(deps.storage, mix_id)? {
        Some(mix_rewarding) => mix_rewarding,
        None => {
            return Ok(StakeSaturationResponse {
                mix_id,
                current_saturation: None,
                uncapped_saturation: None,
            })
        }
    };

    let rewarding_params = rewards_storage::REWARDING_PARAMS.load(deps.storage)?;

    Ok(StakeSaturationResponse {
        mix_id,
        current_saturation: Some(mix_rewarding.bond_saturation(&rewarding_params)),
        uncapped_saturation: Some(mix_rewarding.uncapped_bond_saturation(&rewarding_params)),
    })
}

pub(crate) fn query_layer_distribution(deps: Deps<'_>) -> StdResult<LayerDistribution> {
    storage::LAYERS.load(deps.storage)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::interval::pending_events;
    use crate::support::tests::fixtures::good_mixnode_pledge;
    use crate::support::tests::test_helpers::TestSetup;
    use crate::support::tests::{fixtures, test_helpers};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::Decimal;

    #[cfg(test)]
    mod mixnode_bonds {
        use super::*;
        use crate::support::tests::fixtures::good_mixnode_pledge;

        #[test]
        fn obeys_limits() {
            let mut deps = test_helpers::init_contract();
            let env = mock_env();
            let mut rng = test_helpers::test_rng();
            let limit = 2;

            test_helpers::add_dummy_mixnodes(&mut rng, deps.as_mut(), env, 1000);
            let page1 = query_mixnode_bonds_paged(deps.as_ref(), None, Some(limit)).unwrap();
            assert_eq!(limit, page1.nodes.len() as u32);
        }

        #[test]
        fn has_default_limit() {
            let mut deps = test_helpers::init_contract();
            let env = mock_env();
            let mut rng = test_helpers::test_rng();

            test_helpers::add_dummy_mixnodes(&mut rng, deps.as_mut(), env, 1000);

            // query without explicitly setting a limit
            let page1 = query_mixnode_bonds_paged(deps.as_ref(), None, None).unwrap();

            assert_eq!(
                MIXNODE_BOND_DEFAULT_RETRIEVAL_LIMIT,
                page1.nodes.len() as u32
            );
        }

        #[test]
        fn has_max_limit() {
            let mut deps = test_helpers::init_contract();
            let env = mock_env();
            let mut rng = test_helpers::test_rng();
            test_helpers::add_dummy_mixnodes(&mut rng, deps.as_mut(), env, 1000);

            // query with a crazily high limit in an attempt to use too many resources
            let crazy_limit = 1000;
            let page1 = query_mixnode_bonds_paged(deps.as_ref(), None, Some(crazy_limit)).unwrap();

            // we default to a decent sized upper bound instead
            assert_eq!(MIXNODE_BOND_MAX_RETRIEVAL_LIMIT, page1.nodes.len() as u32);
        }

        #[test]
        fn pagination_works() {
            // as we add mixnodes, we're always inserting them in ascending manner due to monotonically increasing id
            let mut deps = test_helpers::init_contract();
            let env = mock_env();
            let mut rng = test_helpers::test_rng();

            test_helpers::add_mixnode(
                &mut rng,
                deps.as_mut(),
                env.clone(),
                "addr1",
                good_mixnode_pledge(),
            );

            let per_page = 2;
            let page1 = query_mixnode_bonds_paged(deps.as_ref(), None, Some(per_page)).unwrap();

            // page should have 1 result on it
            assert_eq!(1, page1.nodes.len());

            // save another
            test_helpers::add_mixnode(
                &mut rng,
                deps.as_mut(),
                env.clone(),
                "addr2",
                good_mixnode_pledge(),
            );

            // page1 should have 2 results on it
            let page1 = query_mixnode_bonds_paged(deps.as_ref(), None, Some(per_page)).unwrap();
            assert_eq!(2, page1.nodes.len());

            test_helpers::add_mixnode(
                &mut rng,
                deps.as_mut(),
                env.clone(),
                "addr3",
                good_mixnode_pledge(),
            );

            // page1 still has the same 2 results
            let another_page1 =
                query_mixnode_bonds_paged(deps.as_ref(), None, Some(per_page)).unwrap();
            assert_eq!(2, another_page1.nodes.len());
            assert_eq!(page1, another_page1);

            // retrieving the next page should start after the last key on this page
            let start_after = page1.start_next_after.unwrap();
            let page2 = query_mixnode_bonds_paged(deps.as_ref(), Some(start_after), Some(per_page))
                .unwrap();

            assert_eq!(1, page2.nodes.len());

            // save another one
            test_helpers::add_mixnode(&mut rng, deps.as_mut(), env, "addr4", good_mixnode_pledge());

            let page2 = query_mixnode_bonds_paged(deps.as_ref(), Some(start_after), Some(per_page))
                .unwrap();

            // now we have 2 pages, with 2 results on the second page
            assert_eq!(2, page2.nodes.len());
        }
    }

    #[cfg(test)]
    mod mixnode_details {
        use super::*;
        use crate::support::tests::fixtures::good_mixnode_pledge;

        #[test]
        fn obeys_limits() {
            let mut deps = test_helpers::init_contract();
            let env = mock_env();
            let mut rng = test_helpers::test_rng();
            let limit = 2;

            test_helpers::add_dummy_mixnodes(&mut rng, deps.as_mut(), env, 1000);
            let page1 = query_mixnodes_details_paged(deps.as_ref(), None, Some(limit)).unwrap();
            assert_eq!(limit, page1.nodes.len() as u32);
        }

        #[test]
        fn has_default_limit() {
            let mut deps = test_helpers::init_contract();
            let env = mock_env();
            let mut rng = test_helpers::test_rng();
            test_helpers::add_dummy_mixnodes(&mut rng, deps.as_mut(), env, 1000);

            // query without explicitly setting a limit
            let page1 = query_mixnodes_details_paged(deps.as_ref(), None, None).unwrap();

            assert_eq!(
                MIXNODE_DETAILS_DEFAULT_RETRIEVAL_LIMIT,
                page1.nodes.len() as u32
            );
        }

        #[test]
        fn has_max_limit() {
            let mut deps = test_helpers::init_contract();
            let env = mock_env();
            let mut rng = test_helpers::test_rng();
            test_helpers::add_dummy_mixnodes(&mut rng, deps.as_mut(), env, 1000);

            // query with a crazily high limit in an attempt to use too many resources
            let crazy_limit = 1000;
            let page1 =
                query_mixnodes_details_paged(deps.as_ref(), None, Some(crazy_limit)).unwrap();

            // we default to a decent sized upper bound instead
            assert_eq!(
                MIXNODE_DETAILS_MAX_RETRIEVAL_LIMIT,
                page1.nodes.len() as u32
            );
        }

        #[test]
        fn pagination_works() {
            // as we add mixnodes, we're always inserting them in ascending manner due to monotonically increasing id
            let mut deps = test_helpers::init_contract();
            let env = mock_env();
            let mut rng = test_helpers::test_rng();

            test_helpers::add_mixnode(
                &mut rng,
                deps.as_mut(),
                env.clone(),
                "addr1",
                good_mixnode_pledge(),
            );

            let per_page = 2;
            let page1 = query_mixnodes_details_paged(deps.as_ref(), None, Some(per_page)).unwrap();

            // page should have 1 result on it
            assert_eq!(1, page1.nodes.len());

            // save another
            test_helpers::add_mixnode(
                &mut rng,
                deps.as_mut(),
                env.clone(),
                "addr2",
                good_mixnode_pledge(),
            );

            // page1 should have 2 results on it
            let page1 = query_mixnodes_details_paged(deps.as_ref(), None, Some(per_page)).unwrap();
            assert_eq!(2, page1.nodes.len());

            test_helpers::add_mixnode(
                &mut rng,
                deps.as_mut(),
                env.clone(),
                "addr3",
                good_mixnode_pledge(),
            );

            // page1 still has the same 2 results
            let another_page1 =
                query_mixnodes_details_paged(deps.as_ref(), None, Some(per_page)).unwrap();
            assert_eq!(2, another_page1.nodes.len());
            assert_eq!(page1, another_page1);

            // retrieving the next page should start after the last key on this page
            let start_after = page1.start_next_after.unwrap();
            let page2 =
                query_mixnodes_details_paged(deps.as_ref(), Some(start_after), Some(per_page))
                    .unwrap();

            assert_eq!(1, page2.nodes.len());

            // save another one
            test_helpers::add_mixnode(&mut rng, deps.as_mut(), env, "addr4", good_mixnode_pledge());

            let page2 =
                query_mixnodes_details_paged(deps.as_ref(), Some(start_after), Some(per_page))
                    .unwrap();

            // now we have 2 pages, with 2 results on the second page
            assert_eq!(2, page2.nodes.len());
        }
    }

    #[cfg(test)]
    mod unbonded_mixnodes {
        use super::*;
        use cosmwasm_std::Addr;
        use mixnet_contract_common::mixnode::UnbondedMixnode;

        #[test]
        fn obeys_limits() {
            let mut deps = test_helpers::init_contract();
            let _env = mock_env();
            let mut rng = test_helpers::test_rng();
            let limit = 2;

            test_helpers::add_dummy_unbonded_mixnodes(&mut rng, deps.as_mut(), 1000);
            let page1 = query_unbonded_mixnodes_paged(deps.as_ref(), None, Some(limit)).unwrap();
            assert_eq!(limit, page1.nodes.len() as u32);
        }

        #[test]
        fn has_default_limit() {
            let mut deps = test_helpers::init_contract();
            let _env = mock_env();
            let mut rng = test_helpers::test_rng();
            test_helpers::add_dummy_unbonded_mixnodes(&mut rng, deps.as_mut(), 1000);

            // query without explicitly setting a limit
            let page1 = query_unbonded_mixnodes_paged(deps.as_ref(), None, None).unwrap();

            assert_eq!(
                UNBONDED_MIXNODES_DEFAULT_RETRIEVAL_LIMIT,
                page1.nodes.len() as u32
            );
        }

        #[test]
        fn has_max_limit() {
            let mut deps = test_helpers::init_contract();
            let _env = mock_env();
            let mut rng = test_helpers::test_rng();
            test_helpers::add_dummy_unbonded_mixnodes(&mut rng, deps.as_mut(), 1000);

            // query with a crazily high limit in an attempt to use too many resources
            let crazy_limit = 1000;
            let page1 =
                query_unbonded_mixnodes_paged(deps.as_ref(), None, Some(crazy_limit)).unwrap();

            // we default to a decent sized upper bound instead
            assert_eq!(
                UNBONDED_MIXNODES_MAX_RETRIEVAL_LIMIT,
                page1.nodes.len() as u32
            );
        }

        #[test]
        fn pagination_works() {
            fn add_unbonded(storage: &mut dyn Storage, id: MixId) {
                storage::unbonded_mixnodes()
                    .save(
                        storage,
                        id,
                        &UnbondedMixnode {
                            identity_key: format!("dummy{}", id),
                            owner: Addr::unchecked(format!("dummy{}", id)),
                            proxy: None,
                            unbonding_height: 123,
                        },
                    )
                    .unwrap();
            }

            // as we add mixnodes, we're always inserting them in ascending manner due to monotonically increasing id
            let mut deps = test_helpers::init_contract();

            add_unbonded(deps.as_mut().storage, 1);

            let per_page = 2;
            let page1 = query_unbonded_mixnodes_paged(deps.as_ref(), None, Some(per_page)).unwrap();

            // page should have 1 result on it
            assert_eq!(1, page1.nodes.len());

            // save another
            add_unbonded(deps.as_mut().storage, 2);

            // page1 should have 2 results on it
            let page1 = query_unbonded_mixnodes_paged(deps.as_ref(), None, Some(per_page)).unwrap();
            assert_eq!(2, page1.nodes.len());

            add_unbonded(deps.as_mut().storage, 3);

            // page1 still has the same 2 results
            let another_page1 =
                query_unbonded_mixnodes_paged(deps.as_ref(), None, Some(per_page)).unwrap();
            assert_eq!(2, another_page1.nodes.len());
            assert_eq!(page1, another_page1);

            // retrieving the next page should start after the last key on this page
            let start_after = page1.start_next_after.unwrap();
            let page2 =
                query_unbonded_mixnodes_paged(deps.as_ref(), Some(start_after), Some(per_page))
                    .unwrap();

            assert_eq!(1, page2.nodes.len());

            // save another one
            add_unbonded(deps.as_mut().storage, 4);
            let page2 =
                query_unbonded_mixnodes_paged(deps.as_ref(), Some(start_after), Some(per_page))
                    .unwrap();

            // now we have 2 pages, with 2 results on the second page
            assert_eq!(2, page2.nodes.len());
        }
    }

    #[cfg(test)]
    mod unbonded_mixnodes_by_owner {
        use super::*;
        use cosmwasm_std::Addr;
        use mixnet_contract_common::mixnode::UnbondedMixnode;

        fn add_unbonded_with_owner(storage: &mut dyn Storage, id: MixId, owner: &str) {
            storage::unbonded_mixnodes()
                .save(
                    storage,
                    id,
                    &UnbondedMixnode {
                        identity_key: format!("dummy{}", id),
                        owner: Addr::unchecked(owner),
                        proxy: None,
                        unbonding_height: 123,
                    },
                )
                .unwrap();
        }

        #[test]
        fn obeys_limits() {
            let mut deps = test_helpers::init_contract();
            let _env = mock_env();
            let mut rng = test_helpers::test_rng();
            let limit = 2;
            let owner = "owner";

            test_helpers::add_dummy_unbonded_mixnodes_with_owner(
                &mut rng,
                deps.as_mut(),
                owner,
                1000,
            );
            let page1 = query_unbonded_mixnodes_by_owner_paged(
                deps.as_ref(),
                owner.into(),
                None,
                Some(limit),
            )
            .unwrap();
            assert_eq!(limit, page1.nodes.len() as u32);
        }

        #[test]
        fn has_default_limit() {
            let mut deps = test_helpers::init_contract();
            let _env = mock_env();
            let mut rng = test_helpers::test_rng();
            let owner = "owner";

            test_helpers::add_dummy_unbonded_mixnodes_with_owner(
                &mut rng,
                deps.as_mut(),
                owner,
                1000,
            );

            // query without explicitly setting a limit
            let page1 =
                query_unbonded_mixnodes_by_owner_paged(deps.as_ref(), owner.into(), None, None)
                    .unwrap();

            assert_eq!(
                UNBONDED_MIXNODES_DEFAULT_RETRIEVAL_LIMIT,
                page1.nodes.len() as u32
            );
        }

        #[test]
        fn has_max_limit() {
            let mut deps = test_helpers::init_contract();
            let _env = mock_env();
            let mut rng = test_helpers::test_rng();
            let owner = "owner";

            test_helpers::add_dummy_unbonded_mixnodes_with_owner(
                &mut rng,
                deps.as_mut(),
                owner,
                1000,
            );

            // query with a crazily high limit in an attempt to use too many resources
            let crazy_limit = 1000;
            let page1 = query_unbonded_mixnodes_by_owner_paged(
                deps.as_ref(),
                owner.into(),
                None,
                Some(crazy_limit),
            )
            .unwrap();

            // we default to a decent sized upper bound instead
            assert_eq!(
                UNBONDED_MIXNODES_MAX_RETRIEVAL_LIMIT,
                page1.nodes.len() as u32
            );
        }

        #[test]
        fn pagination_works() {
            // as we add mixnodes, we're always inserting them in ascending manner due to monotonically increasing id
            let mut deps = test_helpers::init_contract();
            let owner = "owner";
            add_unbonded_with_owner(deps.as_mut().storage, 1, owner);

            let per_page = 2;
            let page1 = query_unbonded_mixnodes_by_owner_paged(
                deps.as_ref(),
                owner.into(),
                None,
                Some(per_page),
            )
            .unwrap();

            // page should have 1 result on it
            assert_eq!(1, page1.nodes.len());

            // save another
            add_unbonded_with_owner(deps.as_mut().storage, 2, owner);

            // page1 should have 2 results on it
            let page1 = query_unbonded_mixnodes_by_owner_paged(
                deps.as_ref(),
                owner.into(),
                None,
                Some(per_page),
            )
            .unwrap();
            assert_eq!(2, page1.nodes.len());

            add_unbonded_with_owner(deps.as_mut().storage, 3, owner);

            // page1 still has the same 2 results
            let another_page1 = query_unbonded_mixnodes_by_owner_paged(
                deps.as_ref(),
                owner.into(),
                None,
                Some(per_page),
            )
            .unwrap();
            assert_eq!(2, another_page1.nodes.len());
            assert_eq!(page1, another_page1);

            // retrieving the next page should start after the last key on this page
            let start_after = page1.start_next_after.unwrap();
            let page2 = query_unbonded_mixnodes_by_owner_paged(
                deps.as_ref(),
                owner.into(),
                Some(start_after),
                Some(per_page),
            )
            .unwrap();

            assert_eq!(1, page2.nodes.len());

            // save another one
            add_unbonded_with_owner(deps.as_mut().storage, 4, owner);
            let page2 = query_unbonded_mixnodes_by_owner_paged(
                deps.as_ref(),
                owner.into(),
                Some(start_after),
                Some(per_page),
            )
            .unwrap();

            // now we have 2 pages, with 2 results on the second page
            assert_eq!(2, page2.nodes.len());
        }

        #[test]
        fn only_retrieves_nodes_with_specific_owner() {
            let mut deps = test_helpers::init_contract();
            let owner1 = "owner1";
            let owner2 = "owner2";
            let owner3 = "owner3";
            let owner4 = "owner4";

            add_unbonded_with_owner(deps.as_mut().storage, 1, owner1);
            add_unbonded_with_owner(deps.as_mut().storage, 2, owner1);
            add_unbonded_with_owner(deps.as_mut().storage, 3, owner2);
            add_unbonded_with_owner(deps.as_mut().storage, 4, owner1);
            add_unbonded_with_owner(deps.as_mut().storage, 5, owner3);
            add_unbonded_with_owner(deps.as_mut().storage, 6, owner3);
            add_unbonded_with_owner(deps.as_mut().storage, 7, owner4);
            add_unbonded_with_owner(deps.as_mut().storage, 8, owner2);
            add_unbonded_with_owner(deps.as_mut().storage, 9, owner1);
            add_unbonded_with_owner(deps.as_mut().storage, 10, owner3);

            let expected_ids1 = vec![1, 2, 4, 9];
            let expected_ids2 = vec![3, 8];
            let expected_ids3 = vec![5, 6, 10];
            let expected_ids4 = vec![7];

            let res1 =
                query_unbonded_mixnodes_by_owner_paged(deps.as_ref(), owner1.into(), None, None)
                    .unwrap()
                    .nodes
                    .into_iter()
                    .map(|r| r.0)
                    .collect::<Vec<_>>();
            assert_eq!(res1, expected_ids1);

            let res2 =
                query_unbonded_mixnodes_by_owner_paged(deps.as_ref(), owner2.into(), None, None)
                    .unwrap()
                    .nodes
                    .into_iter()
                    .map(|r| r.0)
                    .collect::<Vec<_>>();
            assert_eq!(res2, expected_ids2);

            let res3 =
                query_unbonded_mixnodes_by_owner_paged(deps.as_ref(), owner3.into(), None, None)
                    .unwrap()
                    .nodes
                    .into_iter()
                    .map(|r| r.0)
                    .collect::<Vec<_>>();
            assert_eq!(res3, expected_ids3);

            let res4 =
                query_unbonded_mixnodes_by_owner_paged(deps.as_ref(), owner4.into(), None, None)
                    .unwrap()
                    .nodes
                    .into_iter()
                    .map(|r| r.0)
                    .collect::<Vec<_>>();
            assert_eq!(res4, expected_ids4);

            let res5 = query_unbonded_mixnodes_by_owner_paged(
                deps.as_ref(),
                "doesnt-exist".into(),
                None,
                None,
            )
            .unwrap()
            .nodes
            .into_iter()
            .map(|r| r.0)
            .collect::<Vec<_>>();
            assert!(res5.is_empty());
        }
    }

    #[cfg(test)]
    mod unbonded_mixnodes_by_identity {
        use super::*;
        use cosmwasm_std::Addr;
        use mixnet_contract_common::mixnode::UnbondedMixnode;

        fn add_unbonded_with_identity(storage: &mut dyn Storage, id: MixId, identity: &str) {
            storage::unbonded_mixnodes()
                .save(
                    storage,
                    id,
                    &UnbondedMixnode {
                        identity_key: identity.to_string(),
                        owner: Addr::unchecked(format!("dummy{}", id)),
                        proxy: None,
                        unbonding_height: 123,
                    },
                )
                .unwrap();
        }

        #[test]
        fn obeys_limits() {
            let mut deps = test_helpers::init_contract();
            let _env = mock_env();
            let mut rng = test_helpers::test_rng();
            let limit = 2;
            let identity = "foomp123";

            test_helpers::add_dummy_unbonded_mixnodes_with_identity(
                &mut rng,
                deps.as_mut(),
                identity,
                1000,
            );
            let page1 = query_unbonded_mixnodes_by_identity_paged(
                deps.as_ref(),
                identity.into(),
                None,
                Some(limit),
            )
            .unwrap();
            assert_eq!(limit, page1.nodes.len() as u32);
        }

        #[test]
        fn has_default_limit() {
            let mut deps = test_helpers::init_contract();
            let _env = mock_env();
            let mut rng = test_helpers::test_rng();
            let identity = "foomp123";
            test_helpers::add_dummy_unbonded_mixnodes_with_identity(
                &mut rng,
                deps.as_mut(),
                identity,
                1000,
            );

            // query without explicitly setting a limit
            let page1 = query_unbonded_mixnodes_by_identity_paged(
                deps.as_ref(),
                identity.into(),
                None,
                None,
            )
            .unwrap();

            assert_eq!(
                UNBONDED_MIXNODES_DEFAULT_RETRIEVAL_LIMIT,
                page1.nodes.len() as u32
            );
        }

        #[test]
        fn has_max_limit() {
            let mut deps = test_helpers::init_contract();
            let _env = mock_env();
            let mut rng = test_helpers::test_rng();
            let identity = "foomp123";
            test_helpers::add_dummy_unbonded_mixnodes_with_identity(
                &mut rng,
                deps.as_mut(),
                identity,
                1000,
            );

            // query with a crazily high limit in an attempt to use too many resources
            let crazy_limit = 1000;
            let page1 = query_unbonded_mixnodes_by_identity_paged(
                deps.as_ref(),
                identity.into(),
                None,
                Some(crazy_limit),
            )
            .unwrap();

            // we default to a decent sized upper bound instead
            assert_eq!(
                UNBONDED_MIXNODES_MAX_RETRIEVAL_LIMIT,
                page1.nodes.len() as u32
            );
        }

        #[test]
        fn pagination_works() {
            // as we add mixnodes, we're always inserting them in ascending manner due to monotonically increasing id
            let mut deps = test_helpers::init_contract();
            let identity = "foomp123";

            add_unbonded_with_identity(deps.as_mut().storage, 1, identity);

            let per_page = 2;
            let page1 = query_unbonded_mixnodes_by_identity_paged(
                deps.as_ref(),
                identity.into(),
                None,
                Some(per_page),
            )
            .unwrap();

            // page should have 1 result on it
            assert_eq!(1, page1.nodes.len());

            // save another
            add_unbonded_with_identity(deps.as_mut().storage, 2, identity);

            // page1 should have 2 results on it
            let page1 = query_unbonded_mixnodes_by_identity_paged(
                deps.as_ref(),
                identity.into(),
                None,
                Some(per_page),
            )
            .unwrap();
            assert_eq!(2, page1.nodes.len());

            add_unbonded_with_identity(deps.as_mut().storage, 3, identity);

            // page1 still has the same 2 results
            let another_page1 = query_unbonded_mixnodes_by_identity_paged(
                deps.as_ref(),
                identity.into(),
                None,
                Some(per_page),
            )
            .unwrap();
            assert_eq!(2, another_page1.nodes.len());
            assert_eq!(page1, another_page1);

            // retrieving the next page should start after the last key on this page
            let start_after = page1.start_next_after.unwrap();
            let page2 = query_unbonded_mixnodes_by_identity_paged(
                deps.as_ref(),
                identity.into(),
                Some(start_after),
                Some(per_page),
            )
            .unwrap();

            assert_eq!(1, page2.nodes.len());

            // save another one
            add_unbonded_with_identity(deps.as_mut().storage, 4, identity);
            let page2 = query_unbonded_mixnodes_by_identity_paged(
                deps.as_ref(),
                identity.into(),
                Some(start_after),
                Some(per_page),
            )
            .unwrap();

            // now we have 2 pages, with 2 results on the second page
            assert_eq!(2, page2.nodes.len());
        }

        #[test]
        fn only_retrieves_nodes_with_specific_identity_key() {
            let mut deps = test_helpers::init_contract();
            let identity1 = "identity1";
            let identity2 = "identity2";
            let identity3 = "identity3";
            let identity4 = "identity4";

            add_unbonded_with_identity(deps.as_mut().storage, 1, identity1);
            add_unbonded_with_identity(deps.as_mut().storage, 2, identity1);
            add_unbonded_with_identity(deps.as_mut().storage, 3, identity2);
            add_unbonded_with_identity(deps.as_mut().storage, 4, identity1);
            add_unbonded_with_identity(deps.as_mut().storage, 5, identity3);
            add_unbonded_with_identity(deps.as_mut().storage, 6, identity3);
            add_unbonded_with_identity(deps.as_mut().storage, 7, identity4);
            add_unbonded_with_identity(deps.as_mut().storage, 8, identity2);
            add_unbonded_with_identity(deps.as_mut().storage, 9, identity1);
            add_unbonded_with_identity(deps.as_mut().storage, 10, identity3);

            let expected_ids1 = vec![1, 2, 4, 9];
            let expected_ids2 = vec![3, 8];
            let expected_ids3 = vec![5, 6, 10];
            let expected_ids4 = vec![7];

            let res1 = query_unbonded_mixnodes_by_identity_paged(
                deps.as_ref(),
                identity1.into(),
                None,
                None,
            )
            .unwrap()
            .nodes
            .into_iter()
            .map(|r| r.0)
            .collect::<Vec<_>>();
            assert_eq!(res1, expected_ids1);

            let res2 = query_unbonded_mixnodes_by_identity_paged(
                deps.as_ref(),
                identity2.into(),
                None,
                None,
            )
            .unwrap()
            .nodes
            .into_iter()
            .map(|r| r.0)
            .collect::<Vec<_>>();
            assert_eq!(res2, expected_ids2);

            let res3 = query_unbonded_mixnodes_by_identity_paged(
                deps.as_ref(),
                identity3.into(),
                None,
                None,
            )
            .unwrap()
            .nodes
            .into_iter()
            .map(|r| r.0)
            .collect::<Vec<_>>();
            assert_eq!(res3, expected_ids3);

            let res4 = query_unbonded_mixnodes_by_identity_paged(
                deps.as_ref(),
                identity4.into(),
                None,
                None,
            )
            .unwrap()
            .nodes
            .into_iter()
            .map(|r| r.0)
            .collect::<Vec<_>>();
            assert_eq!(res4, expected_ids4);

            let res5 = query_unbonded_mixnodes_by_owner_paged(
                deps.as_ref(),
                "doesnt-exist".into(),
                None,
                None,
            )
            .unwrap()
            .nodes
            .into_iter()
            .map(|r| r.0)
            .collect::<Vec<_>>();
            assert!(res5.is_empty());
        }
    }

    #[test]
    fn query_for_owned_mixnode() {
        let mut deps = test_helpers::init_contract();
        let env = mock_env();
        let mut rng = test_helpers::test_rng();

        let address = "mix-owner".to_string();

        // when it doesnt exist
        let res = query_owned_mixnode(deps.as_ref(), address.clone()).unwrap();
        assert!(res.mixnode_details.is_none());
        assert_eq!(address, res.address);

        // when it [fully] exists
        let id = test_helpers::add_mixnode(
            &mut rng,
            deps.as_mut(),
            env,
            &address,
            good_mixnode_pledge(),
        );
        let res = query_owned_mixnode(deps.as_ref(), address.clone()).unwrap();
        let details = res.mixnode_details.unwrap();
        assert_eq!(address, details.bond_information.owner);
        assert_eq!(
            good_mixnode_pledge()[0],
            details.bond_information.original_pledge
        );
        assert_eq!(address, res.address);

        // when it partially exists, i.e. case when the operator unbonded, but there are still some pending delegates
        // TODO: perhaps this should work slightly differently, to return the underlying mixnode rewarding?

        // manually adjust delegation info as to indicate the rewarding information shouldnt get removed
        let mut rewarding_details = details.rewarding_details;
        rewarding_details.delegates = Decimal::raw(12345);
        rewarding_details.unique_delegations = 1;
        rewards_storage::MIXNODE_REWARDING
            .save(deps.as_mut().storage, id, &rewarding_details)
            .unwrap();

        pending_events::unbond_mixnode(deps.as_mut(), &mock_env(), 123, id).unwrap();
        let res = query_owned_mixnode(deps.as_ref(), address.clone()).unwrap();
        assert!(res.mixnode_details.is_none());
        assert_eq!(address, res.address);
    }

    #[test]
    fn query_for_mixnode_details() {
        let mut deps = test_helpers::init_contract();
        let env = mock_env();
        let mut rng = test_helpers::test_rng();

        // no node under this id
        let res = query_mixnode_details(deps.as_ref(), 42).unwrap();
        assert!(res.mixnode_details.is_none());
        assert_eq!(42, res.mix_id);

        // it exists
        let mix_id =
            test_helpers::add_mixnode(&mut rng, deps.as_mut(), env, "foomp", good_mixnode_pledge());
        let res = query_mixnode_details(deps.as_ref(), mix_id).unwrap();
        let details = res.mixnode_details.unwrap();
        assert_eq!(mix_id, details.bond_information.mix_id);
        assert_eq!(
            good_mixnode_pledge()[0],
            details.bond_information.original_pledge
        );
        assert_eq!(mix_id, res.mix_id);
    }

    #[test]
    fn query_for_mixnode_details_by_identity() {
        let mut test = TestSetup::new();

        // no node under this identity
        let res = query_mixnode_details_by_identity(test.deps(), "foomp".into()).unwrap();
        assert!(res.is_none());

        // it exists
        let mix_id = test.add_dummy_mixnode("owner", None);
        // this was already tested to be working : )
        let expected = query_mixnode_details(test.deps(), mix_id)
            .unwrap()
            .mixnode_details
            .unwrap();
        let mix_identity = expected.bond_information.identity();

        let res = query_mixnode_details_by_identity(test.deps(), mix_identity.into()).unwrap();
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn query_for_mixnode_rewarding_details() {
        let mut deps = test_helpers::init_contract();
        let env = mock_env();
        let mut rng = test_helpers::test_rng();

        // no node under this id
        let res = query_mixnode_rewarding_details(deps.as_ref(), 42).unwrap();
        assert!(res.rewarding_details.is_none());
        assert_eq!(42, res.mix_id);

        let mix_id =
            test_helpers::add_mixnode(&mut rng, deps.as_mut(), env, "foomp", good_mixnode_pledge());
        let res = query_mixnode_rewarding_details(deps.as_ref(), mix_id).unwrap();
        let details = res.rewarding_details.unwrap();
        assert_eq!(
            fixtures::mix_node_cost_params_fixture(),
            details.cost_params
        );
        assert_eq!(mix_id, res.mix_id);
    }

    #[test]
    fn query_for_unbonded_mixnode() {
        let mut deps = test_helpers::init_contract();
        let env = mock_env();
        let mut rng = test_helpers::test_rng();

        let sender = "mix-owner";

        // no node under this id
        let res = query_unbonded_mixnode(deps.as_ref(), 42).unwrap();
        assert!(res.unbonded_info.is_none());
        assert_eq!(42, res.mix_id);

        // add and unbond the mixnode
        let mix_id =
            test_helpers::add_mixnode(&mut rng, deps.as_mut(), env, sender, good_mixnode_pledge());
        pending_events::unbond_mixnode(deps.as_mut(), &mock_env(), 123, mix_id).unwrap();

        let res = query_unbonded_mixnode(deps.as_ref(), mix_id).unwrap();
        assert_eq!(res.unbonded_info.unwrap().owner, sender);
        assert_eq!(mix_id, res.mix_id);
    }

    #[test]
    fn query_for_stake_saturation() {
        let mut deps = test_helpers::init_contract();
        let env = mock_env();
        let mut rng = test_helpers::test_rng();

        // no node under this id
        let res = query_stake_saturation(deps.as_ref(), 42).unwrap();
        assert!(res.current_saturation.is_none());
        assert!(res.uncapped_saturation.is_none());
        assert_eq!(42, res.mix_id);

        let rewarding_params = rewards_storage::REWARDING_PARAMS
            .load(deps.as_ref().storage)
            .unwrap();
        let saturation_point = rewarding_params.interval.stake_saturation_point;

        let mix_id =
            test_helpers::add_mixnode(&mut rng, deps.as_mut(), env, "foomp", good_mixnode_pledge());

        // below saturation point
        // there's only the base pledge without any delegation
        let expected =
            Decimal::from_atomics(good_mixnode_pledge()[0].amount, 0).unwrap() / saturation_point;
        let res = query_stake_saturation(deps.as_ref(), mix_id).unwrap();
        assert_eq!(expected, res.current_saturation.unwrap());
        assert_eq!(expected, res.uncapped_saturation.unwrap());
        assert_eq!(mix_id, res.mix_id);

        // exactly at saturation point
        let mut mix_rewarding = rewards_storage::MIXNODE_REWARDING
            .load(deps.as_ref().storage, mix_id)
            .unwrap();
        mix_rewarding.operator = saturation_point;
        rewards_storage::MIXNODE_REWARDING
            .save(deps.as_mut().storage, mix_id, &mix_rewarding)
            .unwrap();

        let res = query_stake_saturation(deps.as_ref(), mix_id).unwrap();
        assert_eq!(Decimal::one(), res.current_saturation.unwrap());
        assert_eq!(Decimal::one(), res.uncapped_saturation.unwrap());
        assert_eq!(mix_id, res.mix_id);

        // above the saturation point
        let mut mix_rewarding = rewards_storage::MIXNODE_REWARDING
            .load(deps.as_ref().storage, mix_id)
            .unwrap();
        mix_rewarding.delegates = mix_rewarding.operator * Decimal::percent(150);
        rewards_storage::MIXNODE_REWARDING
            .save(deps.as_mut().storage, mix_id, &mix_rewarding)
            .unwrap();

        let res = query_stake_saturation(deps.as_ref(), mix_id).unwrap();
        assert_eq!(Decimal::one(), res.current_saturation.unwrap());
        assert_eq!(Decimal::percent(250), res.uncapped_saturation.unwrap());
        assert_eq!(mix_id, res.mix_id);
    }
}