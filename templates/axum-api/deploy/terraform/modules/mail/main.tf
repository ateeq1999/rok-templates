# Mail through SES: the sending domain's identity and DKIM, and SMTP
# credentials for MAIL_TRANSPORT=smtp. Publish the DNS records the outputs
# list (the verification TXT and three DKIM CNAMEs), and ask AWS to take the
# account out of the SES sandbox before mailing anyone but verified
# addresses.

terraform {
  required_version = ">= 1.10"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.71"
    }
  }
}

variable "name" {
  description = "Names the SMTP user, such as `shop-prod`."
  type        = string
}

variable "domain" {
  description = "The domain mail is sent from (MAIL_FROM's)."
  type        = string
}

data "aws_region" "current" {}

resource "aws_ses_domain_identity" "this" {
  domain = var.domain
}

resource "aws_ses_domain_dkim" "this" {
  domain = aws_ses_domain_identity.this.domain
}

resource "aws_iam_user" "smtp" {
  name = "${var.name}-smtp"
}

resource "aws_iam_user_policy" "smtp" {
  name = "send-mail"
  user = aws_iam_user.smtp.name
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect   = "Allow"
      Action   = ["ses:SendRawEmail"]
      Resource = "*"
    }]
  })
}

resource "aws_iam_access_key" "smtp" {
  user = aws_iam_user.smtp.name
}

output "smtp_url" {
  description = "SMTP_URL: SES's SMTP endpoint over TLS, with the credentials."
  sensitive   = true
  value = format(
    "smtps://%s:%s@email-smtp.%s.amazonaws.com:465",
    urlencode(aws_iam_access_key.smtp.id),
    urlencode(aws_iam_access_key.smtp.ses_smtp_password_v4),
    data.aws_region.current.region,
  )
}

output "dns_records" {
  description = "The DNS records the domain needs: the verification TXT and the DKIM CNAMEs."
  value = concat(
    [{ type = "TXT", name = "_amazonses.${var.domain}", value = aws_ses_domain_identity.this.verification_token }],
    [for token in aws_ses_domain_dkim.this.dkim_tokens : {
      type  = "CNAME"
      name  = "${token}._domainkey.${var.domain}"
      value = "${token}.dkim.amazonses.com"
    }],
  )
}
