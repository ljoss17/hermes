use async_trait::async_trait;

use crate::base::traits::contexts::chain::ChainContext;
use crate::base::traits::contexts::relay::RelayContext;
use crate::std_prelude::*;

#[async_trait]
pub trait PacketAccountQuerier<Relay: RelayContext> {
    async fn query_packet_account(
        relay: &Relay,
        packet: &Relay::Packet,
    ) -> Result<<Relay::SrcChain as ChainContext>::Signer, Relay::Error>;
}