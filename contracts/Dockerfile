FROM ubuntu:latest

RUN apt-get update && apt-get install -y \
    build-essential libssl-dev mesa-opencl-icd ocl-icd-opencl-dev gcc git bzr jq pkg-config curl clang hwloc libhwloc-dev wget ca-certificates gnupg

COPY . /ipc
WORKDIR /ipc
# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# Install forge
RUN curl -L https://foundry.paradigm.xyz | bash
RUN . /root/.bashrc && foundryup
# Install nvm
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
# Set nvm to use system node
ENV NVM_DIR=/root/.nvm
RUN . $NVM_DIR/nvm.sh && nvm install 20

# build contracts
# RUN . /root/.bashrc && cd contracts && forge install && forge build && npm install hardhat
