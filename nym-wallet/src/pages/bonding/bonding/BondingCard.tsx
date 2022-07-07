import React, { useContext, useEffect, useReducer } from 'react';
import { Box, Button, Typography } from '@mui/material';
import { Link } from '@nymproject/react/link/Link';
import { Gateway, MajorCurrencyAmount, MixNode } from '@nymproject/types';
import { ConfirmationModal, NymCard } from '../../../components';
import NodeIdentityModal from './NodeIdentityModal';
import { ACTIONTYPE, AmountData, BondState, FormStep, NodeData, NodeType } from '../types';
import AmountModal from './AmountModal';
import { AppContext, urls } from '../../../context';
import SummaryModal from './SummaryModal';
import { bond, vestingBond } from '../../../requests';
import { TBondArgs } from '../../../types';
import { Console } from '../../../utils/console';

const initialState: BondState = {
  showModal: false,
  formStep: 1,
};

function reducer(state: BondState, action: ACTIONTYPE) {
  let step;
  switch (action.type) {
    case 'change_bond_type':
      return { ...state, type: action.payload };
    case 'set_node_data':
      return { ...state, nodeData: action.payload };
    case 'set_amount_data':
      return { ...state, amountData: action.payload };
    case 'set_step':
      return { ...state, formStep: action.payload };
    case 'set_tx':
      return { ...state, tx: action.payload };
    case 'next_step':
      step = state.formStep + 1;
      return { ...state, formStep: step <= 4 ? (step as FormStep) : 4 };
    case 'prev_step':
      step = state.formStep - 1;
      return { ...state, formStep: step >= 1 ? (step as FormStep) : 1 };
    case 'show_modal':
      return { ...state, showModal: true };
    case 'close_modal':
      return { ...state, showModal: false };
    case 'reset':
      return initialState;
    default:
      throw new Error();
  }
}

const BondingCard = () => {
  const [state, dispatch] = useReducer(reducer, initialState);
  const { formStep, showModal } = state;
  console.log(state);

  const { userBalance, clientDetails, network } = useContext(AppContext);

  useEffect(() => {
    dispatch({ type: 'reset' });
  }, [clientDetails]);

  const formatData = (nodeType: NodeType, nodeData: NodeData, amountData: AmountData): MixNode | Gateway =>
    nodeType === 'mixnode'
      ? {
          host: nodeData.host,
          mix_port: nodeData.mixPort,
          verloc_port: nodeData.verlocPort,
          http_api_port: nodeData.httpApiPort,
          sphinx_key: nodeData.sphinxKey,
          identity_key: nodeData.identityKey,
          version: nodeData.version,
          profit_margin_percent: amountData.profitMargin as number,
        }
      : {
          host: nodeData.host,
          mix_port: nodeData.mixPort,
          clients_port: nodeData.clientsPort,
          location: nodeData.location as string,
          sphinx_key: nodeData.sphinxKey,
          identity_key: nodeData.identityKey,
          version: nodeData.version,
        };

  const onSubmit = async () => {
    const { nodeData, amountData } = state;
    if (!nodeData || !amountData) {
      throw new Error('');
    }
    const request = amountData.tokenPool === 'balance' ? bond : vestingBond;
    dispatch({ type: 'next_step' });
    return request({
      type: nodeData.nodeType,
      ownerSignature: nodeData.signature,
      [nodeData.nodeType]: formatData(nodeData.nodeType, nodeData, amountData),
      pledge: amountData.amount,
    } as TBondArgs)
      .then(async (tx) => {
        if (amountData.tokenPool === 'balance') {
          await userBalance.fetchBalance();
        } else {
          await userBalance.fetchTokenAllocation();
        }
        dispatch({ type: 'set_tx', payload: tx });
        dispatch({ type: 'next_step' });
      })
      .catch((e: any) => {
        Console.error('Failed to bond', e);
        // TODO do something
      });
  };

  const onConfirm = () => {
    dispatch({ type: 'close_modal' });
    dispatch({ type: 'reset' });
  };

  return (
    <NymCard title="Bonding">
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          pt: 0,
        }}
      >
        <Typography>Bond a node or a gateway</Typography>
        <Button
          disabled={false}
          variant="contained"
          color="primary"
          type="button"
          disableElevation
          onClick={() => dispatch({ type: 'show_modal' })}
          sx={{ py: 1.5, px: 3 }}
        >
          Bond
        </Button>
      </Box>
      {formStep === 1 && showModal && (
        <NodeIdentityModal
          open={formStep === 1 && showModal}
          onClose={() => dispatch({ type: 'reset' })}
          onSubmit={async (data) => {
            dispatch({ type: 'set_node_data', payload: data });
            dispatch({ type: 'next_step' });
          }}
        />
      )}
      {formStep === 2 && showModal && (
        <AmountModal
          open={formStep === 2 && showModal}
          onClose={() => dispatch({ type: 'reset' })}
          onSubmit={async (data) => {
            dispatch({ type: 'set_amount_data', payload: data });
            dispatch({ type: 'next_step' });
          }}
          nodeType={state.nodeData?.nodeType || 'mixnode'}
        />
      )}
      {formStep === 3 && showModal && (
        <SummaryModal
          open={formStep === 3 && showModal}
          onClose={() => dispatch({ type: 'reset' })}
          onCancel={() => dispatch({ type: 'prev_step' })}
          onSubmit={onSubmit}
          nodeType={state.nodeData?.nodeType as NodeType}
          identityKey={state.nodeData?.identityKey as string}
          amount={state.amountData?.amount as MajorCurrencyAmount}
        />
      )}
      {formStep === 4 && showModal && (
        <ConfirmationModal
          open={formStep === 4 && showModal}
          onConfirm={onConfirm}
          onClose={onConfirm}
          title="Bonding successful"
          confirmButton="Done"
          maxWidth="xs"
          fullWidth
        >
          <Link href={`${urls(network).blockExplorer}/transaction/${state.tx?.transaction_hash}`} noIcon>
            View on blockchain
          </Link>
        </ConfirmationModal>
      )}
    </NymCard>
  );
};

export default BondingCard;