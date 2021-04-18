use lambda::{handler_fn, Context};
use serde::{Deserialize, Serialize};
use serde_json;
use lru::LruCache;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

use bytes::Bytes;
use rusoto_core::Region;
use rusoto_lambda::{Lambda, LambdaClient, UpdateFunctionCodeRequest};
use zip;

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

    let _ = update_runtime().await;

    Ok(response)
}

async fn get_value(key: String) -> Response {
    let cache_len = env::var("CACHE_MAX_ITEMS").unwrap_or("5".to_string()).parse::<usize>().unwrap();
    let mut cache: LruCache<String, String> = LruCache::new(cache_len);
    deserialize_cache(&mut cache);
    serialize_cache(&mut cache);

    let value = match cache.get(&key) {
        Some(value) => value,
        None => {
            return Response { value: "null".to_owned(), msg: "Key not in cache".to_owned()}
        }
    }

    Response { value: value.clone(), msg: String::from("Successfuly got value!") }
}

async fn set_value<'a>(key: String, value: String) -> Response {
    println!("Setting {} = {}", key, value);

    let cache_len = env::var("CACHE_MAX_ITEMS").unwrap_or("5".to_string()).parse::<usize>().unwrap();
    let mut cache: LruCache<String, String> = LruCache::new(cache_len);
    deserialize_cache(&mut cache);

    cache.put(key, value.clone());

    serialize_cache(&mut cache);

    Response { value: value.clone(), msg: String::from("Successfuly set value!") }
}

const CACHE_FILE: &str = "cache.json";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct CacheEntry {
    key: String,
    value: String,
}

fn serialize_cache<'a>(cache: &'a LruCache<String, String>) -> Vec<CacheEntry> {
    let cache_entries = cache.iter().map(|x| 
        CacheEntry{ key: x.0.clone(), value: x.1.clone() }).collect::<Vec<CacheEntry>>();

    let filepath = format!("/tmp/{}", CACHE_FILE);
    println!("Opening file {} for writing...", filepath);
    let file = OpenOptions::new().write(true).create(true).open(filepath);

    let _ = serde_json::to_writer(file.unwrap(), &cache_entries);
    cache_entries
}

fn deserialize_cache(cache: &mut LruCache<String, String>) {
    let file = OpenOptions::new().read(true).open(CACHE_FILE);
    let cache_entries: Vec<CacheEntry> = serde_json::from_reader(file.unwrap()).unwrap();
    for cache_entry in cache_entries.iter() {
        cache.put(cache_entry.key.clone(), cache_entry.value.clone());
    }
}

async fn update_runtime() {
    let runtime_dir = env::var("LAMBDA_TASK_ROOT").unwrap_or(String::from("./"));
    let lambda_name = env::var("AWS_LAMBDA_FUNCTION_NAME").unwrap();
    
    fs::copy(runtime_dir + "/bootstrap", "/tmp/bootstrap")
        .expect("Problem copying binary");

    // The cache.json containing the deserialized cache is already in /tmp at this point

    let mut buf: Vec<u8> = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o777);

        for filename in ["bootstrap", "cache.json"].iter() {
            let filepath = format!("/tmp/{}", filename);
            
            let contents = fs::read(filepath)
                .expect("Failed to read");

            zip.start_file(String::from(*filename), options)
                .expect("Error starting file");
            zip.write_all(&contents[..])
                .expect("Error writing file");
        }
        zip.finish()
            .expect("Error writing complete zip");
    }

    let buf_slice = &buf[..].to_owned();
    let lambda_client = LambdaClient::new(Region::default());

    let _result = lambda_client.update_function_code(UpdateFunctionCodeRequest {
        function_name: lambda_name,
        publish: Some(true),
        zip_file: Some(Bytes::from(buf_slice.clone())),
        ..Default::default()
    }).await.expect("Call to update function code failed");
}   
