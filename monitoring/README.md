# Monitoring Setup with Loki Integration

This directory contains the complete monitoring stack configuration for the Qdrant AI Search platform, including **Loki log aggregation** integration.

## Architecture

The monitoring stack consists of:

- **Prometheus**: Metrics collection and storage
- **Grafana**: Visualization dashboards and alerting
- **Loki**: Log aggregation and querying
- **Promtail**: Log forwarding agent

## Directory Structure

```
monitoring/
├── loki/
│   └── loki.yml                    # Loki configuration
├── promtail/
│   └── config.yml                  # Promtail configuration for Docker Compose
├── prometheus/
│   └── prometheus.yml              # Prometheus scrape configuration
└── grafana/
    ├── dashboards/
    │   ├── qdrant-ai-dashboard.json      # Main metrics dashboard
    │   └── logs-dashboard.json           # Loki logs dashboard
    └── provisioning/
        ├── datasources/
        │   └── datasources.yml           # Auto-provision Prometheus + Loki
        └── dashboards/
            └── dashboards.yml            # Dashboard provisioning config
```

## Deployment Options

### Option 1: Kubernetes (Production)

Deploy the complete monitoring stack to Kubernetes:

```bash
# Deploy individual components
make deploy-prometheus    # Metrics collection
make deploy-grafana      # Dashboards and visualization
make deploy-loki         # Log aggregation
make deploy-promtail     # Log forwarding (DaemonSet)

# Or deploy everything at once
make deploy-monitoring   # All monitoring components
```

**Access URLs (after port-forwarding):**

- Grafana: http://localhost:3000 (admin/admin)
- Prometheus: http://localhost:9090
- Loki: http://localhost:3100

### Option 2: Docker Compose (Development)

For local development with the complete monitoring stack:

```bash
# Full stack including monitoring
docker-compose -f docker-compose.monitoring.yml up -d

# Your application services + monitoring
docker-compose -f docker-compose.dev.yml -f docker-compose.monitoring.yml up -d
```

## 📊 Features

### Structured Logging

The application automatically detects Kubernetes environment and switches to JSON logging:

- **Development**: Human-readable console logs
- **Kubernetes**: Structured JSON logs for Loki

### Log Queries in Grafana

Example LogQL queries you can use:

```logql
# All backend logs
{service="qdrant-backend"}

# Error logs from all services
{level="ERROR"}

# Search-related logs
{service="qdrant-backend"} |= "search"

# Request rate by service
rate({service="qdrant-backend"}[1m])

# Log volume by level
sum by (level) (rate({service="qdrant-backend"}[1m]))
```

### Dashboards

1. **Main Metrics Dashboard**: Application performance, request rates, response times
2. **Logs Dashboard**: Log rates, error tracking, live log streaming

## Configuration

### Loki Retention

Logs are retained for 200h by default. Modify in `loki.yml`:

```yaml
limits_config:
  retention_period: 200h
```

### Log Labels

Automatic labels applied to all logs:

- `service`: Application service name (backend, rust_accelerator, etc.)
- `level`: Log level (INFO, ERROR, WARNING, DEBUG)
- `logger`: Logger name
- `module`: Source code module

### Custom Log Fields

Add custom fields to your logs:

```python
logger.info("User search completed", extra={
    "user_id": "12345",
    "query": "machine learning",
    "results_count": 42,
    "duration_ms": 150
})
```

These will appear as searchable fields in Loki.

## Troubleshooting

### Logs Not Appearing in Grafana

1. Check Loki is running: `kubectl get pods -n qdrant-ai`
2. Verify Promtail is collecting logs: `kubectl logs -n qdrant-ai daemonset/promtail`
3. Test Loki directly: `curl http://localhost:3100/ready`

### Grafana Can't Connect to Loki

1. Verify service URLs in datasource configuration
2. Check network connectivity between pods
3. Restart Grafana: `make restart-grafana`

### High Memory Usage

Loki memory usage grows with log volume. Adjust resources in `helm/loki/values.yaml`:

```yaml
resources:
  limits:
    memory: "2Gi" # Increase if needed
```

## Log Best Practices

1. **Use structured logging**: JSON format with consistent field names
2. **Include correlation IDs**: Track requests across services
3. **Log at appropriate levels**: INFO for business events, DEBUG for detailed tracing
4. **Avoid logging sensitive data**: Passwords, API keys, personal information
5. **Include context**: User IDs, request IDs, timestamps

Example good log entry:

```json
{
  "timestamp": "2025-01-20T10:30:00Z",
  "level": "INFO",
  "service": "qdrant-backend",
  "message": "Search completed successfully",
  "request_id": "req-12345",
  "user_id": "user-789",
  "query": "machine learning",
  "results_count": 15,
  "duration_ms": 245
}
```
