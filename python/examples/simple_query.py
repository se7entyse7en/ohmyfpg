import asyncio
import time

import ohmyfpg


# 1. Run Postgres:
#     docker run -p 5432:5432 --name rust-postgres \
#       -e POSTGRES_PASSWORD=postgres -d postgres -c log_min_messages=DEBUG5
# 2. Create table:
#     CREATE TABLE performance_test (
#         id INT,
#         foo_bar_int2 INT2,
#         foo_bar_int4 INT4,
#         foo_bar_int8 INT8,
#         foo_bar_float4 FLOAT4,
#         foo_bar_float8 FLOAT8
#     );
# 3. Populate table:
#     INSERT INTO performance_test (
#         id,
#         foo_bar_int2,
#         foo_bar_int4,
#         foo_bar_int8,
#         foo_bar_float4,
#         foo_bar_float8
#     ) VALUES (
#         generate_series(1, 1000000),
#         trunc(random() * (2*32768) - 32768),
#         trunc(random() * (2*2147483648) - 2147483648),
#         trunc(random() * (2*9223372036854775808) - 9223372036854775808),
#         trunc(random()),
#         trunc(random())
#     );


dsn = 'postgres://postgres:postgres@localhost:5432/postgres'
query = 'SELECT * FROM performance_test'


async def main():
    """Run main."""
    t0 = time.time()
    conn = await ohmyfpg.connect(dsn)
    t1 = time.time()
    await conn.fetch(query)
    t2 = time.time()
    print(f"[t1-t0] {int((t1 - t0) * 1000)}ms")
    print(f"[t2-t1] {int((t2 - t1) * 1000)}ms")
    print(f"[t2-t0] {int((t2 - t0) * 1000)}ms")


asyncio.run(main())
