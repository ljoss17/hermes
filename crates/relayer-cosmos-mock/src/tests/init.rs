use basecoin_store::impls::InMemoryStore;
use ibc::core::ics24_host::identifier::ChainId;
use ibc_relayer_runtime::types::runtime::TokioRuntimeContext;
use std::str::FromStr;
use std::sync::Arc;
use tokio::runtime::Runtime as TokioRuntime;

use crate::contexts::basecoin::MockBasecoin;
use crate::contexts::chain::MockCosmosContext;
use crate::contexts::relay::MockCosmosRelay;

pub fn mock_basecoin_binary_stand() -> (
    Arc<MockCosmosContext<MockBasecoin<InMemoryStore>>>,
    Arc<MockCosmosContext<MockBasecoin<InMemoryStore>>>,
    MockCosmosRelay<MockBasecoin<InMemoryStore>, MockBasecoin<InMemoryStore>>,
) {
    let runtime = TokioRuntimeContext::new(Arc::new(
        TokioRuntime::new().expect("failed to build runtime"),
    ));

    // Source chain setup
    let src_chain_id = ChainId::from_str("mock-cosmos-chain-0").expect("never fails");
    let src_chain = Arc::new(MockBasecoin::new_default(src_chain_id));
    src_chain.run();

    let src_chain_ctx = Arc::new(MockCosmosContext::new(runtime.clone(), src_chain));
    src_chain_ctx.sync();

    // Destination chain setup
    let dst_chain_id = ChainId::from_str("mock-cosmos-chain-1").expect("never fails");
    let dst_chain = Arc::new(MockBasecoin::new_default(dst_chain_id));
    dst_chain.run();

    let dst_chain_ctx = Arc::new(MockCosmosContext::new(runtime.clone(), dst_chain));
    dst_chain_ctx.sync();

    // Relayer setup
    let relayer = MockCosmosRelay::new(runtime, src_chain_ctx.clone(), dst_chain_ctx.clone())
        .expect("failed to build relayer");

    (src_chain_ctx, dst_chain_ctx, relayer)
}
