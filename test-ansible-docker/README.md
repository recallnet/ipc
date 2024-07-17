# TEST purposes only

A Dockerfile is NOT necessary for the ansible or helm deployments

The purpose of this directory is to enable local testing of the ansible scripts.

## Build the testing docker image

Run from the PARENT directory to find the ansible files 

```
cd ..
docker build -f test-ansible-docker/Dockerfile -t amazingdatamachine/ipc-node-validator .
```

## Run the test

```
docker run -it amazingdatamachine/ipc-node-validator
```

## For now using github secrets
## this will not be needed when our ipc repo is public

### ONE time - create secrets file with your github token
```
cd ansible
ansible-vault create roles/ipc-node/vars/github_secrets.yml
# after creating a password, enter your github token in this file
---
github_token: [your classic github token]
```

### To build and test manually

```
cd ..
docker build -f test-ansible-docker/Dockerfile.sleep -t amazingdatamachine/ipc-node-validator-manual .
docker run --name ipc-manual -it amazingdatamachine/ipc-node-validator-manual

# will sleep and stay running; in another shell:
docker exec -it ipc-manual /bin/bash
```
then in the docker container

```
cd /ansible
ansible-playbook playbooks/deploy.yml --ask-vault-pass
```
