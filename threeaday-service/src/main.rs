mod service;

use anyhow::Result;
use service::run_service;

#[tokio::main]
async fn main() -> Result<()> {
    run_service().await
}