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

async fn set_value(key: String, value: String) -> Response {
    println!("{} = {}", key, value);

    Response { value: value, msg: String::from("Successfuly set value!") }
}

static CACHE_VALUES: [(&str, &str); 5] = [
    ("test_key0", "test_value0"),
    ("test_key1", "test_value1"),
    ("test_key2", "test_value2"),
    ("test_key3", "test_value3"),
    ("test_key4", "test_value4"),
];
