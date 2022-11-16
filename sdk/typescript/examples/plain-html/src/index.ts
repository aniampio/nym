import { createNymMixnetClient, NymMixnetClient } from '@nymproject/sdk';
import { displayReceived, sendMessageTo, displaySenderAddress } from './dom-utils';

let nym: NymMixnetClient | null = null;

/**
 * The main entry point
 */
async function main() {
  nym = await createNymMixnetClient();

  if (!nym) {
    console.error('Oh no! Could not create client');
    return;
  }

  // // mixnet v1
  // const validatorApiUrl = 'https://validator.nymtech.net/api';
  // const preferredGatewayIdentityKey = 'E3mvZTHQCdBvhfr178Swx9g4QG3kkRUun7YnToLMcMbM';

  // mixnet v2
  const validatorApiUrl = 'https://qwerty-validator-api.qa.nymte.ch/api'; // "http://localhost:8081";
  const preferredGatewayIdentityKey = undefined; // '36vfvEyBzo5cWEFbnP7fqgY39kFw9PQhvwzbispeNaxL';

  // subscribe to connect event, so that we can show the client's address
  nym.events.subscribeToConnected((e) => {
    if (e.args.address) {
      displaySenderAddress(e.args.address);
    }
  });

  // subscribe to message received events and show any string messages received
  nym.events.subscribeToTextMessageReceivedEvent((e) => {
    displayReceived(e.args.payload);
  });

  const sendButton: HTMLButtonElement = document.querySelector('#send-button') as HTMLButtonElement;
  if (sendButton) {
    sendButton.onclick = function () {
      if (nym) {
        sendMessageTo(nym);
      }
    };
  }

  // start up the client
  await nym.client.start({
    clientId: 'My awesome client',
    validatorApiUrl,
    preferredGatewayIdentityKey,
  });
}

// wait for the html to load
window.addEventListener('DOMContentLoaded', () => {
  // let's do this!
  main();
});