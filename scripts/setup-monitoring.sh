#!/bin/bash
# Setup Monitoring Stack for Ar3mis Cloud
# Installs Prometheus and configures node exporter

set -euo pipefail

echo "Setting up monitoring for Ar3mis Cloud..."

# Install Prometheus
if ! command -v prometheus &> /dev/null; then
    echo "Installing Prometheus..."
    PROM_VERSION="2.47.2"
    cd /tmp
    wget -q "https://github.com/prometheus/prometheus/releases/download/v${PROM_VERSION}/prometheus-${PROM_VERSION}.linux-arm64.tar.gz"
    tar -xzf "prometheus-${PROM_VERSION}.linux-arm64.tar.gz"
    sudo cp "prometheus-${PROM_VERSION}.linux-arm64/prometheus" /usr/local/bin/
    sudo cp "prometheus-${PROM_VERSION}.linux-arm64/promtool" /usr/local/bin/
    sudo mkdir -p /etc/prometheus /var/lib/prometheus
    sudo cp "prometheus-${PROM_VERSION}.linux-arm64/prometheus.yml" /etc/prometheus/
    sudo useradd --no-create-home --shell /bin/false prometheus || true
    sudo chown -R prometheus:prometheus /etc/prometheus /var/lib/prometheus
fi

# Create Prometheus config for Ar3mis Cloud
sudo tee /etc/prometheus/prometheus.yml > /dev/null << 'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'ar3mis-cloud'
    static_configs:
      - targets: ['localhost:9090']
        labels:
          service: 'ar3mis-cloud'

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['localhost:9100']
        labels:
          service: 'node-exporter'

  - job_name: 'cadvisor'
    static_configs:
      - targets: ['localhost:8080']
        labels:
          service: 'cadvisor'
EOF

# Create Prometheus systemd service
sudo tee /etc/systemd/system/prometheus.service > /dev/null << 'EOF'
[Unit]
Description=Prometheus
Wants=network-online.target
After=network-online.target

[Service]
User=prometheus
Group=prometheus
Type=simple
ExecStart=/usr/local/bin/prometheus \
    --config.file /etc/prometheus/prometheus.yml \
    --storage.tsdb.path /var/lib/prometheus/ \
    --web.console.libraries=/etc/prometheus/console_libraries \
    --web.console.templates=/etc/prometheus/consoles \
    --web.listen-address=0.0.0.0:9090
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

# Install Node Exporter
if ! command -v node_exporter &> /dev/null; then
    echo "Installing Node Exporter..."
    NODE_VERSION="1.7.0"
    cd /tmp
    wget -q "https://github.com/prometheus/node_exporter/releases/download/v${NODE_VERSION}/node_exporter-${NODE_VERSION}.linux-arm64.tar.gz"
    tar -xzf "node_exporter-${NODE_VERSION}.linux-arm64.tar.gz"
    sudo cp "node_exporter-${NODE_VERSION}.linux-arm64/node_exporter" /usr/local/bin/
    sudo useradd --no-create-home --shell /bin/false node_exporter || true
fi

sudo tee /etc/systemd/system/node_exporter.service > /dev/null << 'EOF'
[Unit]
Description=Node Exporter
Wants=network-online.target
After=network-online.target

[Service]
User=node_exporter
Group=node_exporter
Type=simple
ExecStart=/usr/local/bin/node_exporter --collector.systemd
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

# Install cAdvisor for Docker metrics
if ! docker ps | grep -q cadvisor; then
    echo "Starting cAdvisor container..."
    docker run -d \
        --name=cadvisor \
        --restart=unless-stopped \
        --volume=/:/rootfs:ro \
        --volume=/var/run:/var/run:ro \
        --volume=/sys:/sys:ro \
        --volume=/var/lib/docker/:/var/lib/docker:ro \
        --volume=/dev/disk/:/dev/disk:ro \
        --publish=8080:8080 \
        --detach=true \
        gcr.io/cadvisor/cadvisor:v0.49.1
fi

# Enable and start services
sudo systemctl daemon-reload
sudo systemctl enable prometheus node_exporter
sudo systemctl start prometheus node_exporter

echo ""
echo "Monitoring setup complete!"
echo "Prometheus: http://localhost:9090"
echo "Node Exporter: http://localhost:9100/metrics"
echo "cAdvisor: http://localhost:8080"
echo ""
echo "Ar3mis Cloud metrics available at: /var/lib/prometheus/ar3mis-cloud.prom"
echo "Run: sudo systemctl enable ar3mis-metrics.timer ar3mis-health.timer ar3mis-backup.timer"