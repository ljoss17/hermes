use crate::base::one_for_all::impls::chain::OfaConsensusStateQuerier;
use crate::base::one_for_all::impls::relay::OfaUpdateClientMessageBuilder;
use crate::base::one_for_all::impls::status::OfaBaseChainStatusQuerier;
use crate::base::one_for_all::traits::chain::{OfaBaseChain, OfaIbcChain};
use crate::base::one_for_all::traits::components::chain::{
    OfaBaseChainComponents, OfaIbcChainComponents,
};
use crate::base::one_for_all::traits::components::relay::OfaBaseRelayComponents;
use crate::base::one_for_all::traits::relay::OfaBaseRelay;
use crate::base::relay::impls::message_senders::chain_sender::SendIbcMessagesToChain;
use crate::base::relay::impls::message_senders::update_client::SendIbcMessagesWithUpdateClient;
use crate::base::relay::impls::messages::skip_update_client::SkipUpdateClient;
use crate::base::relay::impls::messages::wait_update_client::WaitUpdateClient;
use crate::base::relay::impls::packet_relayers::base_ack_packet::BaseAckPacketRelayer;
use crate::base::relay::impls::packet_relayers::base_receive_packet::BaseReceivePacketRelayer;
use crate::base::relay::impls::packet_relayers::full_relay::FullRelayer;
use crate::base::relay::impls::packet_relayers::retry::RetryRelayer;
use crate::base::relay::impls::packet_relayers::skip_received_packet::SkipReceivedPacketRelayer;
use crate::base::relay::impls::packet_relayers::timeout_unordered_packet::BaseTimeoutUnorderedPacketRelayer;

pub struct BaseComponents;

impl<Chain> OfaBaseChainComponents<Chain> for BaseComponents
where
    Chain: OfaBaseChain,
{
    type ChainStatusQuerier = OfaBaseChainStatusQuerier;
}

impl<Chain, Counterparty> OfaIbcChainComponents<Chain, Counterparty> for BaseComponents
where
    Chain: OfaIbcChain<Counterparty>,
    Counterparty: OfaIbcChain<Chain>,
{
    type ConsensusStateQuerier = OfaConsensusStateQuerier;
}

impl<Relay> OfaBaseRelayComponents<Relay> for BaseComponents
where
    Relay: OfaBaseRelay<Components = BaseComponents>,
{
    type PacketRelayer = RetryRelayer<FullRelayer>;

    type ReceivePacketRelayer = SkipReceivedPacketRelayer<BaseReceivePacketRelayer>;

    type AckPacketRelayer = BaseAckPacketRelayer;

    type TimeoutUnorderedPacketRelayer = BaseTimeoutUnorderedPacketRelayer;

    type UpdateClientMessageBuilder =
        SkipUpdateClient<WaitUpdateClient<OfaUpdateClientMessageBuilder>>;

    type IbcMessageSender = SendIbcMessagesWithUpdateClient<SendIbcMessagesToChain>;
}