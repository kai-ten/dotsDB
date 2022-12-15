#! /bin/sh

# Build gradle app
cd ./lib/iceberg
./gradlew uberJar
cd -

# TODO: Include Rust cargo-lambda build

# Apply terraform
cd ./iac
terraform apply -auto-approve
cd -
