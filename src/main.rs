use cita_cloud_proto::{
    client::{ClientOptions, InterceptedSvc, StorageClientTrait},
    retry::RetryClient,
    storage::{storage_service_client::StorageServiceClient, Content, ExtKey},
};
use std::env;

fn u64_decode(data: &[u8]) -> u64 {
    u64::from_be_bytes(data.try_into().unwrap())
}

async fn convert(
    old: &RetryClient<StorageServiceClient<InterceptedSvc>>,
    new: &RetryClient<StorageServiceClient<InterceptedSvc>>,
    height: u64,
) {
    let height_bytes = height.to_be_bytes().to_vec();
    let block_extkey = ExtKey {
        region: 11,
        key: height_bytes.clone(),
    };
    let block = old.load(block_extkey).await.unwrap().value;
    let hash_extkey = ExtKey {
        region: 4,
        key: height_bytes.clone(),
    };
    let mut bytes = old.load(hash_extkey).await.unwrap().value;
    bytes.extend_from_slice(block.as_slice());

    let content = Content {
        region: 12,
        key: height_bytes.clone(),
        value: bytes,
    };
    let status = new.store(content).await.unwrap();

    if status.code != 0 {
        panic!("store height: {} failed: {:?}", height, status)
    }
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

    let client_options =
        ClientOptions::new("old".to_string(), format!("http://127.0.0.1:{}", old_port));
    let old_client = match client_options.connect_storage() {
        Ok(retry_client) => retry_client,
        Err(e) => panic!("client init error: {:?}", &e),
    };

    let client_options =
        ClientOptions::new("new".to_string(), format!("http://127.0.0.1:{}", new_port));
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
        convert(&old_client, &new_client, height).await;
        println!("converted: {} / {}", height, current_height);
    }
}
