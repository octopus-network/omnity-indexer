

# Omnity indexer

## Architecture
![alt text](./assets/architecture.png)  
Omnity Indexer is made up of three main parts: Synchronizer, Index database and API service: 
 
1. Synchronizer, which synchronizes data and states from omnity canisters and saves them to the index database. 
 
2. Index database, responsible for providing data storage, summary statistics and retrieval services to external system. 
 
3. API service, responsible for retrieving data from the index database, and providing a variety of interface access methods, such as restful, graphql and so on. 

## Local env test

### Deploy omnity ic canisters
```bash
# open new terminal and start dfx local net
 dfx start --clean

# open new terminal or tab
# clone omnity canister repo
git clone git@github.com:octopus-network/omnity.git
cd omnity ; git checkout boern/dev

# deploy the hub canister. Note: use your identity
dfx deploy omnity_hub --argument '(variant { Init = record { admin = principal "rv3oc-smtnf-i2ert-ryxod-7uj7v-j7z3q-qfa5c-bhz35-szt3n-k3zks-fqe"} })' --mode reinstall --yes

# deploy the bitcoin custom canister
dfx deploy bitcoin_mock --argument '(null)' --mode reinstall -y

# deploy the icp route canister
dfx deploy icp_mock --mode reinstall -y

# feed test data
./scripts/hub_test.sh

```
### Run PostgreSql and Hasura in docker
```bash
docker compose up -d
# check docker status
docker compose ps -a
```

### Create or drop the schema
```bash  
# enter docker
docker compose exec -it postgres bash

# connect to pg 
psql -U postgres

# create omnity db
CREATE DATABASE omnity ENCODING = 'UTF8';

# exit docker 
# install sea orm cli
cargo install sea-orm-cli

# clone and cd omnity-indexer 

# create the schema
sea-orm-cli migrate up -u postgres://postgres:omnity_go@localhost/omnity

# generate entity
#sea-orm-cli generate entity -o sync/src/entity
# drop the schema
#sea-orm-cli migrate down -u postgres://postgres:omnity_go@localhost/omnity

```

### Build and run the omnity indexer sync


```bash
cargo build --release -p omnity-indexer-sync

# update config.toml use your indentity and canister id

# start sync
./target/release/omnity_indexer_sync -c ~/config.toml start

# open other terminal and watch log
tail -f logs/omnity-indexer.log
```

### Hasura