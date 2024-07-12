# Use the latest Node.js 20 image as the base
FROM node:20

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    cmake \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    git \
    curl \
    ansible

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set up the workspace
WORKDIR /usr/src/app

# Copy Ansible playbooks and roles
COPY ../ansible /ansible

# Set the entrypoint
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]

