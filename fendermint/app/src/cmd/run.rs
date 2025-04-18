// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::sync::Arc;

use anyhow::{anyhow, bail, Context};
use async_stm::atomically_or_err;
use fendermint_abci::ApplicationService;
use fendermint_app::ipc::{AppParentFinalityQuery, AppVote};
use fendermint_app::{App, AppConfig, AppStore, BitswapBlockstore};
use fendermint_app_settings::AccountKind;
use fendermint_crypto::SecretKey;
use fendermint_rocksdb::{blockstore::NamespaceBlockstore, namespaces, RocksDb, RocksDbConfig};
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_interpreter::chain::ChainEnv;
use fendermint_vm_interpreter::fvm::observe::register_metrics as register_interpreter_metrics;
use fendermint_vm_interpreter::fvm::upgrades::UpgradeScheduler;
use fendermint_vm_interpreter::{
    bytes::{BytesMessageInterpreter, ProposalPrepareMode},
    chain::{BlobPool, ChainMessageInterpreter, CheckpointPool, ReadRequestPool},
    fvm::{Broadcaster, FvmMessageInterpreter, ValidatorContext},
    signed::SignedMessageInterpreter,
};
use fendermint_vm_iroh_resolver::iroh::IrohResolver;
use fendermint_vm_resolver::ipld::IpldResolver;
use fendermint_vm_snapshot::{SnapshotManager, SnapshotParams};
use fendermint_vm_topdown::observe::register_metrics as register_topdown_metrics;
use fendermint_vm_topdown::proxy::{IPCProviderProxy, IPCProviderProxyWithLatency};
use fendermint_vm_topdown::sync::launch_polling_syncer;
use fendermint_vm_topdown::voting::{publish_vote_loop, Error as VoteError, VoteTally};
use fendermint_vm_topdown::{
    CachedFinalityProvider, IPCBlobFinality, IPCParentFinality, IPCReadRequestClosed, Toggle,
};
use fvm_shared::address::{current_network, Address, Network};
use ipc_ipld_resolver::{Event as ResolverEvent, IrohConfig, VoteRecord};
use ipc_observability::{emit, observe::register_metrics as register_default_metrics};
use ipc_provider::config::subnet::{EVMSubnet, SubnetConfig};
use ipc_provider::IpcProvider;
use tokio::sync::broadcast::error::RecvError;
use tower::ServiceBuilder;
use tracing::{debug, error, info, warn};

use crate::cmd::key::read_secret_key;
use crate::{cmd, options::run::RunArgs, settings::Settings};
use fendermint_app::observe::register_metrics as register_consensus_metrics;
use fendermint_vm_iroh_resolver::observe::{
    register_metrics as register_blobs_metrics, BlobsFinalityVotingFailure,
    BlobsFinalityVotingSuccess, ReadRequestsCloseVoting,
};

cmd! {
  RunArgs(self, settings) {
      run(settings, IrohConfig {
          path: self.iroh_path.clone(),
          rpc_addr: self.iroh_rpc_addr,
          v4_addr: self.iroh_v4_addr,
          v6_addr: self.iroh_v6_addr,
      }).await
  }
}

// Database collection names.
namespaces! {
    Namespaces {
        app,
        state_hist,
        state_store,
        bit_store
    }
}

