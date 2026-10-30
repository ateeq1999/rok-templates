# `terraform test`: applies envs/local to Floci, checks what it made, and
# destroys it. Run with Floci up (`docker compose up -d floci`).

run "apply_to_floci" {
  command = apply

  assert {
    condition     = output.bucket == "api-local-uploads"
    error_message = "the uploads bucket is not the one asked for"
  }

  assert {
    condition = alltrue([
      for name in ["JWT_SECRET", "TOTP_ENCRYPTION_KEY", "DATABASE_URL", "SMTP_URL"] :
      startswith(lookup(output.secret_arns, name, ""), "arn:aws:secretsmanager:")
    ])
    error_message = "a secret the API reads is missing"
  }

  assert {
    condition     = output.database_address != ""
    error_message = "the database has no address"
  }

  assert {
    condition     = length(output.mail_dns_records) == 4
    error_message = "the mail domain needs a verification TXT and three DKIM CNAMEs"
  }
}

# The secrets hold what the API needs, readable back from Secrets Manager.
run "secrets_are_readable" {
  command = apply

  module {
    source = "./tests/read"
  }

  variables {
    jwt_secret_arn = run.apply_to_floci.secret_arns["JWT_SECRET"]
    totp_key_arn   = run.apply_to_floci.secret_arns["TOTP_ENCRYPTION_KEY"]
    floci_endpoint = run.apply_to_floci.floci_endpoint
  }

  assert {
    condition     = length(output.jwt_secret) >= 32
    error_message = "JWT_SECRET is shorter than the API accepts"
  }

  assert {
    # Base64 of 32 bytes: 44 characters, one `=` of padding. (Terraform's
    # base64decode wants UTF-8, which random key bytes are not.)
    condition     = length(output.totp_key) == 44 && endswith(output.totp_key, "=") && !endswith(output.totp_key, "==")
    error_message = "TOTP_ENCRYPTION_KEY is not 32 bytes in base64"
  }
}
