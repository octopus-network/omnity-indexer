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
# open new terminal or tab
# clone omnity canister repo
git clone git@github.com:octopus-network/omnity.git
cd omnity ; git checkout ting/indexer

# deploy the omnity canisters. Note: use your identity
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
cargo build --locked --release -p omnity-indexer-sync

# update config.toml use your indentity and canister id

# start sync
./target/release/omnity_indexer_sync -c ./config.toml start

# optional,open other terminal and watch log
tail -f logs/omnity-indexer.log

```
### Optional update log4rs.yaml
If you want the output into log file ,just modify the appenders,like the following:

```yaml
refresh_rate: 30 seconds
appenders:
  stdout:
    kind: console
  requests:
    kind: rolling_file
    path: "logs/omnity-indexer.log"
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 5mb 
      roller:
        kind: fixed_window
        pattern: "logs/omnity-indexer.{}.log"
        base: 1
        count: 5  
root:
  level: info
  appenders:
    - requests
loggers:
  app::requests:
    level: info
    appenders:
      - requests
    additive: false

```


### Update config.toml
```toml
# use your config env
database_url = 'postgres://postgres:omnity_go@localhost:5432/omnity'
dfx_network = 'http://127.0.0.1:4943'
log_config = './log4rs.yaml'
# dfx env vars
dfx_identity = './test.pem'
omnity_hub_canister_id = 'bkyz2-fmaaa-aaaaa-qaaaq-cai'
omnity_customs_bitcoin_canister_id = 'be2us-64aaa-aaaaa-qaabq-cai'
omnity_routes_icp_canister_id = 'br5f7-7uaaa-aaaaa-qaaca-cai'

```

### Config Hasura  
1. Deploy Hasura
1. Open browser and access hasura console，eg: http://localhost:8080/console 
2. Config database for omnity indexer
3. Import Hasura metadata:
   Navigate to the location: `SETTING -> METADATA -> Export metadata`
   Select `hasura_metadata.json` and import it
4. Open file `omnity_indexer.http` ,modify @host and test api service.


## Testnet or Mainnet deployment

### Deploy or upgrade omnity ic canisters

### Create the omnity indexer schema

```bash  
# create omnity db
psql -U postgres -h hostname/ip -p 5432 -c "CREATE DATABASE omnity ENCODING = 'UTF8';"

# import omnity db objects
psql -U postgres -h hostname/ip -p 5432 -d omnity < omnity.sql

```
### Config identity
```bash  

# export DFX_IDENTITY=$(<./test.pem)
# export DFX_IDENTITY=$(dfx identity export default)
export DFX_IDENTITY=your identity file
```

### Update config.toml
```toml
# use your config env
database_url = 'postgres://postgres:omnity_go@localhost:5432/omnity'
dfx_network = 'http://127.0.0.1:4943'
log_config = './log4rs.yaml'
# dfx env vars
# dfx_identity = './test.pem'
omnity_hub_canister_id = 'bkyz2-fmaaa-aaaaa-qaaaq-cai'
omnity_customs_bitcoin_canister_id = 'be2us-64aaa-aaaaa-qaabq-cai'
omnity_routes_icp_canister_id = 'br5f7-7uaaa-aaaaa-qaaca-cai'

```

### Build and run the omnity indexer sync

```bash
# first, install rust and compile the omnity indexer sync
cargo build --locked --release -p omnity-indexer-sync

# start sync
./target/release/omnity_indexer_sync -c ./config.toml start

```

### Config Hasura  
1. Deploy Hasura
1. Open browser and access hasura console，eg: http://localhost:8080/console 
2. Config database for omnity indexer
3. Import Hasura metadata:
   Navigate to the location: `SETTING -> METADATA -> Export metadata`
   Select `hasura_metadata.json` and import it
4. Open file `omnity_indexer.http` ,modify @host and test api service.