/// Run the Fendermint ABCI Application.
///
/// This method acts as our composition root.
async fn run(settings: Settings, iroh_config: IrohConfig) -> anyhow::Result<()> {
    let tendermint_rpc_url = settings.tendermint_rpc_url()?;
    info!("Connecting to Tendermint at {tendermint_rpc_url}");

    let tendermint_client: tendermint_rpc::HttpClient =
        tendermint_rpc::HttpClient::new(tendermint_rpc_url)
            .context("failed to create Tendermint client")?;

    // Prometheus metrics
    let metrics_registry = if settings.metrics.enabled {
        let registry = prometheus::Registry::new_custom(
            Some("ipc".to_string()),
            Some([("subnet_id".to_string(), settings.ipc.subnet_id.to_string())].into()),
        )
        .context("failed to create Prometheus registry")?;

        register_default_metrics(&registry).context("failed to register default metrics")?;
        register_topdown_metrics(&registry).context("failed to register topdown metrics")?;
        register_interpreter_metrics(&registry)
            .context("failed to register interpreter metrics")?;
        register_consensus_metrics(&registry).context("failed to register consensus metrics")?;
        register_blobs_metrics(&registry).context("failed to register blobs metrics")?;

        Some(registry)
    } else {
        None
    };

    let validator = match settings.validator_key {
        Some(ref key) => {
            let sk = key.path(settings.home_dir());
            if sk.exists() && sk.is_file() {
                let sk = read_secret_key(&sk).context("failed to read validator key")?;
                let addr = to_address(&sk, &key.kind)?;
                info!("validator key address: {addr} detected");
                Some((sk, addr))
            } else {
                bail!("validator key does not exist: {}", sk.to_string_lossy());
            }
        }
        None => {
            info!("validator key not configured");
            None
        }
    };

    let validator_keypair = validator.as_ref().map(|(sk, _)| {
        let mut bz = sk.serialize();
        let sk = libp2p::identity::secp256k1::SecretKey::try_from_bytes(&mut bz)
            .expect("secp256k1 secret key");
        let kp = libp2p::identity::secp256k1::Keypair::from(sk);
        libp2p::identity::Keypair::from(kp)
    });

    let validator_ctx = validator.map(|(sk, addr)| {
        // For now we are using the validator key for submitting transactions.
        // This allows us to identify transactions coming from empowered validators, to give priority to protocol related transactions.
        let broadcaster = Broadcaster::new(
            tendermint_client.clone(),
            addr,
            sk.clone(),
            settings.fvm.gas_fee_cap.clone(),
            settings.fvm.gas_premium.clone(),
            settings.fvm.gas_overestimation_rate,
        )
        .with_max_retries(settings.broadcast.max_retries)
        .with_retry_delay(settings.broadcast.retry_delay);

        ValidatorContext::new(sk, addr, broadcaster)
    });

    let testing_settings = match settings.testing.as_ref() {
        Some(_) if current_network() == Network::Mainnet => {
            bail!("testing settings are not allowed on Mainnet");
        }
        other => other,
    };

    let interpreter = FvmMessageInterpreter::<NamespaceBlockstore, _>::new(
        tendermint_client.clone(),
        validator_ctx,
        settings.fvm.gas_overestimation_rate,
        settings.fvm.gas_search_step,
        settings.fvm.exec_in_check,
        UpgradeScheduler::new(),
    )
    .with_push_chain_meta(testing_settings.map_or(true, |t| t.push_chain_meta));

    let interpreter = SignedMessageInterpreter::new(interpreter);
    let interpreter = ChainMessageInterpreter::<_, NamespaceBlockstore>::new(interpreter);
    let interpreter = BytesMessageInterpreter::new(
        interpreter,
        ProposalPrepareMode::PrependOnly,
        false,
        settings.abci.block_max_msgs,
    );

    let ns = Namespaces::default();
    let db = open_db(&settings, &ns).context("error opening DB")?;

    // Blockstore for actors.
    let state_store =
        NamespaceBlockstore::new(db.clone(), ns.state_store).context("error creating state DB")?;

    let checkpoint_pool = CheckpointPool::new();
    let blob_pool = BlobPool::new();
    let read_request_pool = ReadRequestPool::new();
    let parent_finality_votes = VoteTally::empty();

    let topdown_enabled = settings.topdown_enabled();

    // If enabled, start a resolver that communicates with the application through the resolve pool.
    if settings.resolver_enabled() {
        let mut service = make_resolver_service(
            &settings,
            db.clone(),
            state_store.clone(),
            ns.bit_store,
            iroh_config,
        )
        .await?;

        // set iroh RPC config for SYSCALL
        let rpc_addr = service.iroh().rpc_addr().to_string();
        std::env::set_var("IROH_SYSCALL_RPC_ADDR", rpc_addr);

        // Register all metrics from the IPLD resolver stack
        if let Some(ref registry) = metrics_registry {
            service
                .register_metrics(registry)
                .context("failed to register IPLD resolver metrics")?;
        }

        let client = service.client();

        let own_subnet_id = settings.ipc.subnet_id.clone();

        client
            .add_provided_subnet(own_subnet_id.clone())
            .context("error adding own provided subnet.")?;

        let resolver = IpldResolver::new(
            client.clone(),
            checkpoint_pool.queue(),
            settings.resolver.retry_delay,
            own_subnet_id.clone(),
        );

        if topdown_enabled {
            if let Some(key) = validator_keypair.clone() {
                let parent_finality_votes = parent_finality_votes.clone();
                let own_subnet_id = own_subnet_id.clone();

                info!("starting the parent finality vote gossip loop...");
                let client = client.clone();
                tokio::spawn(async move {
                    publish_vote_loop(
                        parent_finality_votes,
                        settings.ipc.vote_interval,
                        settings.ipc.vote_timeout,
                        key,
                        own_subnet_id,
                        client,
                        |height, block_hash| {
                            AppVote::ParentFinality(IPCParentFinality { height, block_hash })
                        },
                    )
                    .await
                });
            } else {
                info!("parent finality vote gossip disabled");
            }
        } else {
            info!("parent finality vote gossip disabled");
        }

        if let Some(key) = validator_keypair {
            // Blob resolver
            let iroh_resolver = IrohResolver::new(
                client.clone(),
                blob_pool.queue(),
                settings.resolver.retry_delay,
                parent_finality_votes.clone(),
                key.clone(),
                own_subnet_id.clone(),
                |hash, success| AppVote::BlobFinality(IPCBlobFinality::new(hash, success)),
                blob_pool.results(),
            );

            info!("starting the iroh Resolver...");
            tokio::spawn(async move { iroh_resolver.run().await });

            // Read request resolver
            let read_request_resolver = IrohResolver::new(
                client.clone(),
                read_request_pool.queue(),
                settings.resolver.retry_delay,
                parent_finality_votes.clone(),
                key,
                own_subnet_id,
                |hash, _| AppVote::ReadRequestClosed(IPCReadRequestClosed::new(hash)),
                read_request_pool.results(),
            );

            info!("starting the read request resolver...");
            tokio::spawn(async move { read_request_resolver.run().await });
        } else {
            info!("iroh resolver disabled.");
            info!("read request resolver disabled.");
        }

        info!("subscribing to gossip...");
        let rx = service.subscribe();
        let parent_finality_votes = parent_finality_votes.clone();
        tokio::spawn(async move {
            dispatch_resolver_events(rx, parent_finality_votes, topdown_enabled).await;
        });

        info!("starting the IPLD Resolver Service...");
        tokio::spawn(async move {
            if let Err(e) = service.run().await {
                error!("IPLD Resolver Service failed: {e:#}")
            }
        });

        info!("starting the IPLD Resolver...");
        tokio::spawn(async move { resolver.run().await });
    } else {
        info!("IPLD Resolver disabled.")
    }

    let (parent_finality_provider, ipc_tuple) = if topdown_enabled {
        info!("topdown finality enabled");
        let topdown_config = settings.ipc.topdown_config()?;
        let mut config = fendermint_vm_topdown::Config::new(
            topdown_config.chain_head_delay,
            topdown_config.polling_interval,
            topdown_config.exponential_back_off,
            topdown_config.exponential_retry_limit,
        )
        .with_proposal_delay(topdown_config.proposal_delay)
        .with_max_proposal_range(topdown_config.max_proposal_range);

        if let Some(v) = topdown_config.max_cache_blocks {
            info!(value = v, "setting max cache blocks");
            config = config.with_max_cache_blocks(v);
        }

        let ipc_provider = {
            let p = make_ipc_provider_proxy(&settings)?;
            Arc::new(IPCProviderProxyWithLatency::new(p))
        };

        let finality_provider =
            CachedFinalityProvider::uninitialized(config.clone(), ipc_provider.clone()).await?;

        let p = Arc::new(Toggle::enabled(finality_provider));
        (p, Some((ipc_provider, config)))
    } else {
        info!("topdown finality disabled");
        (Arc::new(Toggle::disabled()), None)
    };

    // Start a snapshot manager in the background.
    let snapshots = if settings.snapshots.enabled {
        let (manager, client) = SnapshotManager::new(
            state_store.clone(),
            SnapshotParams {
                snapshots_dir: settings.snapshots_dir(),
                download_dir: settings.snapshots.download_dir(),
                block_interval: settings.snapshots.block_interval,
                chunk_size: settings.snapshots.chunk_size_bytes,
                hist_size: settings.snapshots.hist_size,
                last_access_hold: settings.snapshots.last_access_hold,
                sync_poll_interval: settings.snapshots.sync_poll_interval,
            },
        )
        .context("failed to create snapshot manager")?;

        info!("starting the SnapshotManager...");
        let tendermint_client = tendermint_client.clone();
        tokio::spawn(async move { manager.run(tendermint_client).await });

        Some(client)
    } else {
        info!("snapshots disabled");
        None
    };

    let app: App<_, _, AppStore, _> = App::new(
        AppConfig {
            app_namespace: ns.app,
            state_hist_namespace: ns.state_hist,
            state_hist_size: settings.db.state_hist_size,
            halt_height: settings.halt_height,
        },
        db,
        state_store,
        interpreter,
        ChainEnv {
            checkpoint_pool,
            parent_finality_provider: parent_finality_provider.clone(),
            parent_finality_votes: parent_finality_votes.clone(),
            blob_pool,
            blob_concurrency: settings.blob_concurrency,
            read_request_pool,
            read_request_concurrency: settings.read_request_concurrency,
            blob_metrics_interval: settings.blob_metrics_interval,
            blob_queue_gas_limit: settings.blob_queue_gas_limit,
        },
        snapshots,
    )?;

    if let Some((agent_proxy, config)) = ipc_tuple {
        let app_parent_finality_query = AppParentFinalityQuery::new(app.clone());
        tokio::spawn(async move {
            match launch_polling_syncer(
                app_parent_finality_query,
                config,
                parent_finality_provider,
                parent_finality_votes,
                agent_proxy,
                tendermint_client,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => error!("cannot launch polling syncer: {e}"),
            }
        });
    }

    // Start the metrics on a background thread.
    if let Some(registry) = metrics_registry {
        info!(
            listen_addr = settings.metrics.listen.to_string(),
            "serving metrics"
        );
        let mut builder = prometheus_exporter::Builder::new(settings.metrics.listen.try_into()?);
        builder.with_registry(registry);
        let _ = builder.start().context("failed to start metrics server")?;
    } else {
        info!("metrics disabled");
    }

    let service = ApplicationService(app);

    // Split it into components.
    let (consensus, mempool, snapshot, info) =
        tower_abci::split::service(service, settings.abci.bound);

    // Hand those components to the ABCI server. This is where tower layers could be added.
    // TODO: Check out the examples about load shedding in `info` requests.
    let server = tower_abci::v037::Server::builder()
        .consensus(
            // Limiting the concurrency to 1 here because the `AplicationService::poll_ready` always
            // reports `Ready`, because it doesn't know which request it's going to get.
            // Not limiting the concurrency to 1 can lead to transactions being applied
            // in different order across nodes. The buffer size has to be large enough
            // to allow all in-flight requests to not block message handling in
            // `tower_abci::Connection::run`, which could lead to deadlocks.
            // With ABCI++ we need to be able to handle all block transactions plus the begin/end/commit
            // around it. With ABCI 2.0 we'll get the block as a whole, which makes this easier.
            ServiceBuilder::new()
                .buffer(settings.abci.block_max_msgs + 3)
                .concurrency_limit(1)
                .service(consensus),
        )
        .snapshot(snapshot)
        .mempool(mempool)
        .info(info)
        .finish()
        .context("error creating ABCI server")?;

    // Run the ABCI server.
    server
        .listen(settings.abci.listen.to_string())
        .await
        .map_err(|e| anyhow!("error listening: {e}"))?;

    Ok(())
}

