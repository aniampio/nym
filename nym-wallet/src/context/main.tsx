import React, { useMemo, createContext, useEffect, useState } from 'react';
import { useHistory } from 'react-router-dom';
import { useSnackbar } from 'notistack';
import { TLoginType } from 'src/pages/sign-in/types';
import { Account, Network, TCurrency, TMixnodeBondDetails, AccountEntry } from '../types';
import { TUseuserBalance, useGetBalance } from '../hooks/useGetBalance';
import { config } from '../../config';
import {
  getMixnodeBondDetails,
  listAccounts,
  selectNetwork,
  signInWithMnemonic,
  signInWithPassword,
  signOut,
} from '../requests';
import { currencyMap } from '../utils';
import { Console } from '../utils/console';

export const { ADMIN_ADDRESS, IS_DEV_MODE } = config;

export const urls = (networkName?: Network) =>
  networkName === 'MAINNET'
    ? {
        blockExplorer: 'https://blocks.nymtech.net',
        networkExplorer: 'https://explorer.nymtech.net',
      }
    : {
        blockExplorer: `https://${networkName}-blocks.nymtech.net`,
        networkExplorer: `https://${networkName}-explorer.nymtech.net`,
      };

type TClientContext = {
  mode: 'light' | 'dark';
  clientDetails?: Account;
  storedAccounts?: AccountEntry[];
  mixnodeDetails?: TMixnodeBondDetails | null;
  userBalance: TUseuserBalance;
  showAdmin: boolean;
  showSettings: boolean;
  network?: Network;
  currency?: TCurrency;
  isLoading: boolean;
  error?: string;
  setIsLoading: (isLoading: boolean) => void;
  setError: (value?: string) => void;
  switchNetwork: (network: Network) => void;
  getBondDetails: () => Promise<void>;
  handleShowSettings: () => void;
  handleShowAdmin: () => void;
  logIn: (opts: { type: 'mnemonic' | 'password'; value: string }) => void;
  signInWithPassword: (password: string) => void;
  logOut: () => void;
  onAccountChange: (mnemonic: string) => void;
};

export const ClientContext = createContext({} as TClientContext);

export const ClientContextProvider = ({ children }: { children: React.ReactNode }) => {
  const [clientDetails, setClientDetails] = useState<Account>();
  const [storedAccounts, setStoredAccounts] = useState<AccountEntry[]>();
  const [mixnodeDetails, setMixnodeDetails] = useState<TMixnodeBondDetails | null>();
  const [network, setNetwork] = useState<Network | undefined>();
  const [currency, setCurrency] = useState<TCurrency>();
  const [showAdmin, setShowAdmin] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [mode] = useState<'light' | 'dark'>('light');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string>();

  const userBalance = useGetBalance(clientDetails?.client_address);
  const history = useHistory();
  const { enqueueSnackbar } = useSnackbar();

  const clearState = () => {
    userBalance.clearAll();
    setStoredAccounts(undefined);
    setNetwork(undefined);
    setError(undefined);
    setIsLoading(false);
    setMixnodeDetails(undefined);
  };

  const loadAccount = async (n: Network) => {
    try {
      const client = await selectNetwork(n);
      setClientDetails(client);
    } catch (e) {
      enqueueSnackbar('Error loading account', { variant: 'error' });
      Console.error(e as string);
    } finally {
      setCurrency(currencyMap(n));
    }
  };

  const getBondDetails = async () => {
    setMixnodeDetails(undefined);
    try {
      const mixnode = await getMixnodeBondDetails();
      setMixnodeDetails(mixnode);
    } catch (e) {
      Console.error(e as string);
    }
  };

  useEffect(() => {
    const refreshAccount = async () => {
      if (network) {
        await loadAccount(network);
      }
    };
    refreshAccount();
  }, [network]);

  const loadAccounts = async () => {
    const accs = await listAccounts();
    setStoredAccounts(accs);
  };

  useEffect(() => {
    if (!clientDetails) clearState();
  }, [clientDetails]);

  const logIn = async ({ type, value }: { type: TLoginType; value: string }) => {
    if (value.length === 0) {
      setError(`A ${type} must be provided`);
      return;
    }
    try {
      setIsLoading(true);
      if (type === 'mnemonic') {
        await signInWithMnemonic(value);
      } else {
        await signInWithPassword(value);
        await loadAccounts();
      }
      setNetwork('MAINNET');
      history.push('/balance');
    } catch (e) {
      setError(e as string);
    } finally {
      setIsLoading(false);
    }
  };

  const logOut = async () => {
    await signOut();
    setClientDetails(undefined);
    enqueueSnackbar('Successfully logged out', { variant: 'success' });
  };

  const onAccountChange = async (value: string) => {
    clearState();
    await signOut();
    await logIn({ type: 'mnemonic', value });
    enqueueSnackbar('Account switch success', { variant: 'success', preventDuplicate: true });
  };

  const handleShowAdmin = () => setShowAdmin((show) => !show);
  const handleShowSettings = () => setShowSettings((show) => !show);
  const switchNetwork = (_network: Network) => setNetwork(_network);

  const memoizedValue = useMemo(
    () => ({
      mode,
      isLoading,
      error,
      clientDetails,
      storedAccounts,
      mixnodeDetails,
      userBalance,
      showAdmin,
      showSettings,
      network,
      currency,
      setIsLoading,
      setError,
      signInWithPassword,
      switchNetwork,
      getBondDetails,
      handleShowSettings,
      handleShowAdmin,
      logIn,
      logOut,
      onAccountChange,
    }),
    [mode, isLoading, error, clientDetails, mixnodeDetails, userBalance, showAdmin, showSettings, network, currency],
  );

  return <ClientContext.Provider value={memoizedValue}>{children}</ClientContext.Provider>;
};
