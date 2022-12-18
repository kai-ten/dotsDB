#! /bin/sh

# Build gradle app
cd ./lib/iceberg
./gradlew uberJar
cd -

# TODO: Include Rust cargo-lambda build
cd ./lib/apigw-ingest
cargo lambda build --release --arm64
cd -

# Apply terraform
cd ./iac
terraform apply -auto-approve
cd -
