# Oh My Fast Postgres!

`ohmyfpg` is a Postgres client library for Python that aims to return data as columns. This is often needed when working with numerical data. Usually this is achieved by taking the output of the client library and then convert it either into `numpy` arrays or `pandas` dataframes. When dealing with a large amount of data, this conversion is not much performant.

The goal of this library is to return data already as `numpy` arrays without sacrificing performance. The section of "Performance comparison" goes more in-depth on this topic.

In order to squeeze performance the underlying implementation is written in Rust. The Python layer on top is very thin.

## Why `ohmyfpg`?

When working with Postgres at work we faced multiple times performance issues. Most of the times our reactions were along the lines of: "OMG", "F*****g PG", etc. So `ohmyfpg` is kinda a mix of the two, but where the `f` now stands for `fast`.

(To be fair, when we faced performance issues with Postgres was most of the times because of our inexperience with tuning the server configurations.)

## Installation

```
pip install ohmyfpg
```

## Quickstart

```
import asyncio
import ohmyfpg

DSN = 'postgres://postgres:postgres@postgres:5432/postgres'
QUERY = 'SELECT * FROM performance_test'

async def main():
    conn = await ohmyfpg.connect(DSN)
    print(await conn.fetch(QUERY))

if __name__ == '__main__':
    asyncio.run(main())
```

## Performance comparison

The image below compares the performance of `ohmyfpg` with `asyncpg`. The 4 bars have the following meaning:
- `ohmyfpg`: plain fetch,
- `asyncpg`: plain fetch,
- `ohmyfpg-pandas`: plain fetch + conversion to `pandas` Dataframe,
- `asyncpg`: plain fetch + conversion to `pandas` Dataframe,

See details [here](performance/compare.py), especially how the conversion to `pandas` Dataframe has been implemented.

The query is a `SELECT *` that has been run on a table with 1mln rows and the following schema:
```
(id INT, foo_bar_int2 INT2, foo_bar_int4 INT4, foo_bar_int8 INT8, foo_bar_float4 FLOAT4, foo_bar_float8 FLOAT8)
```

It has been run inside docker with 8 CPU and 8GB of RAM allocated to the daemon on a MBP with 2.2 GHz 6-Core Intel Core i7 and 16GB 2400 MHz DDR4.

![Performance comparison](performance/performance-comparison.png)

### Detailed summary

Plain fetch:
```
--------------------------------------------------
ohmyfpg
avg: 1045.8ms
min: 898ms
p25: 960.75ms
median: 1041.5ms
p75: 1083.0ms
max: 1421ms
--------------------------------------------------
asyncpg
avg: 1194.9ms
min: 1037ms
p25: 1080.5ms
median: 1224.0ms
p75: 1259.75ms
max: 1567ms
--------------------------------------------------
```

Plain fetch + conversion to `pandas` Dataframe:
```
--------------------------------------------------
ohmyfpg-pandas
avg: 1212.3ms
min: 1131ms
p25: 1166.5ms
median: 1192.0ms
p75: 1220.75ms
max: 1724ms
--------------------------------------------------
asyncpg-pandas
avg: 4013.0333333333333ms
min: 3771ms
p25: 3841.25ms
median: 3912.5ms
p75: 4124.5ms
max: 4708ms
--------------------------------------------------
```

## Limitations

This library is highly experimental and has many limitations:
- no support for `NULL`s with unpredictable outcome,
- no support for non-numerical types,
- limited support for authentication,
- no proper logging,
- etc.

## Development

### How to run the performance comparison

```
docker compose build script
docker compose up -d postgres
docker compose exec -- postgres psql -U postgres

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


docker compose up script
docker compose cp script:/usr/src/app/performance-comparison.png ./performance
```

### How to do basic benchmarking

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

### How to do basic profiling

```
sudo CARGO_PROFILE_BENCH_DEBUG=true RUST_BACKTRACE=1 cargo flamegraph -p ohmyfpg_core --example simple_query
```

```
CARGO_PROFILE_BENCH_DEBUG=true RUST_BACKTRACE=1 cargo instruments --release -p ohmyfpg_core --example simple_query -t time
```
