# Oh My Fast Postgres!

Oh My Fast Postgres!

## Basic benchmarking

```
docker run -p 5432:5432 --name rust-postgres -e POSTGRES_PASSWORD=postgres -d postgres -c log_min_messages=DEBUG5
```

Data preparation:
```
CREATE TABLE performance_test (id INT, foo_bar_int2 INT2, foo_bar_int4 INT4, foo_bar_int8 INT8, foo_bar_float4 FLOAT4, foo_bar_float8 FLOAT8);
INSERT INTO performance_test (
    id,
    foo_bar_int2,
    foo_bar_int4,
    foo_bar_int8,
    foo_bar_float4,
    foo_bar_float8
) VALUES (
    generate_series(1, 1000000),
    trunc(random() * (2*32768) - 32768),
    trunc(random() * (2*2147483648) - 2147483648),
    trunc(random() * (2*9223372036854775808) - 9223372036854775808),
    trunc(random()),
    trunc(random())
);
```

```
maturin develop --release --manifest-path ohmyfpg/Cargo.toml
python python/examples/simple_query.py
```

```
RUST_BACKTRACE=1 cargo run -r -p ohmyfpg_core --example simple_query
```

## Basic profiling

```
sudo CARGO_PROFILE_BENCH_DEBUG=true RUST_BACKTRACE=1 cargo flamegraph -p ohmyfpg_core --example simple_query
```
