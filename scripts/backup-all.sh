#!/bin/bash
# Master Backup Script - Runs all backup scripts

set -euo pipefail

LOG_FILE="/mnt/cloud/backups/backup-all.log"
TIMESTAMP=$(date +'%Y-%m-%d %H:%M:%S')

{
    echo "========================================"
    echo "Ar3mis Cloud - Full Backup Started: ${TIMESTAMP}"
    echo "========================================"

    # Run PostgreSQL backup
    echo ""
    echo "[1/2] PostgreSQL Backup"
    /opt/ar3mis-cloud/scripts/backup-postgres.sh

    # Run file storage backup
    echo ""
    echo "[2/2] File Storage Backup"
    /opt/ar3mis-cloud/scripts/backup-files.sh

    echo ""
    echo "========================================"
    echo "Full Backup Completed: $(date +'%Y-%m-%d %H:%M:%S')"
    echo "========================================"
} >> "${LOG_FILE}" 2>&1

# Also output to stdout
cat "${LOG_FILE}" | tail -20