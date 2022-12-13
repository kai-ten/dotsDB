#! /bin/sh

# Build gradle app
cd ./iceberg
./gradlew uberJar
cd -

# Apply terraform
cd ./iac
terraform apply -auto-approve
cd -
