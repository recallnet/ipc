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

