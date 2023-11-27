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

| block_number | normal_dispatch_ref_time | operational_dispatch_ref_time | mandatory_dispatch_ref_time | normal_proof_size | operational_proof_size | mandatory_proof_size |
|--------------|---------------------------|-------------------------------|-----------------------------|-------------------|-------------------------|-----------------------|
| ...          | ...                       | ...                           | ...                         | ...               | ...                     | ...                   |

The percentages themselves are stored by representing them as decimal numbers; 
for example, 50.5% is stored as 0.505 with a precision of three decimals.
