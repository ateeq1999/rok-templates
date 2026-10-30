output "url" {
  description = "Where the API answers: the load balancer's address."
  value       = "${var.certificate_arn == null ? "http" : "https"}://${aws_lb.this.dns_name}"
}

output "repository_url" {
  description = "Push the image here: the server as :<tag>, the migrations as :<tag>-migrate."
  value       = aws_ecr_repository.this.repository_url
}

output "cluster" {
  description = "The ECS cluster."
  value       = aws_ecs_cluster.this.name
}

output "migrate_command" {
  description = "Runs the migrations once, before the new version starts; wait for it to stop with exit code 0."
  value = join(" ", [
    "aws ecs run-task",
    "--cluster ${aws_ecs_cluster.this.name}",
    "--task-definition ${aws_ecs_task_definition.this["migrate"].family}",
    "--launch-type FARGATE",
    "--network-configuration 'awsvpcConfiguration={subnets=[${join(",", var.private_subnet_ids)}],securityGroups=[${var.app_security_group_id}]}'",
  ])
}
