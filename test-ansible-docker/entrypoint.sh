#!/bin/bash

# Run the Ansible playbook
ansible-playbook -i /ansible/inventory/development /ansible/playbooks/deploy.yml

# Keep the container running
tail -f /dev/null

