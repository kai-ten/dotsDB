# API Gateway Ingestion

This lambda processes JSON arrays (and ndjson?) that are sent to an API Gateway endpoint. <br />
The event body is converted to parquet and stored, and the Iceberg metadata + manifest is created
to indicate that new records were inserted into the Iceberg table.



## Requirements to build

- cargo-lambda (see https://github.com/awslabs/aws-lambda-rust-runtime)


## Build

- `cargo lambda build --release --arm64`

