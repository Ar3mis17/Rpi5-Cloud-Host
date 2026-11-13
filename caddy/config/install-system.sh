#!/bin/bash
# Caddy System-Wide Installation Script
# Run with: sudo bash /opt/ar3mis-cloud/caddy/config/install-system.sh

set -euo pipefail

echo "Installing Caddy system-wide..."

# 1. Install Caddy binary
cp /home/ar3mis/.local/bin/caddy /usr/bin/caddy
chmod +x /usr/bin/caddy

# 2. Set capabilities to bind to ports 80/443
setcap 'cap_net_bind_service=+ep' /usr/bin/caddy

# 3. Create system directories
mkdir -p /etc/caddy /var/caddy /var/log/caddy /var/www/html

# 4. Copy configuration
cp /opt/ar3mis-cloud/caddy/config/Caddyfile /etc/caddy/Caddyfile

# 5. Copy systemd service
cp /opt/ar3mis-cloud/caddy/config/caddy-pihole.service /etc/systemd/system/caddy-pihole.service

# 6. Reload systemd and enable
systemctl daemon-reload
systemctl enable caddy-pihole
systemctl restart caddy-pihole

# 7. Verify
sleep 2
systemctl status caddy-pihole --no-pager

echo ""
echo "Installation complete!"
echo "Access Ar3mis Cloud API: https://cloud.ar3mis.com/api/v1"
echo "Access Pi-hole Admin:    https://cloud.ar3mis.com/admin"