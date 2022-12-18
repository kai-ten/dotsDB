data_s3_bucket          = "<your_data_bucket_name_here>"
athena_output_s3_bucket = "<your_athena_bucket_name_here>"
lambda_code_s3_bucket   = "<your_code_bucket_name_here>"

# If false, then API GW will NOT deploy. If true, API GW will deploy. 
# WARNING: DEPLOYING APIGW CREATES A PUBLIC ENDPOINT. You deploy this at your own risk!
deploy_apigw            = false