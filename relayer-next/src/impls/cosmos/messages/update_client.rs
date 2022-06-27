use async_trait::async_trait;
use ibc::Height;
use ibc_relayer::chain::handle::ChainHandle;

use crate::impls::cosmos::chain_types::CosmosChainTypes;
use crate::impls::cosmos::error::Error;
use crate::impls::cosmos::handler::CosmosRelayHandler;
use crate::impls::cosmos::message::CosmosIbcMessage;
use crate::impls::cosmos::relay_types::CosmosRelayTypes;
use crate::impls::cosmos::target::CosmosChainTarget;
use crate::traits::messages::update_client::UpdateClientMessageBuilder;
use crate::traits::target::ChainTarget;

#[async_trait]
impl<SrcChain, DstChain, Target> UpdateClientMessageBuilder<CosmosRelayTypes, Target>
    for CosmosRelayHandler<SrcChain, DstChain>
where
    SrcChain: ChainHandle,
    DstChain: ChainHandle,
    Target: ChainTarget<
        CosmosRelayTypes,
        TargetChain = CosmosChainTypes,
        CounterpartyChain = CosmosChainTypes,
    >,
    Self: CosmosChainTarget<Target>,
{
    async fn build_update_client_messages(
        &self,
        height: Height,
    ) -> Result<Vec<CosmosIbcMessage>, Error> {
        let messages = self
            .target_foreign_client()
            .build_update_client_with_trusted(height, None)
            .map_err(Error::foreign_client)?;

        let ibc_messages = messages
            .into_iter()
            .map(|any| CosmosIbcMessage::new(Some(height), |_| Ok(any)))
            .collect();

        Ok(ibc_messages)
    }
}
