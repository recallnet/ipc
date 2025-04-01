use anyhow::anyhow;
use fvm_ipld_blockstore::{Block, Blockstore};
use fvm_ipld_encoding::CborStore;
use fvm_shared::IPLD_RAW;
use fvm_shared::state::ActorState;
use multihash::Code;
use fendermint_actor_blobs_v2_shared::BLOBS_ACTOR_ID;
use fendermint_actor_blobs_v2_shared::state::TokenCreditRate;
use fendermint_actor_recall_config::State as RecallConfigState;
use fendermint_actor_recall_config_v2::{State as RecallConfigV2State};
use fendermint_actor_recall_config_v2_shared::RecallConfig;
use fendermint_vm_actor_interface::recall_config::RECALL_CONFIG_ACTOR_ID;
use fendermint_vm_interpreter::fvm::state::FvmExecState;

static RECALL_CONFIG_V2_WASM_BIN: &[u8] = include_bytes!("fendermint_actor_recall_config_v2.wasm");
static BLOBS_V2_WASM_BIN: &[u8] = include_bytes!("fendermint_actor_blobs_v2.wasm");

pub fn upgrade<BS: Blockstore + Clone + Sized>(
    state: &mut FvmExecState<BS>,
) -> anyhow::Result<()> {
    let state_tree = state.state_tree_mut();

    // get the Recall Config ActorState from the state tree
    let actor_state = match state_tree.get_actor(RECALL_CONFIG_ACTOR_ID)? {
        Some(actor) => actor,
        None => {
            return Err(anyhow!("recall config actor not found"));
        }
    };
    tracing::info!(
        "recall config code_cid: {:?}, state_cid: {:?}",
        actor_state.code, actor_state.state
    );

    let recall_config_state: RecallConfigState = match state_tree.store().get_cbor(&actor_state.state)? {
        Some(v) => v,
        None => return Err(anyhow!("recall config actor state not found")),
    };

    let recall_config = recall_config_state.config;
    let recall_config_v2_state = RecallConfigV2State  {
        admin: recall_config_state.admin,
        config: RecallConfig {
            blob_capacity: recall_config.blob_capacity,
            token_credit_rate: TokenCreditRate::from(recall_config.token_credit_rate.rate().clone()),
            blob_credit_debit_interval: recall_config.blob_credit_debit_interval,
            blob_min_ttl: recall_config.blob_min_ttl,
            blob_default_ttl: recall_config.blob_default_ttl,
            blob_delete_batch_size: recall_config.blob_delete_batch_size,
            account_debit_batch_size: recall_config.account_debit_batch_size,
            new_config: 780,
        },
    };

    let new_state_cid = state_tree
        .store()
        .put_cbor(&recall_config_v2_state, Code::Blake2b256)
        .map_err(|e| anyhow!("failed to put recall config v2 actor state: {}", e))?;


    // store the new wasm code in the blockstore and get the new code cid
    let new_code_cid = state_tree.store().put(
        Code::Blake2b256,
        &Block {
            codec: IPLD_RAW,
            data: RECALL_CONFIG_V2_WASM_BIN,
        },
    )?;

    // next we update the actor state in the state tree
    state_tree.set_actor(
        RECALL_CONFIG_ACTOR_ID,
        ActorState {
            code: new_code_cid,
            state: new_state_cid,
            sequence: actor_state.sequence,
            balance: actor_state.balance,
            delegated_address: actor_state.delegated_address,
        },
    );

    // get the Blobs ActorState from the state tree
    let actor_state = match state_tree.get_actor(BLOBS_ACTOR_ID)? {
        Some(actor) => actor,
        None => {
            return Err(anyhow!("blobs actor not found"));
        }
    };
    tracing::info!(
        "blobs code_cid: {:?}, state_cid: {:?}",
        actor_state.code, actor_state.state
    );

    // store the new wasm code in the blockstore and get the new code cid
    let new_code_cid = state_tree.store().put(
        Code::Blake2b256,
        &Block {
            codec: IPLD_RAW,
            data: BLOBS_V2_WASM_BIN,
        },
    )?;

    // next we update the actor state in the state tree
    state_tree.set_actor(
        BLOBS_ACTOR_ID,
        ActorState {
            code: new_code_cid,
            state: actor_state.state,
            sequence: actor_state.sequence,
            balance: actor_state.balance,
            delegated_address: actor_state.delegated_address,
        },
    );

    tracing::info!(
        "update executed",
    );

    return Ok(());
}
