# The whole deployment, one environment of it: the network, the database,
# the uploads bucket, mail, the secrets and the service, wired together.
# `envs/staging` and `envs/prod` call it with their own sizes.

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
  description = "Names every resource, such as `shop-prod`."
  type        = string
}

variable "image_tag" {
  description = "The image tag to run (the server as <tag>, the migrations as <tag>-migrate)."
  type        = string
}

variable "app_url" {
  description = "APP_URL: where users reach the front end (mailed links), https:// in production."
  type        = string
}

variable "mail_domain" {
  description = "The domain mail is sent from."
  type        = string
}

variable "mail_from" {
  description = "MAIL_FROM, such as `Shop <no-reply@shop.example>`."
  type        = string
}

variable "cors_allowed_origins" {
  description = "The front ends calling the API from another origin (CORS_ALLOWED_ORIGINS, and the bucket's CORS)."
  type        = list(string)
  default     = []
}

variable "certificate_arn" {
  description = "An ACM certificate for HTTPS; none, HTTP only."
  type        = string
  default     = null
}

variable "environment" {
  description = "Further settings for the API (OTEL_EXPORTER_OTLP_ENDPOINT, GLOBAL_RATE_LIMIT …)."
  type        = map(string)
  default     = {}
}

variable "database" {
  description = "The database's size and safety."
  type = object({
    instance_class      = string
    allocated_storage   = number
    multi_az            = bool
    deletion_protection = bool
  })
}

variable "service" {
  description = "The tasks' size and count."
  type = object({
    cpu          = number
    memory       = number
    web_count    = number
    worker_count = number
  })
}

module "network" {
  source = "../network"
  name   = var.name
}

module "database" {
  source              = "../database"
  name                = var.name
  instance_class      = var.database.instance_class
  allocated_storage   = var.database.allocated_storage
  multi_az            = var.database.multi_az
  deletion_protection = var.database.deletion_protection
  subnet_ids          = module.network.private_subnet_ids
  security_group_ids  = [module.network.database_security_group_id]
}

module "storage" {
  source               = "../storage"
  bucket               = "${var.name}-uploads"
  cors_allowed_origins = var.cors_allowed_origins
}

module "mail" {
  source = "../mail"
  name   = var.name
  domain = var.mail_domain
}

module "secrets" {
  source = "../secrets"
  prefix = "${var.name}/"
  names  = ["DATABASE_URL", "SMTP_URL"]
  values = {
    DATABASE_URL = module.database.database_url
    SMTP_URL     = module.mail.smtp_url
  }
}

module "service" {
  source          = "../service"
  name            = var.name
  image_tag       = var.image_tag
  certificate_arn = var.certificate_arn
  cpu             = var.service.cpu
  memory          = var.service.memory
  web_count       = var.service.web_count
  worker_count    = var.service.worker_count
  secret_arns     = module.secrets.arns
  bucket_arn      = module.storage.arn
  environment = merge({
    APP_URL              = var.app_url
    MAIL_TRANSPORT       = "smtp"
    MAIL_FROM            = var.mail_from
    STORAGE_URL          = "s3://${module.storage.bucket}"
    TRUST_PROXY          = "true"
    CORS_ALLOWED_ORIGINS = join(",", var.cors_allowed_origins)
  }, var.environment)
  vpc_id                          = module.network.vpc_id
  public_subnet_ids               = module.network.public_subnet_ids
  private_subnet_ids              = module.network.private_subnet_ids
  load_balancer_security_group_id = module.network.load_balancer_security_group_id
  app_security_group_id           = module.network.app_security_group_id
}

output "url" {
  description = "Where the API answers."
  value       = module.service.url
}

output "repository_url" {
  description = "Push the images here."
  value       = module.service.repository_url
}

output "migrate_command" {
  description = "Runs the migrations once before each deploy."
  value       = module.service.migrate_command
}

output "mail_dns_records" {
  description = "The DNS records the mail domain needs."
  value       = module.mail.dns_records
}
