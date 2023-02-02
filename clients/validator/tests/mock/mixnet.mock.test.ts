import {
  ContractStateParams,
  LayerDistribution,
  MixnetContractVersion,
  RewardingStatus,
  StakeSaturationResponse,
} from '../../compiledTypes';
import expect from 'expect';
import { TestHelper } from './client';
import { allunbondednodes, contract, delegation, gateway, mixnode, mixnodebond, ownedNode, rewardingnode, saturation, unbondednode, layerDistribution, rewardingParams, contractVersion } from '../../types/expectedResponses';
import { mixnet, mix_id, mix_identity, rewardingIntervalNonce } from "./testData";
import { RewardingParams } from '../../compiledTypes/types/global';

describe('Mixnet mock tests', () => {

  let testHelper = new TestHelper();

  it('get Layer Distribution', () => {
    let execute = testHelper.tests('getLayerDistribution', [mixnet],
      // layerDistribution
      <LayerDistribution>{
        gateways: 10,
        layer1: 2,
        layer2: 2,
        layer3: 5,
      }
    );
    expect(execute).toBeTruthy();
  });

  it('get Reward Params', () => {
    let execute = testHelper.tests('getRewardParams', [mixnet],
      // rewardingParams
      <RewardingParams>{
        interval: {},
        rewarded_set_size: 0,
        active_set_size: 0
      }
    );
    expect(execute).toBeTruthy();
  });

  it('get Rewarding Status', () => {
    let execute = testHelper.tests('getRewardingStatus', [mixnet, mix_identity, rewardingIntervalNonce], <
      RewardingStatus
      >{
        Complete: {},
      });
    expect(execute).toBeTruthy();
  });

  it('get Stake Saturation', () => {
    let execute = testHelper.tests('getStakeSaturation', [mixnet, mix_id],
      // saturation
      <StakeSaturationResponse>{
        mix_id: 0,
        current_saturation: '',
        uncapped_saturation: '',
      }
    );
    expect(execute).toBeTruthy();
  });

  it('get State Params', () => {
    let execute = testHelper.tests('getStateParams', [mixnet],
      // contract
      <ContractStateParams>{
        minimum_mixnode_pledge: '',
        minimum_gateway_pledge: '',
        mixnode_rewarded_set_size: 240,
        mixnode_active_set_size: 240,
      }
    );
    expect(execute).toBeTruthy();
  });

  it('get Contract Version', () => {
    let execute = testHelper.tests('getContractVersion', [mixnet],
      // contractVersion
      <MixnetContractVersion>{
        build_timestamp: 'test',
        commit_branch: 'test',
        build_version: 'test',
        rustc_version: 'test',
        commit_sha: 'test',
        commit_timestamp: 'test',
      }
    );
    expect(execute).toBeTruthy();
  });
});