/// Open database with all
fn open_db(settings: &Settings, ns: &Namespaces) -> anyhow::Result<RocksDb> {
    let path = settings.data_dir().join("rocksdb");
    info!(
        path = path.to_string_lossy().into_owned(),
        "opening database"
    );
    let config = RocksDbConfig {
        compaction_style: settings.db.compaction_style.to_string(),
        ..Default::default()
    };
    let db = RocksDb::open_cf(path, &config, ns.values().iter())?;
    Ok(db)
}

async fn make_resolver_service(
    settings: &Settings,
    db: RocksDb,
    state_store: NamespaceBlockstore,
    bit_store_ns: String,
    iroh_config: IrohConfig,
) -> anyhow::Result<ipc_ipld_resolver::Service<libipld::DefaultParams, AppVote>> {
    // Blockstore for Bitswap.
    let bit_store = NamespaceBlockstore::new(db, bit_store_ns).context("error creating bit DB")?;

    // Blockstore for Bitswap with a fallback on the actor store for reads.
    let bitswap_store = BitswapBlockstore::new(state_store, bit_store);

    let config =
        to_resolver_config(settings, iroh_config).context("error creating resolver config")?;

    let service = ipc_ipld_resolver::Service::new(config, bitswap_store)
        .await
        .context("error creating IPLD Resolver Service")?;

    Ok(service)
}

