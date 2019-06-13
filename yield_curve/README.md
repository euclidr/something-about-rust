# yield curve api

It's a toy project that sync daily American bond yield data and provide an API to fetch daily record.

In this project I use `hyper` to build an HTTP server, I also use `hyper` to request data from treasury government official website.

In order to extract data from the HTML, `select` crate is used.

Fetching data from remote website is slow(in China), I have to store it in a database and I use file to store the data for simplicity and wrap it as a kv storage.

The data is updated daily, I also store sync records in that kv storage, so we can check if there's new data only if it's necessary.

When the application started up, it will load all data from `kv.db` file, and then it will check if all history data is synced, and sync it in background if not.

There is a periodic checker to check if new data should be synced, I use `Interval` from `tokio` to achieve periodic job.

Result of request is serialized into json with a `code` indicate whether it is a correct result.