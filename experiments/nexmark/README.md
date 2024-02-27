# Nexmark Benchmark

This directory contains the Nexmark benchmark implemented for Aqua.

The benchmark contains 2 experiments:
* Experiment 1: The 8 standard Nexmark Queries. Each query:
* Experiment 2: A custom Sliding Window Aggregation query evaluated for different window sizes using bid data provided in the Nexmark dataset

All queries process a total of 1M events. Queries target Rust and Flink, with and without optimisations.

## Running

To run the experiments, install and startup [docker](https://docs.docker.com/) and then run:

```bash
./docker.sh
```

## Output

After running, plots of the experiments can be found in the generated `output/` folder.
