# The API on ECS Fargate: one image (ECR), two services from it, `web`
# (APP_ROLE=web, behind the load balancer, which checks /health/ready) and
# `worker` (APP_ROLE=worker, the jobs and the schedule), and the `migrate`
# task definition (the image's `migrate` target), which the deploy runs
# once before the new version starts. Settings come as environment
# variables, secrets from Secrets Manager.

terraform {
  required_version = ">= 1.10"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.71"
    }
  }
}

data "aws_region" "current" {}

locals {
  region = data.aws_region.current.region
  # The ECS stop timeout covers the drain: SHUTDOWN_DELAY_SECS plus
  # SHUTDOWN_GRACE_SECS (5 + 25), and a margin.
  stop_timeout = 40
  environment = [for name, value in merge(var.environment, {
    BIND_ADDR           = "0.0.0.0:${var.app_port}"
    LOG_FORMAT          = "json"
    SHUTDOWN_DELAY_SECS = "5"
    SHUTDOWN_GRACE_SECS = "25"
  }) : { name = name, value = value }]
  secrets = [for name, arn in var.secret_arns : { name = name, valueFrom = arn }]
  image   = "${aws_ecr_repository.this.repository_url}:${var.image_tag}"
}

resource "aws_ecr_repository" "this" {
  name                 = var.name
  image_tag_mutability = "IMMUTABLE"
  image_scanning_configuration {
    scan_on_push = true
  }
}

resource "aws_ecs_cluster" "this" {
  name = var.name
  setting {
    name  = "containerInsights"
    value = "enabled"
  }
}

resource "aws_cloudwatch_log_group" "this" {
  name              = "/ecs/${var.name}"
  retention_in_days = var.log_retention_days
}

# What ECS itself does for a task: pull the image, write the logs, read the
# secrets into the environment.
resource "aws_iam_role" "execution" {
  name = "${var.name}-execution"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "ecs-tasks.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy_attachment" "execution" {
  role       = aws_iam_role.execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

resource "aws_iam_role_policy" "execution_secrets" {
  name = "read-secrets"
  role = aws_iam_role.execution.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect   = "Allow"
      Action   = ["secretsmanager:GetSecretValue"]
      Resource = values(var.secret_arns)
    }]
  })
}

# What the API does: the uploads bucket, and nothing else.
resource "aws_iam_role" "task" {
  name = "${var.name}-task"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "ecs-tasks.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy" "task_storage" {
  name = "uploads"
  role = aws_iam_role.task.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["s3:GetObject", "s3:PutObject", "s3:DeleteObject"]
        Resource = "${var.bucket_arn}/*"
      },
      {
        Effect   = "Allow"
        Action   = ["s3:ListBucket"]
        Resource = var.bucket_arn
      },
    ]
  })
}

resource "aws_lb" "this" {
  name               = var.name
  load_balancer_type = "application"
  subnets            = var.public_subnet_ids
  security_groups    = [var.load_balancer_security_group_id]
}

resource "aws_lb_target_group" "web" {
  name        = "${var.name}-web"
  port        = var.app_port
  protocol    = "HTTP"
  target_type = "ip"
  vpc_id      = var.vpc_id
  # Past SHUTDOWN_DELAY_SECS the task stops accepting: the load balancer
  # has seen /health/ready answer 503 by then.
  deregistration_delay = 30
  health_check {
    path                = "/health/ready"
    interval            = 5
    timeout             = 3
    healthy_threshold   = 2
    unhealthy_threshold = 2
  }
}

resource "aws_lb_listener" "http" {
  load_balancer_arn = aws_lb.this.arn
  port              = 80
  protocol          = "HTTP"
  default_action {
    type             = var.certificate_arn == null ? "forward" : "redirect"
    target_group_arn = var.certificate_arn == null ? aws_lb_target_group.web.arn : null
    dynamic "redirect" {
      for_each = var.certificate_arn == null ? [] : [1]
      content {
        port        = "443"
        protocol    = "HTTPS"
        status_code = "HTTP_301"
      }
    }
  }
}

resource "aws_lb_listener" "https" {
  count             = var.certificate_arn == null ? 0 : 1
  load_balancer_arn = aws_lb.this.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  certificate_arn   = var.certificate_arn
  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.web.arn
  }
}

resource "aws_ecs_task_definition" "this" {
  for_each                 = { web = "web", worker = "worker", migrate = "migrate" }
  family                   = "${var.name}-${each.key}"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = each.key == "migrate" ? 256 : var.cpu
  memory                   = each.key == "migrate" ? 512 : var.memory
  execution_role_arn       = aws_iam_role.execution.arn
  task_role_arn            = aws_iam_role.task.arn
  container_definitions = jsonencode([{
    name        = each.key
    image       = each.key == "migrate" ? "${local.image}-migrate" : local.image
    essential   = true
    stopTimeout = local.stop_timeout
    environment = concat(local.environment, [{ name = "APP_ROLE", value = each.key == "worker" ? "worker" : "web" }])
    secrets     = local.secrets
    portMappings = each.key == "migrate" ? [] : [{
      containerPort = var.app_port
      protocol      = "tcp"
    }]
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        awslogs-group         = aws_cloudwatch_log_group.this.name
        awslogs-region        = local.region
        awslogs-stream-prefix = each.key
      }
    }
  }])
}

resource "aws_ecs_service" "web" {
  name                              = "web"
  cluster                           = aws_ecs_cluster.this.id
  task_definition                   = aws_ecs_task_definition.this["web"].arn
  desired_count                     = var.web_count
  launch_type                       = "FARGATE"
  health_check_grace_period_seconds = 30
  network_configuration {
    subnets         = var.private_subnet_ids
    security_groups = [var.app_security_group_id]
  }
  load_balancer {
    target_group_arn = aws_lb_target_group.web.arn
    container_name   = "web"
    container_port   = var.app_port
  }
  deployment_circuit_breaker {
    enable   = true
    rollback = true
  }
  depends_on = [aws_lb_listener.http]
}

resource "aws_ecs_service" "worker" {
  name            = "worker"
  cluster         = aws_ecs_cluster.this.id
  task_definition = aws_ecs_task_definition.this["worker"].arn
  desired_count   = var.worker_count
  launch_type     = "FARGATE"
  network_configuration {
    subnets         = var.private_subnet_ids
    security_groups = [var.app_security_group_id]
  }
  deployment_circuit_breaker {
    enable   = true
    rollback = true
  }
}
