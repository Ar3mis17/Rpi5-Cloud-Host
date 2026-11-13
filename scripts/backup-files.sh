#!/bin/bash
# File Storage Backup Script for Ar3mis Cloud
# Backs up user files to compressed archive

set -euo pipefail

# Configuration
BACKUP_DIR="/mnt/cloud/backups/files"
FILES_ROOT="/mnt/cloud/ar3mis-cloud/files"
RETENTION_DAYS=14
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/files_${TIMESTAMP}.tar.gz"
LATEST_LINK="${BACKUP_DIR}/latest.tar.gz"

# Create backup directory
mkdir -p "${BACKUP_DIR}"

echo "Starting file storage backup at $(date)"

# Check if files directory exists
if [ ! -d "${FILES_ROOT}" ]; then
    echo "Files directory not found: ${FILES_ROOT}"
    exit 1
fi

# Create tar.gz backup
tar -czf "${BACKUP_FILE}" -C "${FILES_ROOT}" .

if [ $? -eq 0 ]; then
    echo "Backup completed: ${BACKUP_FILE}"

    # Update latest symlink
    ln -sf "${BACKUP_FILE}" "${LATEST_LINK}"

    # Verify backup
    if tar -tzf "${BACKUP_FILE}" >/dev/null 2>&1; then
        echo "Backup verification: PASSED"
    else
        echo "Backup verification: FAILED"
        exit 1
    fi

    # Cleanup old backups
    find "${BACKUP_DIR}" -name "files_*.tar.gz" -mtime +${RETENTION_DAYS} -delete
    echo "Cleaned up backups older than ${RETENTION_DAYS} days"

    # Log backup size
    BACKUP_SIZE=$(du -h "${BACKUP_FILE}" | cut -f1)
    echo "Backup size: ${BACKUP_SIZE}"

else
    echo "Backup FAILED"
    rm -f "${BACKUP_FILE}"
    exit 1
fi

echo "File backup finished at $(date)"