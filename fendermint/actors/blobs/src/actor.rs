// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::Hash;
use fendermint_actor_blobs_shared::Method;
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorError, FIRST_EXPORTED_METHOD_NUMBER, SYSTEM_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::MethodNum;

use crate::{State, BLOBS_ACTOR_NAME};

mod admin;
mod metrics;
mod system;
mod user;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(BlobsActor);

/// Singleton actor for managing blob storage.
///
/// The [`Address`]es stored in this actor's state _must_ be ID-based addresses for
/// efficient comparison with message origin and caller addresses, which are always ID-based.
/// [`Address`]es in the method params can be of any type.
/// They will be resolved to ID-based addresses.
///
/// For simplicity, this actor currently manages both blobs and credit.
/// A future version of the protocol will likely separate them in some way.
pub struct BlobsActor;

impl BlobsActor {
    /// Creates a new [`BlobsActor`] state.
    ///
    /// This is only used in tests. This actor is created manually at genesis.
    fn constructor(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let state = State::new(rt.store())?;
        rt.create(&state)
    }

    /// Fallback method for unimplemented method numbers.
    fn fallback(
        rt: &impl Runtime,
        method: MethodNum,
        _: Option<IpldBlock>,
    ) -> Result<Option<IpldBlock>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        if method >= FIRST_EXPORTED_METHOD_NUMBER {
            Ok(None)
        } else {
            Err(actor_error!(unhandled_message; "invalid method: {}", method))
        }
    }
}

impl ActorCode for BlobsActor {
    type Methods = Method;

    fn name() -> &'static str {
        BLOBS_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,

        // User methods
        BuyCredit => buy_credit,
        ApproveCredit => approve_credit,
        RevokeCredit => revoke_credit,
        SetAccountSponsor => set_account_sponsor,
        GetAccount => get_account,
        GetCreditApproval => get_credit_approval,
        AddBlob => add_blob,
        GetBlob => get_blob,
        DeleteBlob => delete_blob,
        OverwriteBlob => overwrite_blob,

        // System methods
        GetGasAllowance => get_gas_allowance,
        UpdateGasAllowance => update_gas_allowance,
        GetBlobStatus => get_blob_status,
        GetAddedBlobs => get_added_blobs,
        GetPendingBlobs => get_pending_blobs,
        SetBlobPending => set_blob_pending,
        FinalizeBlob => finalize_blob,
        DebitAccounts => debit_accounts,

        // Admin methods
        SetAccountStatus => set_account_status,
        TrimBlobExpiries => trim_blob_expiries,

        // Metrics methods
        GetStats => get_stats,
        _ => fallback,
    }
}

/// Makes a syscall that will delete a blob from the underlying Iroh-based data store.
fn delete_from_disc(hash: Hash) -> Result<(), ActorError> {
    #[cfg(feature = "fil-actor")]
    {
        recall_actor_sdk::hash_rm(hash.0).map_err(|en| {
            ActorError::unspecified(format!("failed to delete blob from disc: {:?}", en))
        })?;
        log::debug!("deleted blob {} from disc", hash);
        Ok(())
    }
    #[cfg(not(feature = "fil-actor"))]
    {
        log::debug!("mock deletion from disc (hash={})", hash);
        Ok(())
    }
}
