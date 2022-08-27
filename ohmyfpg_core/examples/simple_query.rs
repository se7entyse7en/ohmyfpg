use ohmyfpg_core::client;
use std::time::Instant;

// 1. Run Postgres:
//     docker run -p 5432:5432 --name rust-postgres \
//       -e POSTGRES_PASSWORD=postgres -d postgres -c log_min_messages=DEBUG5
// 2. Create table:
//     CREATE TABLE performance_test (
//         id INT,
//         foo_bar_int2 INT2,
//         foo_bar_int4 INT4,
//         foo_bar_int8 INT8,
//         foo_bar_float4 FLOAT4,
//         foo_bar_float8 FLOAT8
//     );
// 3. Populate table:
//     INSERT INTO performance_test (
//         id,
//         foo_bar_int2,
//         foo_bar_int4,
//         foo_bar_int8,
//         foo_bar_float4,
//         foo_bar_float8
//     ) VALUES (
//         generate_series(1, 1000000),
//         trunc(random() * (2*32768) - 32768),
//         trunc(random() * (2*2147483648) - 2147483648),
//         trunc(random() * (2*9223372036854775808) - 9223372036854775808),
//         trunc(random()),
//         trunc(random())
//     );

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let t0 = Instant::now();
    let mut conn =
        client::connect("postgres://postgres:postgres@localhost:5432/postgres".to_string()).await?;
    let t1 = Instant::now();
    let t1_t0_elapsed = t0.elapsed();
    conn.fetch("SELECT * FROM performance_test".to_string())
        .await?;
    let t2_t1_elapsed = t1.elapsed();
    let t2_t0_elapsed = t0.elapsed();
    println!("[t1-t0] {}ms", t1_t0_elapsed.as_millis());
    println!("[t2-t1] {}ms", t2_t1_elapsed.as_millis());
    println!("[t2-t0] {}ms", t2_t0_elapsed.as_millis());
    Ok(())
}
