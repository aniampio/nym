import { MixNode, MajorCurrencyAmount } from './rust';

export type TNodeType = 'mixnode' | 'gateway';

export type TNodeOwnership = {
  hasOwnership: boolean;
  nodeType?: TNodeType;
};

export type TDelegation = {
  owner: string;
  node_identity: string;
  amount: MajorCurrencyAmount;
  block_height: number;
  proxy: string; // proxy address used to delegate the funds on behalf of anouther address
};

export type TPagedDelegations = {
  delegations: TDelegation[];
  start_next_after: string;
};

export type TMixnodeBondDetails = {
  pledge_amount: MajorCurrencyAmount;
  total_delegation: MajorCurrencyAmount;
  owner: string;
  layer: string;
  block_height: number;
  mix_node: MixNode;
  proxy: any;
};