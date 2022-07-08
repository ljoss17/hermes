use alloc::sync::Arc;
use async_trait::async_trait;
use core::marker::PhantomData;
use core::time::Duration;
use std::collections::VecDeque;
use std::time::Instant;
use tokio::sync::mpsc::{channel, error::TryRecvError, Receiver, Sender};
use tokio::sync::oneshot::{channel as once_channel, Sender as OnceSender};
use tokio::task;
use tokio::time::sleep;

use crate::traits::chain_context::IbcChainContext;
use crate::traits::core::Async;
use crate::traits::ibc_message_sender::IbcMessageSender;
use crate::traits::message::Message as ChainMessage;
use crate::traits::relay_context::RelayContext;
use crate::traits::target::ChainTarget;

pub struct ChannelClosedError;

pub struct BatchedMessageSender<Sender>(PhantomData<Sender>);

pub struct MessageBatch<Message, Event, Error> {
    messages: Vec<Message>,
    result_sender: OnceSender<Result<Vec<Vec<Event>>, Error>>,
}

pub struct MessageSink<Message, Event, Error> {
    sender: Sender<MessageBatch<Message, Event, Error>>,
}

pub trait BatchMessageContext<Message, Event, Error> {
    fn message_sink(&self) -> &MessageSink<Message, Event, Error>;
}

#[async_trait]
impl<Context, Target, TargetChain, Message, Event, Error, Sender> IbcMessageSender<Context, Target>
    for BatchedMessageSender<Sender>
where
    Message: Async,
    Event: Async,
    Error: Async,
    Sender: Async,
    Error: From<ChannelClosedError>,
    Context: RelayContext<Error = Error>,
    Context: BatchMessageContext<Message, Event, Error>,
    Target: ChainTarget<Context, TargetChain = TargetChain>,
    TargetChain: IbcChainContext<Target::CounterpartyChain, IbcMessage = Message, IbcEvent = Event>,
{
    async fn send_messages(
        context: &Context,
        messages: Vec<Message>,
    ) -> Result<Vec<Vec<Event>>, Context::Error> {
        let (result_sender, receiver) = once_channel();

        let batch = MessageBatch {
            messages,
            result_sender,
        };

        context
            .message_sink()
            .sender
            .send(batch)
            .await
            .map_err(|_| ChannelClosedError)?;

        let events = receiver.await.map_err(|_| ChannelClosedError)??;

        Ok(events)
    }
}

impl<Sender> BatchedMessageSender<Sender> {
    pub async fn new_sink<Context, Target, Message, Event, Error>(
        context: Arc<Context>,
        max_message_count: usize,
        max_tx_size: usize,
        buffer_size: usize,
        max_delay: Duration,
        sleep_time: Duration,
    ) -> MessageSink<Message, Event, Error>
    where
        Event: Async,
        Error: Clone + Async,
        Context: RelayContext<Error = Error>,
        Message: ChainMessage + Async,
        Target: ChainTarget<Context>,
        Sender: IbcMessageSender<Context, Target>,
        Target::TargetChain:
            IbcChainContext<Target::CounterpartyChain, IbcMessage = Message, IbcEvent = Event>,
    {
        let (sender, receiver) = channel(buffer_size);

        task::spawn(async move {
            Self::run_loop(
                &context,
                max_message_count,
                max_tx_size,
                max_delay,
                sleep_time,
                receiver,
            )
            .await;
        });

        MessageSink { sender }
    }

    async fn run_loop<Context, Target, Message, Event, Error>(
        context: &Context,
        max_message_count: usize,
        max_tx_size: usize,
        max_delay: Duration,
        sleep_time: Duration,
        mut receiver: Receiver<MessageBatch<Message, Event, Error>>,
    ) where
        Error: Clone,
        Context: RelayContext<Error = Error>,
        Message: ChainMessage,
        Target: ChainTarget<Context>,
        Sender: IbcMessageSender<Context, Target>,
        Target::TargetChain:
            IbcChainContext<Target::CounterpartyChain, IbcMessage = Message, IbcEvent = Event>,
    {
        let mut last_sent_time = Instant::now();
        let mut pending_batches: VecDeque<MessageBatch<Message, Event, Error>> = VecDeque::new();

        loop {
            match receiver.try_recv() {
                Ok(batch) => {
                    pending_batches.push_back(batch);
                    let current_batch_size = pending_batches.len();
                    pending_batches = Self::process_message_batches(
                        context,
                        max_message_count,
                        max_tx_size,
                        max_delay,
                        &mut last_sent_time,
                        pending_batches,
                    )
                    .await;

                    if pending_batches.len() == current_batch_size {
                        sleep(sleep_time).await;
                    }
                }
                Err(TryRecvError::Empty) => {
                    sleep(sleep_time).await;
                }
                Err(TryRecvError::Disconnected) => {
                    return;
                }
            }
        }
    }

