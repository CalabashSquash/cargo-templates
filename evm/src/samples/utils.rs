use alloy::{
    consensus::BlockHeader,
    contract::{ContractInstance, Interface},
    dyn_abi::DynSolValue,
    eips::BlockNumberOrTag,
    network::Ethereum,
    primitives::{Address, Bytes},
    providers::{Provider, RootProvider},
    rpc::types::Block,
    signers::local::PrivateKeySigner,
};

use eyre::{anyhow, Result};
use serde_json::Value;

fn fetch_pk() -> Result<PrivateKeySigner> {
    todo!("fetch_pk");
}

fn sol_value_to_string(val: &DynSolValue) -> String {
    match val {
        DynSolValue::Bool(b) => b.to_string(),
        DynSolValue::Int(i, _) => i.to_string(),
        DynSolValue::Uint(u, _) => u.to_string(),
        DynSolValue::FixedBytes(word, _) => word.to_string(),
        DynSolValue::Address(a) => a.to_string(),
        DynSolValue::Function(f) => f.to_string(),
        DynSolValue::Bytes(b) => format!("{:X?}", b),
        DynSolValue::String(s) => s.to_string(),
        DynSolValue::Array(v) => v.iter().fold(String::new(), |old, next| {
            format!("{old}, {}", &sol_value_to_string(&next))
        }),
        DynSolValue::FixedArray(v) => v.iter().fold(String::new(), |old, next| {
            format!("{old}, {}", &sol_value_to_string(next))
        }),
        DynSolValue::Tuple(v) => v.iter().fold(String::new(), |old, next| {
            format!("{old}, {}", &sol_value_to_string(next))
        }),
        #[cfg(feature = "eip712")]
        DynSolValue::CustomStruct {
            name,
            prop_names,
            tuple,
        } => {}
    }
}

pub fn generate_csv_from_timestamped_data(data: Vec<(u64, Vec<DynSolValue>)>) -> String {
    let mut csv = String::new();
    data.iter().for_each(|(timestamp, element)| {
        csv.push_str(&format!("{}", &timestamp.to_string()));
        let elems: String = element.iter().fold(String::new(), |old, x| {
            format!("{old},{}", sol_value_to_string(x))
        });
        csv.push_str(&format!("{elems}\n"));
    });
    csv
}

pub async fn sample_historical_data<T: Provider>(
    contract: ContractInstance<T, Ethereum>,
    function_name: String,
    args: &[DynSolValue],
    start_block: u64,
    end_block: BlockNumberOrTag,
    interval: u64,
    ignore_reverts: bool,
) -> Result<Vec<(u64, Vec<DynSolValue>)>> {
    let mut results: Vec<(u64, Vec<DynSolValue>)> = vec![];
    let function_builder = contract.function(&function_name, args)?;

    let provider = contract.provider();

    let true_end_block = match end_block {
        BlockNumberOrTag::Latest => {
            let latest_block = provider
                .get_block_by_number(BlockNumberOrTag::Latest)
                .await?
                .expect("No Latest Block!");
            latest_block.header.number()
        }
        BlockNumberOrTag::Number(n) => n,
        _ => {
            return Err(anyhow!("Only Latest Block or specific block number"));
        }
    };

    let mut code = Bytes::default();

    for n in 0..(true_end_block - start_block) / interval {
        if code.is_empty() {
            code = provider
                .get_code_at(*contract.address())
                .block_id((start_block + (n * interval)).into())
                .await?;
            continue;
        }
        let call_result = function_builder
            .clone()
            .block((start_block + (n * interval)).into())
            .call()
            .await;
        if let Ok(res) = call_result {
            results.push((start_block + (n * interval), res));
        } else {
            if !ignore_reverts {
                return Err(anyhow!("Call reverted"));
            }
        }
    }
    Ok(results)
}

/// Queries the blockchain via an `eth_call` without submitting a transaction to the network.
/// This function is kind of useless but keeping here for example's sake :)
async fn make_function_call<T: Provider>(
    contract: ContractInstance<T, Ethereum>,
    function_name: String,
    args: &[DynSolValue],
) -> Result<Vec<DynSolValue>, alloy::contract::Error> {
    contract.function(&function_name, args)?.call().await
}

pub fn get_contract_from_abi(
    contract_address: Address,
    provider: RootProvider,
) -> Result<ContractInstance<RootProvider, Ethereum>> {
    let path = std::env::current_dir()?.join("abi/AutocompoundLP.json");

    // Read the artifact which contains `abi`, `bytecode`, `deployedBytecode` and `metadata`.
    let artifact = std::fs::read(path).expect("Failed to read artifact");
    let json: serde_json::Value = serde_json::from_slice(&artifact)?;

    // Get `abi` from the artifact.
    let abi_value = json.get("abi").expect("Failed to get ABI from artifact");

    if let Value::Array(_) = abi_value {
        let abi = serde_json::from_value(abi_value.clone())?;
        // let json_abi = JsonAbi::parse(abi_string);
        let contract/*: ContractInstance<RootProvider<Ethereum>, Ethereum>*/ =
            ContractInstance::new(contract_address, provider.clone(), Interface::new(abi));
        return Ok(contract);
    } else {
        return Err(anyhow!("ABI Invalid"));
    }
}

pub async fn get_latest_block(provider: RootProvider) -> Block {
    let latest = BlockNumberOrTag::Latest;
    provider
        .get_block_by_number(latest)
        .await
        .expect("getting block failed")
        .expect("No Block found")
}
