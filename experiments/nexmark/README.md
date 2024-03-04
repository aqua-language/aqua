# Nexmark Benchmark

This directory contains the Nexmark benchmark implemented for Aqua, targeting Flink and Rust.

The benchmark contains 2 experiments:
* Experiment 1: The 8 standard Nexmark Queries.
* Experiment 2: A custom Sliding Window Aggregation query evaluated for different window sizes using bid data provided by Nexmark.

## Running

To run the experiments, install and startup [docker](https://docs.docker.com/) and then run:

```bash
./docker.sh
```

## Output

After running, plots of the experiments can be found in the generated `output/` folder.
