## Kubernetes Deployment with Helm

This project includes Helm charts for deploying and managing IPC blockchain node validators on Kubernetes.

### Prerequisites

- [Install Kubernetes](https://kubernetes.io/docs/setup/)
- [Install Helm](https://helm.sh/docs/intro/install/)

### Deploying the Helm Chart

1. **Install Helm:**
    ```sh
    curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
    ```

2. **Add Helm Stable Repository:**
    ```sh
    helm repo add stable https://charts.helm.sh/stable
    ```

3. **Deploy the Helm Chart:**
    ```sh
    helm install ipc-node-validator ./helm/ipc-node-validator
    ```

This will deploy the IPC node validators to your Kubernetes cluster using the Helm chart defined in the `helm/ipc-node-validator` directory.

