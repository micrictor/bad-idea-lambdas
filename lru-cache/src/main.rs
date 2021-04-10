use lambda::{handler_fn, Context};
use serde::{Deserialize, Serialize};
use lru::LruCache;
use std::env;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

fn default_value() -> String {
    String::from("default value")
}

#[derive(Deserialize)]
struct Request {
    operation: String,
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

    Ok(response)
}

async fn get_value(key: String) -> Response {
    let cache_len = env::var("CACHE_MAX_ITEMS").unwrap_or("5".to_string()).parse::<usize>().unwrap();
    let mut cache: LruCache<&str, &str> = LruCache::new(cache_len);
    for (cache_key, cache_value) in CACHE_VALUES.iter() {
        cache.put(cache_key, cache_value);
    }

    let value = *cache.get(&&*key as &&str).unwrap_or(&"");

    Response { value: String::from(value), msg: String::from("Successfuly got value!") }
}

const VALUES_HEADER: &str = "static CACHE_VALUES: [(&str, &str); 5] = [";
const VALUES_FORMAT_ENTRY: &str = "    (\"{0}\", \"{1}\"),";
const VALUES_FOOTER: &str = "];";

async fn set_value<'a>(key: String, value: String) -> Response {
    println!("Setting {} = {}", key, value);
    let response = Response { value: value, msg: String::from("Successfuly set value!") };

    let cache_len = env::var("CACHE_MAX_ITEMS").unwrap_or("5".to_string()).parse::<usize>().unwrap();
    let mut cache: LruCache<&str, &str> = LruCache::new(cache_len);
    for (cache_key, cache_value) in CACHE_VALUES.iter() {
        cache.put(cache_key, cache_value);
    }

    cache.put(&key as &str, &response.value as &str);
    let _values = get_cache_values(&cache);


    drop(cache);
    return response;
}

fn get_cache_values<'a>(cache: &'a LruCache<&str, &str>) -> Vec<(&'a str, &'a str)> {
    let result = cache.iter().map(|x| (*x.0, *x.1)).collect::<Vec<(&str, &str)>>();
    drop(cache);
    result
}

static CACHE_VALUES: [(&str, &str); 5] = [
    ("test_key0", "test_value0"),
    ("test_key1", "test_value1"),
    ("test_key2", "test_value2"),
    ("test_key3", "test_value3"),
    ("test_key4", "test_value4"),
];
