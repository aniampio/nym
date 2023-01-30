import type { DecCoin } from './DecCoin';
export interface Delegation {
    owner: string;
    mix_id: number;
    cumulative_reward_ratio: string;
    amount: DecCoin;
    height: bigint;
    proxy: string | null;
}
//# sourceMappingURL=Delegation.d.ts.map