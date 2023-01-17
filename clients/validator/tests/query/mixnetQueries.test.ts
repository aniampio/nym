import ValidatorClient from '../../src';
import expect from 'expect';
const dotenv = require('dotenv');

dotenv.config();

describe('Mixnet queries', () => {
  let client: ValidatorClient;

  beforeEach(async () => {
    client = await ValidatorClient.connectForQuery(
      process.env.rpcAddress || '',
      process.env.validatorAddress || '',
      process.env.prefix || '',
      process.env.mixnetContractAddress || '',
      process.env.vestingContractAddress || '',
      process.env.denom || '',
    );
  });

  it('can query for an account balance', async () => {
    const balance = await client.getBalance('n1ptg680vnmef2cd8l0s9uyc4f0hgf3x8sed6w77');
    expect(Number.parseFloat(balance.amount)).toBeGreaterThan(0);
  });

  it('can query for stake saturation', async () => {
    const stakeSaturation = await client.getStakeSaturation(8);
    expect(stakeSaturation).toBeTruthy();
    expect(stakeSaturation?.current_saturation).toBeTruthy();
  });

  it('can query for contract version', async () => {
    const contract = await client.getMixnetContractVersion();
    expect(contract).toBeTruthy();
  });

  it('can query for unbonded mixnodes', async () => {
    const unbondedNodes = await client.getUnbondedMixNodes();
    expect(unbondedNodes).toBeTruthy();
    expect(Array.isArray(unbondedNodes)).toBeTruthy();
  });

  it('can query for unbonded mixnode information', async () => {
    const unbondedMixnodeInfo = await client.getUnbondedMixNodeInformation(1);
    expect(unbondedMixnodeInfo).toBeTruthy();
  });

  it('can query for mixnode rewarding details', async () => {
    const rewardingDetails = await client.getMixnodeRewardingDetails(1);
    expect(rewardingDetails).toBeTruthy();
  });

  it('can query for owned mixnode', async () => {
    const ownedMixnode = await client.getOwnedMixnode('n1ptg680vnmef2cd8l0s9uyc4f0hgf3x8sed6w77');
    expect(ownedMixnode).toBeTruthy();
  });

  it('can query for all mixnode bonds', async () => {
    const mixnodeBonds = await client.getMixNodeBonds();
    expect(mixnodeBonds).toBeTruthy();
    expect(Array.isArray(mixnodeBonds)).toBeTruthy();
  });

  it('can query for all mixnode details', async () => {
    const mixnodeDetails = await client.getMixNodesDetailed();
    expect(mixnodeDetails).toBeTruthy();
    expect(Array.isArray(mixnodeDetails)).toBeTruthy();
  });
});