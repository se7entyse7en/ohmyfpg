use ohmyfpg_core::client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    client::connect("postgres://postgres:postgres@localhost:5432/postgres".to_string()).await?;
    Ok(())
}
