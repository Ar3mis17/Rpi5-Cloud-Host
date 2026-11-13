#!/bin/bash
# Metrics Collection Script for Ar3mis Cloud
# Outputs Prometheus-compatible metrics

set -euo pipefail

METRICS_FILE="/var/lib/prometheus/ar3mis-cloud.prom"
TMP_FILE="${METRICS_FILE}.tmp"

mkdir -p "$(dirname "${METRICS_FILE}")"

# Collect metrics
API_RESPONSE=$(curl -s -f --max-time 5 "http://127.0.0.1:8000/health" 2>/dev/null || echo "")
DB_RESPONSE=$(curl -s -f --max-time 5 "http://127.0.0.1:8000/health" 2>/dev/null || echo "")

# API health
if echo "${API_RESPONSE}" | grep -q '"status":"healthy"'; then
    API_UP=1
else
    API_UP=0
fi

# Database health
if echo "${DB_RESPONSE}" | grep -q '"database":"connected"'; then
    DB_UP=1
else
    DB_UP=0
fi

# Disk usage
DISK_USAGE=$(df /mnt/cloud | awk 'NR==2 {print $5}' | sed 's/%//')
DISK_AVAILABLE=$(df /mnt/cloud | awk 'NR==2 {print $4}')

# Memory usage
MEM_USAGE=$(free | awk 'NR==2 {printf "%.0f", $3/$2 * 100}')
MEM_TOTAL=$(free -m | awk 'NR==2 {print $2}')
MEM_USED=$(free -m | awk 'NR==2 {print $3}')

# Docker containers
DOCKER_RUNNING=$(docker ps --filter "name=ar3mis-cloud" --format "{{.Names}}" | wc -l)

# Backup status (time since last backup)
LAST_BACKUP=0
if [ -f "/mnt/cloud/backups/postgres/latest.sql.gz" ]; then
    LAST_BACKUP=$(stat -c %Y "/mnt/cloud/backups/postgres/latest.sql.gz")
fi
BACKUP_AGE=$(( $(date +%s) - LAST_BACKUP ))

# Write metrics
cat > "${TMP_FILE}" << EOF
# HELP ar3mis_api_up API health status (1=healthy, 0=unhealthy)
# TYPE ar3mis_api_up gauge
ar3mis_api_up ${API_UP}

# HELP ar3mis_database_up Database connection status (1=connected, 0=disconnected)
# TYPE ar3mis_database_up gauge
ar3mis_database_up ${DB_UP}

# HELP ar3mis_disk_usage_percent Disk usage percentage
# TYPE ar3mis_disk_usage_percent gauge
ar3mis_disk_usage_percent ${DISK_USAGE}

# HELP ar3mis_disk_available_kb Disk available space in KB
# TYPE ar3mis_disk_available_kb gauge
ar3mis_disk_available_kb ${DISK_AVAILABLE}

# HELP ar3mis_memory_usage_percent Memory usage percentage
# TYPE ar3mis_memory_usage_percent gauge
ar3mis_memory_usage_percent ${MEM_USAGE}

# HELP ar3mis_memory_total_mb Total memory in MB
# TYPE ar3mis_memory_total_mb gauge
ar3mis_memory_total_mb ${MEM_TOTAL}

# HELP ar3mis_memory_used_mb Used memory in MB
# TYPE ar3mis_memory_used_mb gauge
ar3mis_memory_used_mb ${MEM_USED}

# HELP ar3mis_docker_containers_running Number of running Ar3mis containers
# TYPE ar3mis_docker_containers_running gauge
ar3mis_docker_containers_running ${DOCKER_RUNNING}

# HELP ar3mis_backup_age_seconds Seconds since last successful backup
# TYPE ar3mis_backup_age_seconds gauge
ar3mis_backup_age_seconds ${BACKUP_AGE}

# HELP ar3mis_backup_timestamp Unix timestamp of last backup
# TYPE ar3mis_backup_timestamp gauge
ar3mis_backup_timestamp ${LAST_BACKUP}
EOF

mv "${TMP_FILE}" "${METRICS_FILE}"