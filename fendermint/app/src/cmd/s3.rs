// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_app_options::s3::{S3Args, S3Commands};
use fendermint_app_settings::s3::S3Settings;

use crate::cmd;
use adm_provider::json_rpc::JsonRpcProvider;
use adm_signer::Wallet;
use basin_s3::Basin;
use homedir::my_home;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as ConnBuilder;
use s3s::auth::SimpleAuth;
use s3s::service::S3ServiceBuilder;
use tendermint_rpc::HttpClient;
use tokio::net::TcpListener;

cmd! {
    S3Args(self, settings: S3Settings) {
        match self.command.clone() {
            S3Commands::Run { access_key, secret_key, network } => {
                    network.get().init();

                    let network = network.get();
                    // Setup network provider
                    let provider =
                        JsonRpcProvider::new_http(network.rpc_url()?, None, Some(network.object_api_url()?))?;

                    let root = my_home()?.unwrap().join(".s3-basin");
                    std::fs::create_dir_all(&root)?;

                    let basin: Basin<HttpClient, Wallet> = Basin::new(root, provider, None)?;

                    // Setup S3 service
                    let service = {
                        let mut b = S3ServiceBuilder::new(basin);

                        // Enable authentication
                        if let (Some(ak), Some(sk)) = (access_key, secret_key) {
                            b.set_auth(SimpleAuth::from_single(ak, sk));
                            tracing::info!("authentication is enabled");
                        }

                        b.build()
                    };

                    // Run server
                    let listener = TcpListener::bind((settings.listen.host, settings.listen.port)).await?;
                    let local_addr = listener.local_addr()?;

                    let hyper_service = service.into_shared();

                    let http_server = ConnBuilder::new(TokioExecutor::new());
                    let graceful = hyper_util::server::graceful::GracefulShutdown::new();

                    let mut ctrl_c = std::pin::pin!(tokio::signal::ctrl_c());

                    tracing::info!("server is running at http://{local_addr}");

                     loop {
                        let (socket, _) = tokio::select! {
                            res =  listener.accept() => {
                                match res {
                                    Ok(conn) => conn,
                                    Err(err) => {
                                        tracing::error!("error accepting connection: {err}");
                                        continue;
                                    }
                                }
                            }
                            _ = ctrl_c.as_mut() => {
                                break;
                            }
                        };

                        let conn = http_server.serve_connection(TokioIo::new(socket), hyper_service.clone());
                        let conn = graceful.watch(conn.into_owned());
                        tokio::spawn(async move {
                            let _ = conn.await;
                        });
                    }

                    tokio::select! {
                        () = graceful.shutdown() => {
                             tracing::debug!("Gracefully shutdown!");
                        },
                        () = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
                             tracing::debug!("Waited 10 seconds for graceful shutdown, aborting...");
                        }
                    }

                    tracing::info!("server is stopped");
                    Ok(())
            },
        }
    }
}
