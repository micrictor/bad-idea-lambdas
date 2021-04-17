use lambda::{handler_fn, Context};
use serde::{Deserialize, Serialize};
use serde_json;
use lru::LruCache;
use std::env;
use std::fs;
use std::fs::OpenOptions;

use rusoto_core::Region;
use rusoto_s3::{S3, S3Client};
use rusoto_lambda::{Lambda, LambdaClient, GetFunctionRequest};

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

fn default_value() -> String {
    String::from("default value")
}

#[derive(Deserialize)]
struct Request {
    #[serde(default)]
    operation: String,
    #[serde(default)]
    key: String,

    #[serde(default = "default_value")]
    value: String
}

#[derive(Serialize)]
struct Response {
    value: String,
    msg: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda::run(handler_fn(handler)).await?;
    Ok(())
}

async fn handler<'a>(event: Request, _: Context) -> Result<Response, Error> {
    let response: Response = match &event.operation as &str {
        "get" => get_value(event.key).await,
        "set" => set_value(event.key, event.value).await,
        &_ => Response { 
            value: String::from("null"),
            msg: String::from("Invalid command! Valid commands are get, set")
        }
    };

    // let _ = update_runtime().await;

    Ok(response)
}

async fn get_value(key: String) -> Response {
    let cache_len = env::var("CACHE_MAX_ITEMS").unwrap_or("5".to_string()).parse::<usize>().unwrap();
    let mut cache: LruCache<String, String> = LruCache::new(cache_len);
    get_cache_values(&mut cache);
    set_cache_values(&mut cache);

    let value = (cache.get(&key).unwrap()).clone();

    Response { value: value.clone(), msg: String::from("Successfuly got value!") }
}

async fn set_value<'a>(key: String, value: String) -> Response {
    println!("Setting {} = {}", key, value);

    let cache_len = env::var("CACHE_MAX_ITEMS").unwrap_or("5".to_string()).parse::<usize>().unwrap();
    let mut cache: LruCache<String, String> = LruCache::new(cache_len);
    get_cache_values(&mut cache);

    cache.put(key, value.clone());

    set_cache_values(&mut cache);

    Response { value: value.clone(), msg: String::from("Successfuly set value!") }
}

const CACHE_FILE: &str = "cache.json";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct CacheEntry {
    key: String,
    value: String,
}

fn set_cache_values<'a>(cache: &'a LruCache<String, String>) -> Vec<CacheEntry> {
    let cache_entries = cache.iter().map(|x| 
        CacheEntry{ key: x.0.clone(), value: x.1.clone() }).collect::<Vec<CacheEntry>>();

    let filepath = format!("/tmp/{}", CACHE_FILE);
    println!("Opening file {} for writing...", filepath);
    let file = OpenOptions::new().write(true).create(true).open(filepath);

    let _ = serde_json::to_writer(file.unwrap(), &cache_entries);
    cache_entries
}

fn get_cache_values(cache: &mut LruCache<String, String>) {
    let file = OpenOptions::new().read(true).open(CACHE_FILE);
    let cache_entries: Vec<CacheEntry> = serde_json::from_reader(file.unwrap()).unwrap();
    for cache_entry in cache_entries.iter() {
        cache.put(cache_entry.key.clone(), cache_entry.value.clone());
    }
}

async fn update_runtime() {
    let runtime_dir = env::var("LAMBDA_TASK_ROOT").unwrap_or(String::from("./"));
    let lambda_name = env::var("AWS_LAMBDA_FUNCTION_NAME").unwrap();

    // First, collect all the info about ourselves we'll need
    let lambda_client = LambdaClient::new(Region::default());
    let lambda_request = GetFunctionRequest { function_name: lambda_name, qualifier: None };
    let lambda_response_future = lambda_client.get_function(lambda_request);
    println!("Lambda reply: {:?}", lambda_response_future.await.unwrap());
    
    // Second, we need to create a new .zip with our JSON and bootstrap
    let _ = fs::copy(runtime_dir + "/bootstrap", "/tmp/bootstrap");
    let _zipfile = fs::File::create("lambda.zip");
}
