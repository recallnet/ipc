#!/usr/bin/env bash

# This file downloads the required nushell version to ./.nu folder and adds it to the PATH.

set +e

if command -v nu > /dev/null 2>&1; then
    echo "Nushell already installed"
else
    set -e
    echo "Installing Nushell..."
    version="0.103.0"
    nu_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/.nu"
    mkdir -p $nu_dir
    cd $nu_dir
    os=$(uname | sed -e s/darwin/apple-darwin/i -e s/linux/unknown-linux-gnu/i)
    arch=$(uname -m | sed -e s/arm64/aarch64/)
    curl -Lo nu.tgz https://github.com/nushell/nushell/releases/download/${version}/nu-${version}-${arch}-${os}.tar.gz
    tar xf nu.tgz
    mv nu-*/nu .
    rm -rf nu-* nu.tgz
    echo "export PATH=$(pwd):\$PATH" > $nu_dir/activate.sh
    source ./activate.sh

    echo "Nushell installed at $(which nu)"
    nu -c version
    echo "Activate: source $nu_dir/activate.sh"
fi
