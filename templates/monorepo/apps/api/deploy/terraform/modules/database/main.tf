# The PostgreSQL database: RDS, encrypted, backed up, its password
# generated and handed on only inside DATABASE_URL (a secret).

terraform {
  required_version = ">= 1.10"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.71"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.7"
    }
  }
}

variable "name" {
  description = "The instance's identifier, such as `shop-prod`."
  type        = string
}

variable "instance_class" {
  description = "The instance size."
  type        = string
  default     = "db.t4g.micro"
}

variable "allocated_storage" {
  description = "Storage, in GiB."
  type        = number
  default     = 20
}

variable "multi_az" {
  description = "A standby in a second availability zone (production)."
  type        = bool
  default     = false
}

variable "subnet_ids" {
  description = "The private subnets it lives in; none, the default VPC's (local only)."
  type        = list(string)
  default     = []
}

variable "security_group_ids" {
  description = "Who may connect: the network module's database group."
  type        = list(string)
  default     = []
}

variable "backup_retention_period" {
  description = "Days of automated backups."
  type        = number
  default     = 7
}

variable "deletion_protection" {
  description = "Refuse `terraform destroy` (production)."
  type        = bool
  default     = true
}

resource "random_password" "database" {
  length  = 32
  special = false
}

resource "aws_db_subnet_group" "this" {
  count      = length(var.subnet_ids) > 0 ? 1 : 0
  name       = var.name
  subnet_ids = var.subnet_ids
}

resource "aws_db_instance" "this" {
  identifier                = var.name
  engine                    = "postgres"
  engine_version            = "17"
  instance_class            = var.instance_class
  allocated_storage         = var.allocated_storage
  storage_encrypted         = true
  db_name                   = "app"
  username                  = "app"
  password                  = random_password.database.result
  multi_az                  = var.multi_az
  db_subnet_group_name      = length(var.subnet_ids) > 0 ? aws_db_subnet_group.this[0].name : null
  vpc_security_group_ids    = var.security_group_ids
  backup_retention_period   = var.backup_retention_period
  deletion_protection       = var.deletion_protection
  skip_final_snapshot       = !var.deletion_protection
  final_snapshot_identifier = var.deletion_protection ? "${var.name}-final" : null
  apply_immediately         = !var.deletion_protection
}

output "address" {
  description = "The database's host name."
  value       = aws_db_instance.this.address
}

output "port" {
  description = "The database's port."
  value       = aws_db_instance.this.port
}

output "database_url" {
  description = "DATABASE_URL, with the generated password."
  sensitive   = true
  value       = "postgres://app:${random_password.database.result}@${aws_db_instance.this.address}:${aws_db_instance.this.port}/app"
}
