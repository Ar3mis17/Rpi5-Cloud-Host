#!/bin/bash
# Install All Systemd Services and Timers for Ar3mis Cloud

set -euo pipefail

echo "Installing Ar3mis Cloud systemd services..."

# Make all scripts executable
chmod +x /opt/ar3mis-cloud/scripts/*.sh

# Copy service files to systemd
SERVICES=(
    "ar3mis-backup.service"
    "ar3mis-backup.timer"
    "ar3mis-health.service"
    "ar3mis-health.timer"
    "ar3mis-metrics.service"
    "ar3mis-metrics.timer"
    "ar3mis-harden.service"
)

for service in "${SERVICES[@]}"; do
    if [ -f "/opt/ar3mis-cloud/scripts/${service}" ]; then
        sudo cp "/opt/ar3mis-cloud/scripts/${service}" "/etc/systemd/system/${service}"
        echo "  Installed ${service}"
    fi
done

# Reload systemd
sudo systemctl daemon-reload

# Enable and start timers
echo ""
echo "Enabling timers..."

TIMERS=(
    "ar3mis-backup.timer"
    "ar3mis-health.timer"
    "ar3mis-metrics.timer"
)

for timer in "${TIMERS[@]}"; do
    sudo systemctl enable "${timer}"
    sudo systemctl start "${timer}"
    echo "  Enabled ${timer}"
done

# Enable hardening service (runs once)
sudo systemctl enable ar3mis-harden.service
echo "  Enabled ar3mis-harden.service"

echo ""
echo "========================================"
echo "Systemd Services Installed"
echo "========================================"
echo ""
echo "Active timers:"
systemctl list-timers --all | grep ar3mis

echo ""
echo "To run hardening now: sudo systemctl start ar3mis-harden.service"
echo "To run backup now: sudo systemctl start ar3mis-backup.service"
echo "To check health: sudo systemctl start ar3mis-health.service"