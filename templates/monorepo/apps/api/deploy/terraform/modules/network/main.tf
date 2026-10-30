# The network: a VPC over two availability zones, public subnets for the
# load balancer and the NAT gateway, private subnets for the tasks and the
# database, and the three security groups: the load balancer's (80, 443
# from anywhere), the tasks' (the API port from the load balancer only),
# the database's (5432 from the tasks only).

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

variable "cidr" {
  description = "The VPC's address range."
  type        = string
  default     = "10.0.0.0/16"
}

variable "app_port" {
  description = "The port the API listens on (BIND_ADDR)."
  type        = number
  default     = 8080
}

data "aws_availability_zones" "available" {
  state = "available"
}

locals {
  zones = slice(data.aws_availability_zones.available.names, 0, 2)
}

resource "aws_vpc" "this" {
  cidr_block           = var.cidr
  enable_dns_hostnames = true
  tags                 = { Name = var.name }
}

resource "aws_internet_gateway" "this" {
  vpc_id = aws_vpc.this.id
  tags   = { Name = var.name }
}

resource "aws_subnet" "public" {
  count                   = length(local.zones)
  vpc_id                  = aws_vpc.this.id
  availability_zone       = local.zones[count.index]
  cidr_block              = cidrsubnet(var.cidr, 8, count.index)
  map_public_ip_on_launch = true
  tags                    = { Name = "${var.name}-public-${local.zones[count.index]}" }
}

resource "aws_subnet" "private" {
  count             = length(local.zones)
  vpc_id            = aws_vpc.this.id
  availability_zone = local.zones[count.index]
  cidr_block        = cidrsubnet(var.cidr, 8, count.index + 10)
  tags              = { Name = "${var.name}-private-${local.zones[count.index]}" }
}

resource "aws_eip" "nat" {
  domain = "vpc"
  tags   = { Name = "${var.name}-nat" }
}

# One NAT gateway: the tasks reach ECR, SES and the OTLP collector through
# it. One per zone survives a zone's outage, at three times the cost.
resource "aws_nat_gateway" "this" {
  allocation_id = aws_eip.nat.id
  subnet_id     = aws_subnet.public[0].id
  tags          = { Name = var.name }
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.this.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.this.id
  }
}

resource "aws_route_table" "private" {
  vpc_id = aws_vpc.this.id
  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.this.id
  }
}

resource "aws_route_table_association" "public" {
  count          = length(aws_subnet.public)
  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

resource "aws_route_table_association" "private" {
  count          = length(aws_subnet.private)
  subnet_id      = aws_subnet.private[count.index].id
  route_table_id = aws_route_table.private.id
}

resource "aws_security_group" "load_balancer" {
  name   = "${var.name}-lb"
  vpc_id = aws_vpc.this.id
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "app" {
  name   = "${var.name}-app"
  vpc_id = aws_vpc.this.id
  ingress {
    from_port       = var.app_port
    to_port         = var.app_port
    protocol        = "tcp"
    security_groups = [aws_security_group.load_balancer.id]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "database" {
  name   = "${var.name}-db"
  vpc_id = aws_vpc.this.id
  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.app.id]
  }
}

output "vpc_id" {
  description = "The VPC."
  value       = aws_vpc.this.id
}

output "public_subnet_ids" {
  description = "The load balancer's subnets."
  value       = aws_subnet.public[*].id
}

output "private_subnet_ids" {
  description = "The tasks' and the database's subnets."
  value       = aws_subnet.private[*].id
}

output "load_balancer_security_group_id" {
  description = "The load balancer's security group."
  value       = aws_security_group.load_balancer.id
}

output "app_security_group_id" {
  description = "The tasks' security group."
  value       = aws_security_group.app.id
}

output "database_security_group_id" {
  description = "The database's security group."
  value       = aws_security_group.database.id
}
