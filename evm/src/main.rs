use std::io::Write;

use alloy::primitives::address;
use alloy::sol;
use alloy::transports::http::reqwest::Url;

use alloy::providers::{ProviderBuilder, RootProvider};
use dotenv::dotenv;
use eyre::Result;
use samples::utils::{
    generate_csv_from_timestamped_data, get_contract_from_abi, sample_historical_data,
};
mod samples;

// Getting contract with sol macro (not currently in use)
sol!(
    #[derive(Debug)]
    IERC20,
    "abi/ERC20.json"
);

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    println!("Hello, world!");
    // Set up the HTTP transport which is consumed by the RPC client.
    let rpc_url: Url = std::env::var("RPC_URL")?.parse()?;

    let contract_address = address!("0xF5911DC17Ee45F46fe538ec972f4A500C78D8521");
    let provider: RootProvider = ProviderBuilder::default().on_http(rpc_url);
    let hello = get_contract_from_abi(contract_address, provider)?;

    let results = sample_historical_data(
        hello,
        "totalAssetsSync".into(),
        &[],
        28000519,
        28473032,
        2500,
        true,
    )
    .await?;
    // println!("Results: {:#?}", results);

    println!(
        "Results CSV: {:#?}",
        generate_csv_from_timestamped_data(results.clone())
    );
    std::fs::File::create("test.csv")?
        .write(generate_csv_from_timestamped_data(results).as_bytes())?;

    Ok(())
}
