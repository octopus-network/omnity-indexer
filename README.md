

# Omnity indexer

## Architecture
![alt text](./assets/architecture.png)  
Omnity Indexer is made up of three main parts: Synchronizer, Index database and API service: 
 
1. Synchronizer, which synchronizes data and states from omnity canisters and saves them to the index database. 
 
2. Index database, responsible for providing data storage, summary statistics and retrieval services to external system. 
 
3. API service, responsible for retrieving data from the index database, and providing a variety of interface access methods, such as restful, graphql and so on. 

## Local deployment

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

1. Open browser and access http://localhost:8080/console
2. Config datasource and API service 

## Testnet or Mainnet deployment

### Deploy or upgrade omnity ic canisters

### Create the omnity indexer schema

```bash  
# create omnity db
psql -U postgres -h hostname/ip -p 5432 -c "CREATE DATABASE omnity ENCODING = 'UTF8';"

# import omnity db objects
psql -U postgres -h hostname/ip -p 5432 -d omnity < omnity.sql

```

### Update config.toml
```toml
# use your config env
database_url = 'postgres://postgres:open-sesame@localhost:5432/omnity'
dfx_network = 'http://127.0.0.1:4943'
log_config = './log4rs.yaml'
# dfx env vars
dfx_identity = './test.pem'
omnity_hub_canister_id = 'bkyz2-fmaaa-aaaaa-qaaaq-cai'
omnity_customs_bitcoin_canister_id = 'be2us-64aaa-aaaaa-qaabq-cai'
omnity_routes_icp_canister_id = 'br5f7-7uaaa-aaaaa-qaaca-cai'

```

### Build and run the omnity indexer sync

```bash
# first, install rust and compile the omnity indexer sync
cargo build --lock --release -p omnity-indexer-sync

# start sync
./target/release/omnity_indexer_sync -c ~/config.toml start

# optional,open other terminal and watch log
tail -f logs/omnity-indexer.log
```

### Config Hasura  
1. Deploy Hasura
1. Open browser and access hasura console，eg: http://localhost:8080/console 
2. Config database for omnity indexer
3. Import Hasura metadata:
   Navigate to the location: `SETTING -> METADATA -> Export metadata`
   Select `hasura_metadata.json` and import it
4. Open file `omnity_indexer.http` ,modify @host and test api service.
