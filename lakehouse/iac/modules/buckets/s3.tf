resource "aws_s3_bucket" "dotsdb_bucket" {
  bucket = var.bucket_name
}

resource "aws_s3_bucket_acl" "dotsdb_bucket_acl" {
  bucket = aws_s3_bucket.dotsdb_bucket.id
  acl    = "private"
}

resource "aws_s3_bucket_public_access_block" "dotsdb_bucket_public_access" {
  bucket = aws_s3_bucket.dotsdb_bucket.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "dotsdb_bucket_encryption" {
  bucket = aws_s3_bucket.dotsdb_bucket.bucket

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "aws:kms"
    }
  }
}

resource "aws_s3_bucket_versioning" "dotsdb_bucket_versioning" {
  bucket = aws_s3_bucket.dotsdb_bucket.id
  versioning_configuration {
    status = "Enabled"
  }
}

output "data" {
  value = aws_s3_bucket.dotsdb_bucket
}
