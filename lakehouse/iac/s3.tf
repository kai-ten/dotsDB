# dotsDB iceberg data bucket
resource "aws_s3_bucket" "dotsdb_iceberg_data" {
  bucket = var.data_s3_bucket
}

resource "aws_s3_bucket_acl" "dotsdb_iceberg_acl" {
  bucket = aws_s3_bucket.dotsdb_iceberg_data.id
  acl    = "private"
}

resource "aws_s3_bucket_public_access_block" "dotsdb_iceberg_data_public_access" {
  bucket = aws_s3_bucket.dotsdb_iceberg_data.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "dotsdb_iceberg_encryption" {
  bucket = aws_s3_bucket.dotsdb_iceberg_data.bucket

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "aws:kms"
    }
  }
}

resource "aws_s3_bucket_versioning" "dotsdb_iceberg_versioning" {
  bucket = aws_s3_bucket.dotsdb_iceberg_data.id
  versioning_configuration {
    status = "Enabled"
  }
}


# Athena query output bucket
resource "aws_s3_bucket" "dotsdb_athena_bucket" {
  bucket = var.athena_output_s3_bucket
}

resource "aws_s3_bucket_acl" "dotsdb_athena_acl" {
  bucket = aws_s3_bucket.dotsdb_athena_bucket.id
  acl    = "private"
}

resource "aws_s3_bucket_public_access_block" "dotsdb_iceberg_athena_public_access" {
  bucket = aws_s3_bucket.dotsdb_athena_bucket.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "dotsdb_athena_encryption" {
  bucket = aws_s3_bucket.dotsdb_athena_bucket.bucket

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "aws:kms"
    }
  }
}

resource "aws_s3_bucket_versioning" "dotsdb_athena_versioning" {
  bucket = aws_s3_bucket.dotsdb_athena_bucket.id
  versioning_configuration {
    status = "Enabled"
  }
}

# dotsDB iceberg lambda zip
resource "aws_s3_bucket" "dotsdb_iceberg_lambda" {
  bucket = var.lambda_code_s3_bucket
}

resource "aws_s3_bucket_acl" "dotsdb_iceberg_lambda_acl" {
  bucket = aws_s3_bucket.dotsdb_iceberg_lambda.id
  acl    = "private"
}

resource "aws_s3_bucket_public_access_block" "dotsdb_iceberg_lambda_public_access" {
  bucket = aws_s3_bucket.dotsdb_iceberg_lambda.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "dotsdb_iceberg_lambda_encryption" {
  bucket = aws_s3_bucket.dotsdb_iceberg_lambda.bucket

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "aws:kms"
    }
  }
}

resource "aws_s3_bucket_versioning" "dotsdb_iceberg_lambda_versioning" {
  bucket = aws_s3_bucket.dotsdb_iceberg_lambda.id
  versioning_configuration {
    status = "Enabled"
  }
}