fn make_ipc_provider_proxy(settings: &Settings) -> anyhow::Result<IPCProviderProxy> {
    let topdown_config = settings.ipc.topdown_config()?;
    let subnet = ipc_provider::config::Subnet {
        id: settings
            .ipc
            .subnet_id
            .parent()
            .ok_or_else(|| anyhow!("subnet has no parent"))?,
        config: SubnetConfig::Fevm(EVMSubnet {
            provider_http: topdown_config.parent_http_endpoint.to_string().parse()?,
            provider_timeout: topdown_config.parent_http_timeout,
            auth_token: topdown_config.parent_http_auth_token.as_ref().cloned(),
            registry_addr: topdown_config.parent_registry,
            gateway_addr: topdown_config.parent_gateway,
        }),
    };
    info!("init ipc provider with subnet: {}", subnet.id);

    let ipc_provider = IpcProvider::new_with_subnet(None, subnet)?;
    IPCProviderProxy::new(ipc_provider, settings.ipc.subnet_id.clone())
}

fn to_resolver_config(
    settings: &Settings,
    iroh_config: IrohConfig,
) -> anyhow::Result<ipc_ipld_resolver::Config> {
    use ipc_ipld_resolver::{
        Config, ConnectionConfig, ContentConfig, DiscoveryConfig, MembershipConfig, NetworkConfig,
    };

    let r = &settings.resolver;

    let local_key: libp2p::identity::Keypair = {
        let path = r.network.local_key(settings.home_dir());
        let sk = read_secret_key(&path)?;
        let sk = libp2p::identity::secp256k1::SecretKey::try_from_bytes(sk.serialize())?;
        libp2p::identity::secp256k1::Keypair::from(sk).into()
    };

    let network_name = format!(
        "ipld-resolver-{}-{}",
        settings.ipc.subnet_id.root_id(),
        r.network.network_name
    );

    let config = Config {
        connection: ConnectionConfig {
            listen_addr: r.connection.listen_addr.clone(),
            external_addresses: r.connection.external_addresses.clone(),
            expected_peer_count: r.connection.expected_peer_count,
            max_incoming: r.connection.max_incoming,
            max_peers_per_query: r.connection.max_peers_per_query,
            event_buffer_capacity: r.connection.event_buffer_capacity,
        },
        network: NetworkConfig {
            local_key,
            network_name,
        },
        discovery: DiscoveryConfig {
            static_addresses: r.discovery.static_addresses.clone(),
            target_connections: r.discovery.target_connections,
            enable_kademlia: r.discovery.enable_kademlia,
        },
        membership: MembershipConfig {
            static_subnets: r.membership.static_subnets.clone(),
            max_subnets: r.membership.max_subnets,
            publish_interval: r.membership.publish_interval,
            min_time_between_publish: r.membership.min_time_between_publish,
            max_provider_age: r.membership.max_provider_age,
        },
        content: ContentConfig {
            rate_limit_bytes: r.content.rate_limit_bytes,
            rate_limit_period: r.content.rate_limit_period,
        },
        iroh: iroh_config,
    };

    Ok(config)
}

