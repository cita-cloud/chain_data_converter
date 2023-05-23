use cita_cloud_proto::{
    blockchain::CompactBlock,
    client::{ClientOptions, InterceptedSvc, StorageClientTrait},
    retry::RetryClient,
    status_code::StatusCodeEnum,
    storage::{storage_service_client::StorageServiceClient, Content, ExtKey},
};
use prost::Message;
use std::env;

fn u64_decode(data: &[u8]) -> u64 {
    u64::from_be_bytes(data.try_into().unwrap())
}

async fn convert(
    old: &RetryClient<StorageServiceClient<InterceptedSvc>>,
    new: &RetryClient<StorageServiceClient<InterceptedSvc>>,
    region: u32,
    key: Vec<u8>,
) -> Vec<u8> {
    let extkey = ExtKey {
        region,
        key: key.clone(),
    };
    let res = old.load(extkey).await.unwrap();
    let code = res.status.as_ref().unwrap().code;
    let value = if code == 0 {
        res.value
    } else {
        panic!("load data failed: {:?}", StatusCodeEnum::from(code));
    };
    let content = Content {
        region,
        key,
        value: value.clone(),
    };
    new.store(content).await.unwrap();
    value
}

#[tokio::main]
async fn main() {
    // cargo run old_port new_port
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("args error: cargo run old_port new_port");
    }
    let old_port = &args[1];
    let new_port = &args[2];

    let client_options = ClientOptions::new(
        "old".to_string(),
        format!("http://127.0.0.1:{}", old_port),
    );
    let old_client = match client_options.connect_storage() {
        Ok(retry_client) => retry_client,
        Err(e) => panic!("client init error: {:?}", &e),
    };

    let client_options = ClientOptions::new(
        "new".to_string(),
        format!("http://127.0.0.1:{}", new_port),
    );
    let new_client = match client_options.connect_storage() {
        Ok(retry_client) => retry_client,
        Err(e) => panic!("client init error: {:?}", &e),
    };

    let key = ExtKey {
        region: 0,
        key: 0u64.to_be_bytes().to_vec(),
    };
    if !new_client.load(key.clone()).await.unwrap().value.is_empty() {
        panic!(
            "new storage should not stored data, check: old_port: {old_port}, new_port: {new_port}"
        )
    }
    let current_height = u64_decode(&old_client.load(key).await.unwrap().value);

    println!("old storage height: {current_height}");

    for height in 0..=current_height {
        let height_bytes = height.to_be_bytes().to_vec();

        // convert CompactBlock
        let compact_block_bytes = convert(&old_client, &new_client, 10, height_bytes.clone()).await;

        // convert tx in CompactBlock
        let compact_block = CompactBlock::decode(compact_block_bytes.as_slice()).unwrap();
        let body = compact_block.body.unwrap();
        for tx_hash in body.tx_hashes {
            // convert Transactions
            convert(&old_client, &new_client, 1, tx_hash.clone()).await;
            // convert TransactionHash2blockHeight
            convert(&old_client, &new_client, 7, tx_hash.clone()).await;
            // convert TransactionIndex
            convert(&old_client, &new_client, 9, tx_hash.clone()).await;
        }
        // convert BlockHash
        let hash = convert(&old_client, &new_client, 4, height_bytes.clone()).await;

        // convert BlockHash2blockHeight
        convert(&old_client, &new_client, 8, hash).await;

        // convert Proof
        convert(&old_client, &new_client, 5, height_bytes.clone()).await;

        // convert Result
        convert(&old_client, &new_client, 6, height_bytes.clone()).await;

        // update new storage height
        let content = Content {
            region: 0,
            key: 0u64.to_be_bytes().to_vec(),
            value: height_bytes,
        };
        new_client.store(content).await.unwrap();

        println!("new storage stores height({height}) succeed");
    }
}
