// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use super::action_controller::{Action, ActionSender};
use super::PendingAcknowledgement;
use crate::client::{
    inbound_messages::{InputMessage, InputMessageReceiver},
    real_messages_control::real_traffic_stream::{BatchRealMessageSender, RealMessage},
    topology_control::TopologyAccessor,
};
use futures::StreamExt;
use log::*;
use nymsphinx::anonymous_replies::ReplySurb;
use nymsphinx::preparer::MessagePreparer;
use nymsphinx::{acknowledgements::AckKey, addressing::clients::Recipient};
use rand::{CryptoRng, Rng};
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "reply-surb")]
use crate::client::reply_key_storage::ReplyKeyStorage;

/// Module responsible for dealing with the received messages: splitting them, creating acknowledgements,
/// putting everything into sphinx packets, etc.
/// It also makes an initial sending attempt for said messages.
pub(super) struct InputMessageListener<R>
where
    R: CryptoRng + Rng,
{
    ack_key: Arc<AckKey>,
    ack_recipient: Recipient,
    input_receiver: InputMessageReceiver,
    message_preparer: MessagePreparer<R>,
    action_sender: ActionSender,
    real_message_sender: BatchRealMessageSender,
    topology_access: TopologyAccessor,
    #[cfg(feature = "reply-surb")]
    reply_key_storage: ReplyKeyStorage,
}

