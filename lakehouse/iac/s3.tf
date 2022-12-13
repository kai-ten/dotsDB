module "dotsdb_athena_bucket" {
  source = "./modules/buckets"
  bucket_name = var.data_s3_bucket
}

module "dotsdb_data_bucket" {
  source = "./modules/buckets"
  bucket_name = var.athena_output_s3_bucket
}

module "dotsdb_lambda_bucket" {
  source = "./modules/buckets"
  bucket_name = var.lambda_code_s3_bucket
}
