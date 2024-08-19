# Consumption Tracker

## Overview

The program is designed to fetch weight utilization data from a predefined set
of parachains. The obtained weight information is then stored in the `out`
directory as multiple CSV files.

## Output Structure

Each parachain has its own dedicated output file, and these files are updated
every time a new block is finalized and the weight consumption data is
successfully queried.

## Data structure

The data stored is the 2D weight consumption per each dispatch class.
The data is stored in the CSV file within the following sequence:

| block_number | timestamp             | normal_dispatch_ref_time | operational_dispatch_ref_time | mandatory_dispatch_ref_time | normal_proof_size | operational_proof_size | mandatory_proof_size |
|--------------|-----------------------|---------------------------|-------------------------------|-----------------------------|-------------------|-------------------------|-----------------------|
| ...          | ...                   | ...                       | ...                           | ...                         | ...               | ...                     | ...                   |

The percentages themselves are stored by representing them as decimal numbers; 
for example, 50.5% is stored as 0.505 with a precision of three decimals.

## Building & Running

To compile the Corespace Weigher project run the following command from the root of the repo:
```
cargo build --release
```

This will output binaries: `tracker` and `server`

The `tracker` binary is responsible for tracking the actual consumption data of parachains. This program will read the parachains.json file to obtain the list of parachains for which it will track consumption data by listening to the latest blocks from the specified RPC nodes.

The `server` binary provides a web interface that can be used for registering a parachain for consumption tracking, as well as for querying all the consumption data.

### Watchdog 🐕

WebSocket connections can be closed due to underlying networking issues. In such cases, the tracking of parachain data would stop. For this reason, a script called 'watchdog' is introduced to ensure the tracker attempts to create a new connection whenever the current one is broken.

```sh
./scripts/watchdog.sh
```

## Web API

#### Registering a parachain

A basic example of registering a parachain:

```
curl -X POST http://127.0.0.1:8000/register_para -H "Content-Type: application/json" -d '{
    "para": ["Polkadot", 2000]                                                 
}'
```

#### Querying consumption data

A basic example of querying the consumption of a parachain with the paraID 2000 that is part of the Polkadot network:

```
curl http://127.0.0.1:8000/consumption/polkadot/2000
```

## Local development

For local development, you can run the entire suite of tests using the command below. It's important to run tests sequentially as some of them depend on shared mock state. This approach ensures that each test runs in isolation without interference from others.
```
cargo test -- --test-threads=1
```
