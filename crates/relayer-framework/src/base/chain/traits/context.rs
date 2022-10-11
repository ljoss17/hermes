//! The base chain contexts upon which higher level chain contexts such as
//! [`OfaBaseChain`] are composed from.
//!
//! These traits can be implemented over the default `OfaBaseChain` trait if the
//! behavior exposed by that trait and the `AfoBaseChain` trait are not desired.

use crate::base::core::traits::runtime::context::HasRuntime;
use crate::base::core::traits::sync::Async;

/// The minimal datatypes that any chain needs to expose.
pub trait ChainContext: HasRuntime {
    type Height: Async;

    type Timestamp: Async;

    type Message: Async;

    type RawMessage: Async;

    type Signer: Async;

    type Event: Async;

    fn encode_message(
        message: &Self::Message,
        signer: &Self::Signer,
    ) -> Result<Self::RawMessage, Self::Error>;

    fn estimate_message_len(message: &Self::Message) -> Result<usize, Self::Error>;
}

/// The datatypes that IBC chains need to expose in addition to the datatypes
/// exposed by the base [`ChainContext`].
///
/// Each [`IbcChainContext`] is parameterized by a [`Counterparty`] chain
/// which must also implement the `ChainContext` trait.
pub trait IbcChainContext<Counterparty>: ChainContext
where
    Counterparty: ChainContext,
{
    type ClientId: Async;

    type ConnectionId: Async;

    type ChannelId: Async;

    type PortId: Async;

    type Sequence: Async;

    fn counterparty_message_height(message: &Self::Message) -> Option<Counterparty::Height>;
}