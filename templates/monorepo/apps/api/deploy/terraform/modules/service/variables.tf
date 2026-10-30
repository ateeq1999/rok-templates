variable "name" {
  description = "Names every resource, such as `shop-prod`."
  type        = string
}

variable "image_tag" {
  description = "The tag to run, pushed to this module's ECR repository: the server as <tag>, the migrations as <tag>-migrate."
  type        = string
}

variable "app_port" {
  description = "The port the API listens on (BIND_ADDR)."
  type        = number
  default     = 8080
}

variable "cpu" {
  description = "A web or worker task's CPU units (1024 is one vCPU)."
  type        = number
  default     = 512
}

variable "memory" {
  description = "A web or worker task's memory, in MiB."
  type        = number
  default     = 1024
}

variable "web_count" {
  description = "How many web tasks run."
  type        = number
  default     = 2
}

variable "worker_count" {
  description = "How many worker tasks run."
  type        = number
  default     = 1
}

variable "environment" {
  description = "The API's settings that are not secrets (APP_URL, MAIL_FROM, STORAGE_URL …)."
  type        = map(string)
  default     = {}
}

variable "secret_arns" {
  description = "Each secret variable's Secrets Manager ARN, from the secrets module."
  type        = map(string)
}

variable "bucket_arn" {
  description = "The uploads bucket the tasks may read and write."
  type        = string
}

variable "certificate_arn" {
  description = "An ACM certificate for the load balancer's HTTPS listener; none, HTTP only."
  type        = string
  default     = null
}

variable "log_retention_days" {
  description = "How long CloudWatch keeps the logs."
  type        = number
  default     = 30
}

variable "vpc_id" {
  description = "The network module's VPC."
  type        = string
}

variable "public_subnet_ids" {
  description = "The load balancer's subnets."
  type        = list(string)
}

variable "private_subnet_ids" {
  description = "The tasks' subnets."
  type        = list(string)
}

variable "load_balancer_security_group_id" {
  description = "The load balancer's security group."
  type        = string
}

variable "app_security_group_id" {
  description = "The tasks' security group."
  type        = string
}
