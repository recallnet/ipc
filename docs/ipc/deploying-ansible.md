## Ansible Deployment Without Kubernetes

This project includes Ansible playbooks for deploying IPC blockchain node validators without using Kubernetes.

### Prerequisites

- [Install Ansible](https://docs.ansible.com/ansible/latest/installation_guide/intro_installation.html)
- Ensure you have access to the necessary inventory files for your environment.
- Set up your inventory variables file (`ansible/inventory/development/vars.yml`).

### Running the Ansible Playbook

To deploy the IPC blockchain node validators without Kubernetes, use the following command:

```sh
ansible-playbook -i ansible/inventory/development ansible/playbooks/deploy.yml
```