    async fn process_message_batches<Context, Target, Message, Event, Error>(
        context: &Context,
        max_message_count: usize,
        max_tx_size: usize,
        max_delay: Duration,
        last_sent_time: &mut Instant,
        pending_batches: VecDeque<MessageBatch<Message, Event, Error>>,
    ) -> VecDeque<MessageBatch<Message, Event, Error>>
    where
        Error: Clone,
        Context: RelayContext<Error = Error>,
        Message: ChainMessage,
        Target: ChainTarget<Context>,
        Sender: IbcMessageSender<Context, Target>,
        Target::TargetChain:
            IbcChainContext<Target::CounterpartyChain, IbcMessage = Message, IbcEvent = Event>,
    {
        let batch_result =
            partition_message_batches(max_message_count, max_tx_size, pending_batches);

        let now = Instant::now();

        if batch_result.ready_batches.is_empty() {
            // If there is nothing to send, return the remaining batches which should also be empty
            batch_result.remaining_batches
        } else if
        // If the current batch is not full and there is still some time until max delay,
        // return everything and wait until the next batch is full
        batch_result.remaining_batches.is_empty()
            && now.duration_since(*last_sent_time) < max_delay
        {
            batch_result.ready_batches
        } else {
            Self::send_ready_batches(context, batch_result.ready_batches).await;
            *last_sent_time = now;

            batch_result.remaining_batches
        }
    }

    async fn send_ready_batches<Context, Target, TargetChain, Message, Event, Error>(
        context: &Context,
        ready_batches: VecDeque<
            MessageBatch<TargetChain::IbcMessage, TargetChain::IbcEvent, Error>,
        >,
    ) where
        Context: RelayContext<Error = Error>,
        Target: ChainTarget<Context, TargetChain = TargetChain>,
        Sender: IbcMessageSender<Context, Target>,
        TargetChain:
            IbcChainContext<Target::CounterpartyChain, IbcMessage = Message, IbcEvent = Event>,
        Error: Clone,
    {
        let (messages, senders): (Vec<_>, Vec<_>) = ready_batches
            .into_iter()
            .map(|batch| {
                let message_count = batch.messages.len();
                (batch.messages, (message_count, batch.result_sender))
            })
            .unzip();

        let in_messages = messages.into_iter().flatten().collect::<Vec<_>>();

        let send_result = Sender::send_messages(context, in_messages).await;

        match send_result {
            Err(e) => {
                for (_, sender) in senders.into_iter() {
                    let _ = sender.send(Err(e.clone()));
                }
            }
            Ok(all_events) => {
                let mut all_events = all_events.into_iter();
                for (message_count, sender) in senders.into_iter() {
                    let events = take(&mut all_events, message_count);
                    let _ = sender.send(Ok(events));
                }
            }
        }
    }
}

fn take<T, I: Iterator<Item = T>>(it: &mut I, count: usize) -> Vec<T> {
    let mut res = Vec::new();
    for _ in 0..count {
        match it.next() {
            Some(x) => {
                res.push(x);
            }
            None => {
                return res;
            }
        }
    }
    res
}

fn batch_size<Message: ChainMessage>(messages: &[Message]) -> usize {
    messages
        .iter()
        .map(|message| {
            // return 0 on encoding error, as we don't want
            // the batching operation to error out.
            message.estimate_len().unwrap_or(0)
        })
        .sum()
}

struct BatchResult<Message, Event, Error> {
    ready_batches: VecDeque<MessageBatch<Message, Event, Error>>,
    remaining_batches: VecDeque<MessageBatch<Message, Event, Error>>,
}

fn partition_message_batches<Message, Event, Error>(
    max_message_count: usize,
    max_tx_size: usize,
    batches: VecDeque<MessageBatch<Message, Event, Error>>,
) -> BatchResult<Message, Event, Error>
where
    Message: ChainMessage,
{
    let mut total_message_count: usize = 0;
    let mut total_batch_size: usize = 0;

    let (mut ready_batches, mut remaining_batches): (VecDeque<_>, _) =
        batches.into_iter().partition(|batch| {
            if total_message_count > max_message_count || total_batch_size > max_tx_size {
                false
            } else {
                let current_message_count = batch.messages.len();
                let current_batch_size = batch_size(&batch.messages);

                if total_message_count + current_message_count > max_message_count
                    || total_batch_size + current_batch_size > max_tx_size
                {
                    false
                } else {
                    total_message_count += current_message_count;
                    total_batch_size += current_batch_size;

                    true
                }
            }
        });

    // If for some reason ready batch is empty but remaining batches is not,
    // it means there are single batch that are too big to fit in.
    // In that case put the first remaining batch as ready.
    if ready_batches.is_empty() && !remaining_batches.is_empty() {
        remaining_batches.pop_front().and_then(|batch| {
            ready_batches.push_back(batch);
            Some(())
        });
    }

    BatchResult {
        ready_batches,
        remaining_batches,
    }
}