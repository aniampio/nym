import bs58 from 'bs58'
import { minor, valid } from 'semver'
import { invoke } from '@tauri-apps/api'

export const validateKey = (key: string): boolean => {
  // it must be a valid base58 key
  try {
    const bytes = bs58.decode(key)
    // of length 32
    return bytes.length === 32
  } catch (e) {
    console.log(e)
    return false
  }
}

export const validateAmount = async (
  rawValue: string,
  minimum: string
): Promise<boolean> => {
  // tests basic coin value requirements, like no more than 6 decimal places, value lower than total supply, etc
  if (!basicRawCoinValueValidation(rawValue)) {
    return false
  }

  try {
    const nativeValueString: string = await invoke(
      'printable_balance_to_native',
      { amount: rawValue }
    )

    let nativeValue = parseInt(nativeValueString)

    return nativeValue >= parseInt(minimum)
  } catch (e) {
    console.log(e)
    return false
  }

  // this conversion seems really iffy but I'm not sure how to better approach it
}

export const basicRawCoinValueValidation = (rawAmount: string): boolean => {
  let amountFloat = parseFloat(rawAmount)
  if (isNaN(amountFloat)) {
    return false
  }

  // it cannot have more than 6 decimal places
  if (amountFloat != parseFloat(amountFloat.toFixed(6))) {
    return false
  }

  // it cannot be larger than the total supply
  if (amountFloat > 1_000_000_000_000_000) {
    return false
  }

  // it can't be lower than one micro coin
  return amountFloat >= 0.000001
}

export const isValidHostname = (value: string) => {
  // regex for ipv4 and ipv6 and hhostname- source http://jsfiddle.net/DanielD/8S4nq/
  const hostnameRegex =
    /((^\s*((([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5]))\s*$)|(^\s*((([0-9A-Fa-f]{1,4}:){7}([0-9A-Fa-f]{1,4}|:))|(([0-9A-Fa-f]{1,4}:){6}(:[0-9A-Fa-f]{1,4}|((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3})|:))|(([0-9A-Fa-f]{1,4}:){5}(((:[0-9A-Fa-f]{1,4}){1,2})|:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3})|:))|(([0-9A-Fa-f]{1,4}:){4}(((:[0-9A-Fa-f]{1,4}){1,3})|((:[0-9A-Fa-f]{1,4})?:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:))|(([0-9A-Fa-f]{1,4}:){3}(((:[0-9A-Fa-f]{1,4}){1,4})|((:[0-9A-Fa-f]{1,4}){0,2}:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:))|(([0-9A-Fa-f]{1,4}:){2}(((:[0-9A-Fa-f]{1,4}){1,5})|((:[0-9A-Fa-f]{1,4}){0,3}:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:))|(([0-9A-Fa-f]{1,4}:){1}(((:[0-9A-Fa-f]{1,4}){1,6})|((:[0-9A-Fa-f]{1,4}){0,4}:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:))|(:(((:[0-9A-Fa-f]{1,4}){1,7})|((:[0-9A-Fa-f]{1,4}){0,5}:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:)))(%.+)?\s*$))|(^\s*((?=.{1,255}$)(?=.*[A-Za-z].*)[0-9A-Za-z](?:(?:[0-9A-Za-z]|\b-){0,61}[0-9A-Za-z])?(?:\.[0-9A-Za-z](?:(?:[0-9A-Za-z]|\b-){0,61}[0-9A-Za-z])?)*)\s*$)/

  return hostnameRegex.test(value)
}

export const validateVersion = (version: string): boolean => {
  try {
    const minorVersion = minor(version)
    const validVersion = valid(version)
    return validVersion !== null && minorVersion >= 11
  } catch (e) {
    console.log(e)
    return false
  }
}
