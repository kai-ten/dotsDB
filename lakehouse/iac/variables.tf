variable "data_s3_bucket" {
  type = string
}

variable "athena_output_s3_bucket" {
  type = string
}

variable "lambda_code_s3_bucket" {
  type = string
}

variable "deploy_apigw" {
  type = bool
}