fn to_address(sk: &SecretKey, kind: &AccountKind) -> anyhow::Result<Address> {
    let pk = sk.public_key().serialize();
    match kind {
        AccountKind::Regular => Ok(Address::new_secp256k1(&pk)?),
        AccountKind::Ethereum => Ok(Address::from(EthAddress::new_secp256k1(&pk)?)),
    }
}

async fn dispatch_resolver_events(
    mut rx: tokio::sync::broadcast::Receiver<ResolverEvent<AppVote>>,
    parent_finality_votes: VoteTally,
    topdown_enabled: bool,
) {
    loop {
        match rx.recv().await {
            Ok(event) => match event {
                ResolverEvent::ReceivedPreemptive(_, _) => {}
                ResolverEvent::ReceivedVote(vote) => {
                    dispatch_vote(*vote, &parent_finality_votes, topdown_enabled).await;
                }
            },
            Err(RecvError::Lagged(n)) => {
                warn!("the resolver service skipped {n} gossip events")
            }
            Err(RecvError::Closed) => {
                error!("the resolver service stopped receiving gossip");
                return;
            }
        }
    }
}

async fn dispatch_vote(
    vote: VoteRecord<AppVote>,
    parent_finality_votes: &VoteTally,
    topdown_enabled: bool,
) {
    match vote.content {
        AppVote::ParentFinality(f) => {
            if !topdown_enabled {
                debug!("ignoring vote; topdown disabled");
                return;
            }
            match atomically_or_err(|| {
                parent_finality_votes.add_vote(
                    vote.public_key.clone(),
                    f.height,
                    f.block_hash.clone(),
                )
            })
            .await
            {
                Ok(_) => {
                    debug!("vote handled for parent finality");
                }
                Err(e @ VoteError::Equivocation(_, _, _, _)) => {
                    warn!(
                        error = e.to_string(),
                        "failed to handle parent finality vote"
                    );
                }
                Err(
                    e @ (VoteError::Uninitialized // early vote, we're not ready yet
                    | VoteError::UnpoweredValidator(_) // maybe arrived too early or too late, or spam
                    | VoteError::UnexpectedBlock(_, _)), // won't happen here
                ) => {
                    debug!(
                        error = e.to_string(),
                        "failed to handle parent finality vote"
                    );
                }
            }
        }
        AppVote::BlobFinality(f) => {
            debug!(hash = %f.hash, success = ?f.success, "received vote for blob finality");
            match atomically_or_err(|| {
                parent_finality_votes.add_blob_vote(
                    vote.public_key.clone(),
                    f.hash.as_bytes().to_vec(),
                    f.success,
                )
            })
            .await
            {
                Ok(_) => {
                    debug!("vote handled for blob finality");
                    if f.success {
                        emit(BlobsFinalityVotingSuccess {
                            blob_hash: Some(f.hash.to_string()),
                        });
                    } else {
                        emit(BlobsFinalityVotingFailure {
                            blob_hash: Some(f.hash.to_string()),
                        });
                    }
                }
                Err(e @ VoteError::Equivocation(_, _, _, _)) => {
                    warn!(error = e.to_string(), "failed to handle blob finality vote");
                }
                Err(
                    e @ (VoteError::Uninitialized // early vote, we're not ready yet
                    | VoteError::UnpoweredValidator(_) // maybe arrived too early or too late, or spam
                    | VoteError::UnexpectedBlock(_, _)), // won't happen here
                ) => {
                    debug!(error = e.to_string(), "failed to handle blob finality vote");
                }
            }
        }
        AppVote::ReadRequestClosed(r) => {
            debug!(hash = %r.hash, "received vote for read request completion");
            match atomically_or_err(|| {
                parent_finality_votes.add_blob_vote(
                    vote.public_key.clone(),
                    r.hash.as_bytes().to_vec(),
                    true,
                )
            })
            .await
            {
                Ok(_) => {
                    debug!("vote handled for read request completion");
                    emit(ReadRequestsCloseVoting {
                        read_request_id: Some(r.hash.to_string()),
                    });
                }
                Err(e @ VoteError::Equivocation(_, _, _, _)) => {
                    warn!(
                        error = e.to_string(),
                        "failed to handle read request completion vote"
                    );
                }
                Err(
                    e @ (VoteError::Uninitialized // early vote, we're not ready yet
                    | VoteError::UnpoweredValidator(_) // maybe arrived too early or too late, or spam
                    | VoteError::UnexpectedBlock(_, _)), // won't happen here
                ) => {
                    debug!(
                        error = e.to_string(),
                        "failed to handle read request completion vote"
                    );
                }
            }
        }
    }
}
