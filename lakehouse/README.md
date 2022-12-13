# Lakehouse

The lakehouse folder contains the necessary code to spin up and tear down a data lake for testing purposes.

While this project will continue to receive updates, they will only be for a certain cause. <br />
Please read the below to understand what I am currently going for versus what an ideal state may look like.

<br />

__Ideally, this Iceberg-based data lakehouse will:__

- :heavy_check_mark: Create tables, schemas, partitions, etc using Apache Iceberg libs and AWS Glue as the catalog, using a hardcoded schema
- Update tables, schemas, partitions, etc as needed for this hardcoded schema
- Delete tables when they are no longer configured
- Accept API Gateway requests to create records 


__Future goals may be:__

- Support arbitrary data schemas
- Support multiple ingestion patterns with built-in redundancy
- Support multiple catalogs
- Support multiple clouds

<br />
<hr />
<br />

## Getting Started

<br />

### Requirements & Expectations

- Recommended you use tfenv, if not then configure the correct terraform version on your machine
- JDK 11 (current LTS support for AWS Lambda)
- Configured your machine with AWS Credentials, as well as having necessary permissions to the account that you are deploying to

<br />

### How to Deploy

1. Navigate to the iac directory
1. Run `terraform init` 
1. Rename the file __terraform.example.tfvars__ to __terraform.tfvars__
    - This will automatically configure tfvars when applying the terraform infrastructure
    - Note that if you add environment specific tfvars file, you are responsible for adding these files to .gitignore
1. Set your unique S3 bucket names in the terraform.tfvars file
    - One bucket will store the data to test with
    - One bucket will store Athena output
    - One bucket will store the lambda function archives
1. Navigate back to the project root
1. Run `chmod +x ./build.sh`
1. Run `./build.sh`


Data source / inspiration = [AWS Open Source Book Reviews Data](https://s3.console.aws.amazon.com/s3/buckets/amazon-reviews-pds?region=us-east-1&prefix=parquet/product_category%3DBooks/&showversions=false)
