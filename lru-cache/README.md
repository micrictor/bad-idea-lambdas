## Rust LRU Cache

An LRU cache that asks the question - what if I wanted to write a serverless, network accessible LRU cache?

By invoking the lambda, you can get and set values, as well as manipulate the max size of the cache.

The cache is maintained as a JSON-blob, stored inside the Lambda code definition alongside the executable actually being ran. This JSON is deserialized into an LRU cache, with the max size dictated by an environment variable. The resulting cache, after the apprpriate get/set, is serialized back into a file. This file, and the compiled Rust binary, are then zipped up and used to create a new version of the Lambda function.

Meta, right?


## Build/deploy

As this was made using the serverless framwork, you can install all needed dependencies and deploy it to your own AWS account using two commands.
1. `npm install`
2. `npx serverless deploy`

## Expected inputs

The Lambda expects a JSON body as a parameter. 

The body looks roughly like this:

```
{
  "operation": "<get/set>",
  "key": "<cache key to operate on>",
  "value": "<optional, value to set for set request>"
}
```
