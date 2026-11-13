#!/bin/bash
# PostgreSQL Logical Backup Script for Ar3mis Cloud
# Run via cron or systemd timer

set -euo pipefail

# Configuration
BACKUP_DIR="/mnt/cloud/backups/postgres"
RETENTION_DAYS=30
DB_NAME="ar3mis_cloud"
DB_USER="ar3mis_app"
DB_HOST="localhost"
DB_PORT="5432"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/ar3mis_cloud_${TIMESTAMP}.sql.gz"
LATEST_LINK="${BACKUP_DIR}/latest.sql.gz"

# Load database password from .env
if [ -f /opt/ar3mis-cloud/backend/.env ]; then
    export $(grep -v '^#' /opt/ar3mis-cloud/backend/.env | xargs)
fi

# Create backup directory
mkdir -p "${BACKUP_DIR}"

echo "Starting PostgreSQL backup at $(date)"

# Perform logical backup with pg_dump via Docker
docker exec ar3mis-cloud-db pg_dump \
    -U "${DB_USER}" \
    -d "${DB_NAME}" \
    --no-owner \
    --no-privileges \
    --clean \
    --if-exists \
    | gzip > "${BACKUP_FILE}"

if [ $? -eq 0 ]; then
    echo "Backup completed: ${BACKUP_FILE}"

    # Update latest symlink
    ln -sf "${BACKUP_FILE}" "${LATEST_LINK}"

    # Verify backup
    if gunzip -t "${BACKUP_FILE}" 2>/dev/null; then
        echo "Backup verification: PASSED"
    else
        echo "Backup verification: FAILED"
        exit 1
    fi

    # Cleanup old backups
    find "${BACKUP_DIR}" -name "ar3mis_cloud_*.sql.gz" -mtime +${RETENTION_DAYS} -delete
    echo "Cleaned up backups older than ${RETENTION_DAYS} days"

    # Log backup size
    BACKUP_SIZE=$(du -h "${BACKUP_FILE}" | cut -f1)
    echo "Backup size: ${BACKUP_SIZE}"

else
    echo "Backup FAILED"
    rm -f "${BACKUP_FILE}"
    exit 1
fi

echo "Backup finished at $(date)"