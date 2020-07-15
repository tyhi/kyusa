# kyusa
kyusa is a ShareX server written in rust.


## Endpoints
- /file
	- kyusa stores all files are stored in /uploads named as its blake3 hash
- /
	- This is where you will send the multipart uploads to.
## Features
- Fast, written in rust and actix-web this keeps thing running as quick as possible.

## sqlx-cli
Kyusa uses sqlx-cli to manage database migrations for postgres. Make sure to have your DATABASE_URL env set then run:

```
cargo sqlx database create
cargo sqlx migration run
```

This will hopefully be built into the app down the road.
