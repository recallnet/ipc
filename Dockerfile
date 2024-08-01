FROM ubuntu:latest

RUN apt-get update && \
  apt-get install -y build-essential clang cmake pkg-config libssl-dev protobuf-compiler git curl

# Add rust to the container
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install cargo-make
RUN cargo install --force cargo-make

# Add wasm32-unknown-unknown target
RUN rustup target add wasm32-unknown-unknown

# Install Foundry
RUN curl -L https://foundry.paradigm.xyz | bash
ENV PATH="/root/.foundry/bin:${PATH}"
RUN foundryup

# Clean up
RUN apt-get clean && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the current directory contents into the container at /app
COPY . /app

# Run make
RUN make

# Set the default command for the container
CMD ["bash"]
