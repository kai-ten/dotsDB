resource "aws_athena_workgroup" "dotDB_Athena_Workgroup" {
  name = "dotsDB"

  configuration {
    enforce_workgroup_configuration    = true
    publish_cloudwatch_metrics_enabled = true

    result_configuration {
      output_location = "s3://${aws_s3_bucket.dotsdb_athena_bucket.bucket}/results"

      ## TODO: Create custom key for Athena bucket
      #   encryption_configuration {
      #     encryption_option = "SSE_KMS"
      #   }
    }
  }
}

