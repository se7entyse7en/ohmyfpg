# Oh My Fast Postgres!

Oh My Fast Postgres!

## Basic benchmarking

```
python python/examples/simple_query.py
```

```
RUST_BACKTRACE=1 cargo run -r -p ohmyfpg_core --example simple_query
```

## Basic profiling

```
sudo CARGO_PROFILE_BENCH_DEBUG=true RUST_BACKTRACE=1 cargo flamegraph -p ohmyfpg_core --example simple_query
```
