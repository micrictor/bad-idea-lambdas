## Rust LRU Cache

An LRU cache that asks the question - what if I wanted to write a serverless, network accessible LRU cache?

By invoking the lambda, you can get and set values, as well as manipulate the max size of the cache.


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
