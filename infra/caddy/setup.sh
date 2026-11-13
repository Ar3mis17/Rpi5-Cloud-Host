#!/bin/bash
# Raspberry Pi 5 Setup for Ar3mis Cloud
# Caddy-based reverse proxy for external access

set -euo pipefail

# Configuration
RPI_IP="192.168.1.200"
DOMAIN="cloud.ar3mis.com"
EMAIL="admin@ar3mis.com"
BACKUP_DIR="/mnt/cloud/backups/caddy"
CERT_DIR="/etc/caddy"

# Create directories
mkdir -p "$BACKUP_DIR"
mkdir -p "$CERT_DIR"

# Install Caddy
if ! command -v caddy &> /dev/null; then
    echo "Installing Caddy..."
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy.gpg
    echo "deb [signed-by=/usr/share/keyrings/caddy.gpg] https://dl.cloudsmith.io/public/caddy/stable/deb/ ./" | tee /etc/apt/sources.list.d/caddy.list
    apt-get update
    apt-get install -y caddy
fi

# Configure Caddy
cat > /etc/caddy/Caddyfile << ENDCADDY
# Ar3mis Cloud - Reverse Proxy to Backend API
$DOMAIN {
    root * /var/www/html
    # API proxy
    @api {
        path /api/v1/*
    }
    reverse_proxy @api http://127.0.0.1:8000 {
        # Server settings
        transport dns {
            # Failover to local if primary fails
            primary_resolver 127.0.0.1
            secondary_resolver localhost
        }
        # Pass through headers
        header_up Host {host}
        header_up X-Forwarded-Proto https
        header_up X-Forwarded-Proto http
        # Timeout settings
        timeout 30s
        # Buffering
        buffering
        # Health checks
        health_check interval=30s
    }

    # Health check endpoint
    @health {
        path /health
    }
    reverse_proxy @health http://127.0.0.1:8000 {
        transport dns {
            primary_resolver 127.0.0.1
            secondary_resolver localhost
        }
    }

    # Static files (Vercel)
    import caddyhttp.responseheaders
    @static {
        path /static/*
    }
    reverse_proxy @static https://$DOMAIN {
        # Trusted proxy
        untrusted
    }

    # HTTPS configuration
    tls {
        automation off
        # Use ACME for HTTPS
        certificates /etc/caddy
    }

    # Logs
    log {
        level DEBUG
        output file /var/log/caddy.log
    }
}
ENDCADDY

# Set permissions
cat > /etc/systemd/system/caddy.service << EOF
[Unit]
Description=Caddy Reverse Proxy
After=network.target

[Service]
Type=notify
User=root
Group=root
ExecStart=/usr/bin/caddy run --config /etc/caddy/Caddyfile
Restart=on-failure
RestartSec=5
TimeoutStopSec=5
KillMode=control-group
LimitNOFILE=1048576

[Install]
WantedBy=multi-user.target
EOF

# Enable and start Caddy
systemctl daemon-reload
systemctl enable caddy
systemctl restart caddy

# Create backup script
cat > /opt/ar3mis-cloud/scripts/caddy-backup.sh << BACKUP
#!/bin/bash
BACKUP_DIR="/mnt/cloud/backups/caddy"
DATE=$(date +%Y%m%d_%H%M%S)

cd /etc/caddy
# Backup Caddy configuration
tar -czf "$BACKUP_DIR/caddy-config-$DATE.tar.gz" Caddyfile

# Backup Caddy logs if they exist
if [ -f /var/log/caddy.log ]; then
    tar -czf "$BACKUP_DIR/caddy-logs-$DATE.tar.gz" /var/log/caddy.log
fi

# Cleanup old backups (keep 7 days)
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +7 -delete
BACKUP

chmod +x /opt/ar3mis-cloud/scripts/caddy-backup.sh

# Create recovery script
cat > /opt/ar3mis-cloud/scripts/caddy-restore.sh << RESTORE
#!/bin/bash
BACKUP_DIR="/mnt/cloud/backups/caddy"

if [ $# -eq 0 ]; then
    echo "Usage: $0 <backup-date>"
    exit 1
fi

BACKUP_FILE="$BACKUP_DIR/caddy-config-$1.tar.gz"
if [ ! -f "$BACKUP_FILE" ]; then
    echo "Backup file not found: $BACKUP_FILE"
    exit 1
fi

tar -xzf "$BACKUP_FILE" -C /etc/caddy
systemctl restart caddy
RESTORE

chmod +x /opt/ar3mis-cloud/scripts/caddy-restore.sh

# Create systemd service for backups
HOURS="0 1 2 3 4 5 6"
for hour in $HOURS; do
    cat > /etc/systemd/system/caddy-backup-hourly@$hour.service << HOUREOF
[Unit]
Description=Caddy Backup - Hour $hour
After=network.target
Requires=caddy.service

[Service]
Type=oneshot
User=root
ExecStart=/opt/ar3mis-cloud/scripts/caddy-backup.sh

[Install]
WantedBy=multi-user.target
HOUREOF
    systemctl enable "caddy-backup-hourly@$hour.service"
    systemctl link "caddy-backup-hourly@$hour.service"
 done

# Create SSL backup service
cat > /etc/systemd/system/caddy-ssl-backup.service << SSLBACKUPEOF
[Unit]
Description=Caddy SSL Backup
After=caddy.service

[Service]
Type=oneshot
User=root
ExecStart=/bin/bash -c 'mkdir -p /etc/caddy/backup && cp -r /etc/caddy/* /etc/caddy/backup/'

[Install]
WantedBy=multi-user.target
SSLBACKUPEOF

systemctl daemon-reload

# Configure automatic SSL renewal (using ACME)
cat > /opt/ar3mis-cloud/scripts/ssl-renew.sh << RENEW
#!/bin/bash
set -euo pipefail

# Renew SSL certificates
caddy reload --config /etc/caddy/Caddyfile

# Verify SSL
SSL_INFO=$(openssl s_client -connect $DOMAIN:443 -servername $DOMAIN 2>/dev/null | openssl x509 -noout -dates)
if [ $? -eq 0 ]; then
    echo "SSL certificate renewed successfully"
    echo "SSL info: $SSL_INFO"
else
    echo "SSL renewal verification failed"
    exit 1
fi
RENEW

chmod +x /opt/ar3mis-cloud/scripts/ssl-renew.sh

# Create systemd service for SSL renewal
cat > /etc/systemd/system/caddy-ssl-renew.service << RENEWEOF
[Unit]
Description=Caddy SSL Certificate Renewal
After=caddy.service

[Service]
Type=oneshot
User=root
ExecStart=/opt/ar3mis-cloud/scripts/ssl-renew.sh

[Install]
WantedBy=multi-user.target
RENEWEOF

systemctl daemon-reload

# Enable SSL renewal weekly
for day in {0..6}; do
    systemctl enable "caddy-ssl-renew@$day.service"
    systemctl link "caddy-ssl-renew@$day.service"
 done

# Configure log rotation
cat > /etc/logrotate.d/caddy << LOGROTATE
/var/log/caddy.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 root root
    postrotate
        systemctl reload caddy
    endscript
}
LOGROTATE

# Configure firewall if ufw is available
if command -v ufw &> /dev/null; then
    # Allow SSH
    ufw allow ssh
    # Allow HTTP and HTTPS for Caddy
    ufw allow 80/tcp
    ufw allow 443/tcp
    # Enable firewall
    ufw enable
fi

# Create service status check
if ! systemctl is-active --quiet caddy; then
    echo "ERROR: Caddy service failed to start"
    exit 1
fi

# Verify backend connection
curl -f http://127.0.0.1:8000/health > /dev/null || {
    echo "ERROR: Cannot connect to backend API"
    exit 1
}

echo "Caddy configuration completed successfully"
