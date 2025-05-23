// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::io::Write;
use std::path::{Path, PathBuf};

/// Generate Rust bindings from the IPC Solidity Actors ABI artifacts.
///
/// These are built by `make ipc-actors-abi`, here we just add the final step
/// so we have better code completion with Rust Analyzer.
fn main() {
    // Run with `cargo build -vv` to see output from any `eprintln!` or `println!`.

    // Maybe we want to skip the build and use the files as-is, could be imported as crate.
    // Enabled by default so that in the monorepo we don't have to worry about stale code.
    if std::env::var("BUILD_BINDINGS").unwrap_or("0".to_string()) == "0" {
        return;
    }

    // Where are the Solidity artifacts.
    let output_dir = std::env::var("OUTPUT").unwrap_or("out".to_string());

    let ipc_actors_dir = workspace_dir()
        .join("contracts")
        .to_string_lossy()
        .into_owned();

    let lib_path = format!("{ipc_actors_dir}/binding/src/lib.rs");
    let mut lib = std::fs::File::create(&lib_path)
        .unwrap_or_else(|e| panic!("failed to create {lib_path}: {e}"));

    writeln!(lib, "// DO NOT EDIT! This file was generated by build.rs").unwrap();
    writeln!(lib, "#[macro_use]\nmod convert;").unwrap();

    // The list of actors we need bindings for, based on how the ipc-actor uses `abigen!`.
    // With the diamond pattern, there is a contract that holds state, and there are these facets which have the code,
    // so we need bindings for the facets, but well (I think) use the same address with all of them.
    for contract_name in [
        "IDiamond",
        "DiamondLoupeFacet",
        "DiamondCutFacet",
        "OwnershipFacet",
        "GatewayDiamond",
        "GatewayManagerFacet",
        "GatewayGetterFacet",
        "CheckpointingFacet",
        "TopDownFinalityFacet",
        "XnetMessagingFacet",
        "GatewayMessengerFacet",
        "SubnetActorActivityFacet",
        "SubnetActorCheckpointingFacet",
        "SubnetActorDiamond",
        "SubnetActorGetterFacet",
        "SubnetActorManagerFacet",
        "SubnetActorPauseFacet",
        "SubnetActorRewardFacet",
        "SubnetRegistryDiamond",
        "RegisterSubnetFacet",
        "SubnetGetterFacet",
        "LibStaking",
        "LibStakingChangeLog",
        "LibGateway",
        "LibQuorum",
    ] {
        let module_name = camel_to_snake(contract_name);
        let input_path =
            format!("{ipc_actors_dir}/{output_dir}/{contract_name}.sol/{contract_name}.json");
        let output_path = format!("{ipc_actors_dir}/binding/src/{}.rs", module_name);

        ethers::prelude::Abigen::new(contract_name, &input_path)
            .expect("failed to create Abigen")
            .generate()
            .expect("failed to generate Rust bindings")
            .write_to_file(output_path)
            .expect("failed to write Rust code");

        writeln!(lib, "#[allow(clippy::all)]\npub mod {module_name};").unwrap();

        println!("cargo:rerun-if-changed={input_path}");
    }

    writeln!(
        lib,
        "\n// The list of contracts need to convert FvmAddress to fvm_shared::Address"
    )
    .unwrap();
    let fvm_address_conversion = vec![
        "GatewayManagerFacet",
        "GatewayGetterFacet",
        "XnetMessagingFacet",
        "GatewayMessengerFacet",
        "SubnetActorCheckpointingFacet",
        "SubnetActorGetterFacet",
        "LibGateway",
        "CheckpointingFacet",
    ];

    let modules = fvm_address_conversion.into_iter().map(camel_to_snake);
    for module in modules {
        writeln!(lib, "fvm_address_conversion!({module});").unwrap();
    }

    writeln!(
        lib,
        "\n// The list of contracts that need to convert common types between each other"
    )
    .unwrap();
    let common_type_conversion = vec![
        ("SubnetActorGetterFacet", "CheckpointingFacet"),
        ("SubnetActorGetterFacet", "XnetMessagingFacet"),
    ];
    for (contract1, contract2) in common_type_conversion {
        writeln!(
            lib,
            "common_type_conversion!({}, {});",
            camel_to_snake(contract1),
            camel_to_snake(contract2)
        )
        .unwrap();
    }

    println!("cargo:rerun-if-changed=../out");
    println!("cargo:rerun-if-changed=build.rs");

    // Run rustfmt on binding/src/lib.rs to make sure we don't accidentally format it in our IDEs
    //
    // sync the binding/src/lib.rs file to disk
    lib.sync_all().unwrap();
    // then run rustfmt on the file (it should be available as its specifed in our toolchain
    let mut proc = std::process::Command::new("rustfmt")
        .arg(lib_path)
        .spawn()
        .expect("rustfmt failed to start");
    let ecode = proc.wait().expect("rustfmt failed to run");
    assert!(ecode.success());
}

/// Convert ContractName to contract_name so we can use it as a Rust module.
///
/// We could just lowercase, but this is what `Abigen` does as well, and it's more readable with complex names.
fn camel_to_snake(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        match (i, c) {
            (0, c) if c.is_uppercase() => {
                out.push(c.to_ascii_lowercase());
            }
            (_, c) if c.is_uppercase() => {
                out.push('_');
                out.push(c.to_ascii_lowercase());
            }
            (_, c) => {
                out.push(c);
            }
        }
    }
    out
}

// Find the root of the workspace, not this crate, which is what `env!("CARGO_MANIFEST_DIR")` would return
fn workspace_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;

    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}
