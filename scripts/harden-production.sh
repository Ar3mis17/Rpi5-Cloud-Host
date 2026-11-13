#!/bin/bash
# Production Hardening Script for Ar3mis Cloud
# Run after deployment to secure the system

set -euo pipefail

echo "========================================"
echo "Ar3mis Cloud - Production Hardening"
echo "========================================"
echo ""

# 1. Secure SSH
echo "[1/8] Securing SSH..."
sudo tee /etc/ssh/sshd_config.d/99-ar3mis-hardening.conf > /dev/null << 'EOF'
# Ar3mis Cloud SSH Hardening
PermitRootLogin no
PasswordAuthentication no
PubkeyAuthentication yes
AuthorizedKeysFile .ssh/authorized_keys
PermitEmptyPasswords no
ChallengeResponseAuthentication no
UsePAM yes
X11Forwarding no
PrintMotd no
ClientAliveInterval 300
ClientAliveCountMax 2
MaxAuthTries 3
LoginGraceTime 30
AllowUsers ar3mis
EOF

sudo systemctl reload sshd
echo "  ✓ SSH hardened"

# 2. Configure UFW Firewall
echo "[2/8] Configuring Firewall..."
sudo ufw --force enable
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
# Allow Docker internal network
sudo ufw allow from 172.16.0.0/12
sudo ufw allow from 10.0.0.0/8
echo "  ✓ Firewall configured"

# 3. Secure Shared Memory
echo "[3/8] Securing Shared Memory..."
if ! grep -q "/run/shm" /etc/fstab; then
    echo "tmpfs /run/shm tmpfs defaults,noexec,nosuid,nodev 0 0" | sudo tee -a /etc/fstab
fi
echo "  ✓ Shared memory secured"

# 4. Disable Unused Filesystems
echo "[4/8] Disabling Unused Filesystems..."
sudo tee /etc/modprobe.d/ar3mis-filesystems.conf > /dev/null << 'EOF'
# Disable unused filesystems
install cramfs /bin/true
install freevxfs /bin/true
install jffs2 /bin/true
install hfs /bin/true
install hfsplus /bin/true
install squashfs /bin/true
install udf /bin/true
EOF
echo "  ✓ Unused filesystems disabled"

# 5. Kernel Parameters Hardening
echo "[5/8] Hardening Kernel Parameters..."
sudo tee /etc/sysctl.d/99-ar3mis-hardening.conf > /dev/null << 'EOF'
# Ar3mis Cloud Kernel Hardening

# Network Security
net.ipv4.ip_forward = 0
net.ipv4.conf.all.send_redirects = 0
net.ipv4.conf.default.send_redirects = 0
net.ipv4.conf.all.accept_redirects = 0
net.ipv4.conf.default.accept_redirects = 0
net.ipv4.conf.all.secure_redirects = 0
net.ipv4.conf.default.secure_redirects = 0
net.ipv4.conf.all.log_martians = 1
net.ipv4.conf.default.log_martians = 1
net.ipv4.icmp_echo_ignore_broadcasts = 1
net.ipv4.icmp_ignore_bogus_error_responses = 1
net.ipv4.tcp_syncookies = 1
net.ipv4.tcp_max_syn_backlog = 2048
net.ipv4.tcp_synack_retries = 2
net.ipv4.tcp_syn_retries = 5

# IPv6
net.ipv6.conf.all.accept_redirects = 0
net.ipv6.conf.default.accept_redirects = 0
net.ipv6.conf.all.accept_ra = 0
net.ipv6.conf.default.accept_ra = 0

# Kernel Security
kernel.dmesg_restrict = 1
kernel.kptr_restrict = 2
kernel.yama.ptrace_scope = 1
kernel.unprivileged_bpf_disabled = 1
fs.protected_hardlinks = 1
fs.protected_symlinks = 1
EOF

sudo sysctl --system
echo "  ✓ Kernel parameters hardened"

# 6. Set up Fail2Ban
echo "[6/8] Configuring Fail2Ban..."
if ! command -v fail2ban-client &> /dev/null; then
    sudo apt-get update && sudo apt-get install -y fail2ban
fi

sudo tee /etc/fail2ban/jail.d/ar3mis.conf > /dev/null << 'EOF'
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 3
backend = systemd

[sshd]
enabled = true
port = ssh
filter = sshd
logpath = %(sshd_log)s
maxretry = 3

[ar3mis-api]
enabled = true
port = 80,443
filter = ar3mis-api
logpath = /var/log/caddy/access.log
maxretry = 10
findtime = 60
bantime = 1800
EOF

sudo tee /etc/fail2ban/filter.d/ar3mis-api.conf > /dev/null << 'EOF'
[Definition]
failregex = ^.*"remote_ip":"<HOST>".*"status":(401|403|429).*
ignoreregex =
EOF

sudo systemctl enable fail2ban
sudo systemctl restart fail2ban
echo "  ✓ Fail2Ban configured"

# 7. Configure Log Rotation
echo "[7/8] Setting up Log Rotation..."
sudo tee /etc/logrotate.d/ar3mis-cloud > /dev/null << 'EOF'
/var/log/caddy/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 640 root adm
    sharedscripts
    postrotate
        systemctl reload caddy-pihole > /dev/null 2>&1 || true
    endscript
}

/mnt/cloud/backups/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 640 root adm
}
EOF
echo "  ✓ Log rotation configured"

# 8. Docker Security
echo "[8/8] Hardening Docker..."
sudo tee /etc/docker/daemon.json > /dev/null << 'EOF'
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  },
  "live-restore": true,
  "userland-proxy": false,
  "no-new-privileges": true,
  "default-ulimits": {
    "nofile": {
      "Name": "nofile",
      "Hard": 64000,
      "Soft": 64000
    }
  }
}
EOF

sudo systemctl restart docker
echo "  ✓ Docker hardened"

# 9. Set proper permissions on backup directory
echo "[9/9] Setting backup permissions..."
sudo chown -R root:root /mnt/cloud/backups
sudo chmod 700 /mnt/cloud/backups
echo "  ✓ Backup permissions set"

echo ""
echo "========================================"
echo "Production Hardening Complete!"
echo "========================================"
echo ""
echo "Summary of changes:"
echo "  • SSH: Key-only auth, root login disabled"
echo "  • Firewall: UFW enabled, only SSH/HTTP/HTTPS open"
echo "  • Kernel: Network and memory protections enabled"
echo "  • Fail2Ban: SSH and API protection active"
echo "  • Logs: Rotation configured for Caddy and backups"
echo "  • Docker: Security options enabled"
echo "  • Backups: Directory permissions restricted"
echo ""
echo "IMPORTANT: Test SSH access before closing this session!"
echo "  ssh ar3mis@your-server-ip"