impl<R> InputMessageListener<R>
where
    R: CryptoRng + Rng,
{
    pub fn get_action_sender(&self) -> ActionSender {
        self.action_sender.clone()
    }

    // at this point I'm not entirely sure how to deal with this warning without
    // some considerable refactoring
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        ack_key: Arc<AckKey>,
        ack_recipient: Recipient,
        input_receiver: InputMessageReceiver,
        message_preparer: MessagePreparer<R>,
        action_sender: ActionSender,
        real_message_sender: BatchRealMessageSender,
        topology_access: TopologyAccessor,
        #[cfg(feature = "reply-surb")] reply_key_storage: ReplyKeyStorage,
    ) -> Self {
        InputMessageListener {
            ack_key,
            ack_recipient,
            input_receiver,
            message_preparer,
            action_sender,
            real_message_sender,
            topology_access,
            #[cfg(feature = "reply-surb")]
            reply_key_storage,
        }
    }

    // we require topology for replies to generate surb_acks
    async fn handle_reply(&mut self, reply_surb: ReplySurb, data: Vec<u8>) -> Option<RealMessage> {
        let topology_permit = self.topology_access.get_read_permit().await;
        let topology = match topology_permit.try_get_valid_topology_ref(&self.ack_recipient, None) {
            Some(topology_ref) => topology_ref,
            None => {
                warn!("Could not process the message - the network topology is invalid");
                return None;
            }
        };

        match self
            .message_preparer
            .prepare_reply_for_use(data, reply_surb, topology, &self.ack_key)
            .await
        {
            Ok((mix_packet, reply_id)) => {
                // TODO: later probably write pending ack here
                // and deal with them....
                // ... somehow
                Some(RealMessage::new(mix_packet, reply_id))
            }
            Err(err) => {
                // TODO: should we have some mechanism to indicate to the user that the `reply_surb`
                // could be reused since technically it wasn't used up here?
                warn!("failed to deal with received reply surb - {:?}", err);
                None
            }
        }
    }

    async fn handle_fresh_message(
        &mut self,
        recipient: Recipient,
        content: Vec<u8>,
        with_reply_surb: bool,
    ) -> Option<Vec<RealMessage>> {
        let topology_permit = self.topology_access.get_read_permit().await;
        let topology = match topology_permit
            .try_get_valid_topology_ref(&self.ack_recipient, Some(&recipient))
        {
            Some(topology_ref) => topology_ref,
            None => {
                warn!("Could not process the message - the network topology is invalid");
                return None;
            }
        };

        // split the message, attach optional reply surb
        let (split_message, reply_key) = self
            .message_preparer
            .prepare_and_split_message(content, with_reply_surb, topology)
            .expect("somehow the topology was invalid after all!");

        #[cfg(feature = "reply-surb")]
        if let Some(reply_key) = reply_key {
            self.reply_key_storage
                .insert_encryption_key(reply_key)
                .expect("Failed to insert surb reply key to the store!")
        }

        #[cfg(not(feature = "reply-surb"))]
        let _reply_key = reply_key;

        // encrypt chunks, put them inside sphinx packets and generate acks
        let mut pending_acks = Vec::with_capacity(split_message.len());
        let mut real_messages = Vec::with_capacity(split_message.len());
        for message_chunk in split_message {
            // we need to clone it because we need to keep it in memory in case we had to retransmit
            // it. And then we'd need to recreate entire ACK again.
            let chunk_clone = message_chunk.clone();
            let prepared_fragment = self
                .message_preparer
                .prepare_chunk_for_sending(chunk_clone, topology, &self.ack_key, &recipient)
                .unwrap();

            real_messages.push(RealMessage::new(
                prepared_fragment.mix_packet,
                message_chunk.fragment_identifier(),
            ));

            pending_acks.push(PendingAcknowledgement::new(
                message_chunk,
                prepared_fragment.total_delay,
                recipient,
            ));
        }

        // tells the controller to put this into the hashmap
        self.action_sender
            .unbounded_send(Action::new_insert(pending_acks))
            .unwrap();

        Some(real_messages)
    }

    async fn on_input_message(&mut self, msg: InputMessage) {
        let real_messages = match msg {
            InputMessage::Fresh {
                recipient,
                data,
                with_reply_surb,
            } => {
                self.handle_fresh_message(recipient, data, with_reply_surb)
                    .await
            }
            InputMessage::Reply { reply_surb, data } => self
                .handle_reply(reply_surb, data)
                .await
                .map(|message| vec![message]),
        };

        // there's no point in trying to send nothing
        if let Some(real_messages) = real_messages {
            // tells real message sender (with the poisson timer) to send this to the mix network
            if self.real_message_sender.send(real_messages).await.is_err() {
                panic!();
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) async fn run_with_shutdown(&mut self, mut shutdown: task::ShutdownListener) {
        debug!("Started InputMessageListener with graceful shutdown support");

        while !shutdown.is_shutdown() {
            tokio::select! {
                input_msg = self.input_receiver.next() => match input_msg {
                    Some(input_msg) => {
                        self.on_input_message(input_msg).await;
                    },
                    None => {
                        log::trace!("InputMessageListener: Stopping since channel closed");
                        break;
                    }
                },
                _ = shutdown.recv() => {
                    log::trace!("InputMessageListener: Received shutdown");
                }
            }
        }
        assert!(shutdown.is_shutdown_poll());
        log::debug!("InputMessageListener: Exiting");
    }

    #[cfg(target_arch = "wasm32")]
    pub(super) async fn run(&mut self) {
        debug!("Started InputMessageListener without graceful shutdown support");
        while let Some(input_msg) = self.input_receiver.next().await {
            self.on_input_message(input_msg).await;
        }
    }
}

#[derive(Clone)]
pub struct FreshInputMessageChunker<R>
where
    R: CryptoRng + Rng,
{
    ack_key: Arc<AckKey>,
    ack_recipient: Recipient,
    message_preparer: MessagePreparer<R>,
    action_sender: ActionSender,
    real_message_sender: BatchRealMessageSender,
    topology_access: TopologyAccessor,
}

impl<R> FreshInputMessageChunker<R>
where
    R: CryptoRng + Rng,
{
    pub fn new(
        ack_key: Arc<AckKey>,
        ack_recipient: Recipient,
        message_preparer: MessagePreparer<R>,
        action_sender: ActionSender,
        real_message_sender: BatchRealMessageSender,
        topology_access: TopologyAccessor,
    ) -> Self {
        Self {
            ack_key,
            ack_recipient,
            message_preparer,
            action_sender,
            real_message_sender,
            topology_access,
        }
    }

    // we require topology for replies to generate surb_acks
    //async fn handle_reply(&mut self, reply_surb: ReplySurb, data: Vec<u8>) -> Option<RealMessage> {
    //    let topology_permit = self.topology_access.get_read_permit().await;
    //    let topology = match topology_permit.try_get_valid_topology_ref(&self.ack_recipient, None) {
    //        Some(topology_ref) => topology_ref,
    //        None => {
    //            warn!("Could not process the message - the network topology is invalid");
    //            return None;
    //        }
    //    };

    //    match self
    //        .message_preparer
    //        .prepare_reply_for_use(data, reply_surb, topology, &self.ack_key)
    //        .await
    //    {
    //        Ok((mix_packet, reply_id)) => {
    //            // TODO: later probably write pending ack here
    //            // and deal with them....
    //            // ... somehow
    //            Some(RealMessage::new(mix_packet, reply_id))
    //        }
    //        Err(err) => {
    //            // TODO: should we have some mechanism to indicate to the user that the `reply_surb`
    //            // could be reused since technically it wasn't used up here?
    //            warn!("failed to deal with received reply surb - {:?}", err);
    //            None
    //        }
    //    }
    //}

    async fn handle_fresh_message(
        &mut self,
        recipient: Recipient,
        content: Vec<u8>,
        with_reply_surb: bool,
    ) -> Option<Vec<RealMessage>> {
        let topology_permit = self.topology_access.get_read_permit().await;
        let topology = match topology_permit
            .try_get_valid_topology_ref(&self.ack_recipient, Some(&recipient))
        {
            Some(topology_ref) => topology_ref,
            None => {
                warn!("Could not process the message - the network topology is invalid");
                return None;
            }
        };

        // split the message, attach optional reply surb
        let (split_message, reply_key) = self
            .message_preparer
            .prepare_and_split_message(content, with_reply_surb, topology)
            .expect("somehow the topology was invalid after all!");

        //#[cfg(feature = "reply-surb")]
        //if let Some(reply_key) = reply_key {
        //    self.reply_key_storage
        //        .insert_encryption_key(reply_key)
        //        .expect("Failed to insert surb reply key to the store!")
        //}

        #[cfg(not(feature = "reply-surb"))]
        let _reply_key = reply_key;

        // encrypt chunks, put them inside sphinx packets and generate acks
        let mut pending_acks = Vec::with_capacity(split_message.len());
        let mut real_messages = Vec::with_capacity(split_message.len());
        for message_chunk in split_message {
            // we need to clone it because we need to keep it in memory in case we had to retransmit
            // it. And then we'd need to recreate entire ACK again.
            let chunk_clone = message_chunk.clone();
            let prepared_fragment = self
                .message_preparer
                .prepare_chunk_for_sending(chunk_clone, topology, &self.ack_key, &recipient)
                .unwrap();

            real_messages.push(RealMessage::new(
                prepared_fragment.mix_packet,
                message_chunk.fragment_identifier(),
            ));

            pending_acks.push(PendingAcknowledgement::new(
                message_chunk,
                prepared_fragment.total_delay,
                recipient,
            ));
        }

        // tells the controller to put this into the hashmap
        self.action_sender
            .unbounded_send(Action::new_insert(pending_acks))
            .unwrap();

        Some(real_messages)
    }

    pub async fn on_input_message(&mut self, msg: InputMessage) {
        log::info!("Using new chunker!");
        let real_messages = match msg {
            InputMessage::Fresh {
                recipient,
                data,
                with_reply_surb,
            } => {
                self.handle_fresh_message(recipient, data, with_reply_surb)
                    .await
            }
            InputMessage::Reply { .. } => panic!(),
            //InputMessage::Reply { reply_surb, data } => self
            //    .handle_reply(reply_surb, data)
            //    .await
            //    .map(|message| vec![message]),
        };

        // there's no point in trying to send nothing
        if let Some(real_messages) = real_messages {
            // tells real message sender (with the poisson timer) to send this to the mix network

            use itertools::Itertools;

            log::info!("chunked into {} messages", real_messages.len());
            log::info!("current capacity: {}", self.real_message_sender.capacity());
            if real_messages.len() > 10 {
                // only send if there is nothing in queue
                log::info!("sending large message(s)");
                let grouped: Vec<Vec<RealMessage>> = real_messages
                    .into_iter()
                    .chunks(10)
                    .into_iter()
                    .map(|c| c.collect())
                    .collect();

                for msg in grouped {
                    //let msgs = vec![msg];
                    //let msgs = msg.to_vec();
                    loop {
                        if self.real_message_sender.capacity() > 2 {
                            if self.real_message_sender.send(msg).await.is_err() {
                                panic!();
                            }
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                }
                log::error!("finished sending large message(s)");
            } else {
                if self.real_message_sender.send(real_messages).await.is_err() {
                    panic!();
                }
            }
        }
    }
}
