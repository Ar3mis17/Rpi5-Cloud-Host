#!/bin/bash
# Health Check Script for Ar3mis Cloud
# Returns 0 if healthy, 1 if unhealthy

set -euo pipefail

API_URL="http://127.0.0.1:8000"
CADDY_URL="http://localhost"
TIMEOUT=10

check_api() {
    local response
    response=$(curl -s -f --max-time ${TIMEOUT} "${API_URL}/health" 2>/dev/null || echo "FAILED")
    if echo "${response}" | grep -q '"status":"healthy"'; then
        echo "✓ API: Healthy"
        return 0
    else
        echo "✗ API: Unhealthy (${response})"
        return 1
    fi
}

check_database() {
    local response
    response=$(curl -s -f --max-time ${TIMEOUT} "${API_URL}/health" 2>/dev/null || echo "FAILED")
    if echo "${response}" | grep -q '"database":"connected"'; then
        echo "✓ Database: Connected"
        return 0
    else
        echo "✗ Database: Disconnected (${response})"
        return 1
    fi
}

check_caddy() {
    local response
    response=$(curl -s -f --max-time ${TIMEOUT} "${CADDY_URL}/health" 2>/dev/null || echo "FAILED")
    if echo "${response}" | grep -q '"status":"healthy"'; then
        echo "✓ Caddy: Healthy"
        return 0
    else
        echo "✗ Caddy: Unhealthy (${response})"
        return 1
    fi
}

check_disk_space() {
    local usage
    usage=$(df /mnt/cloud | awk 'NR==2 {print $5}' | sed 's/%//')
    if [ "${usage}" -lt 90 ]; then
        echo "✓ Disk Space: ${usage}% used"
        return 0
    else
        echo "✗ Disk Space: ${usage}% used (CRITICAL)"
        return 1
    fi
}

check_memory() {
    local usage
    usage=$(free | awk 'NR==2 {printf "%.0f", $3/$2 * 100}')
    if [ "${usage}" -lt 90 ]; then
        echo "✓ Memory: ${usage}% used"
        return 0
    else
        echo "✗ Memory: ${usage}% used (WARNING)"
        return 1
    fi
}

check_docker() {
    local containers
    containers=$(docker ps --filter "name=ar3mis-cloud" --format "{{.Names}}" | wc -l)
    if [ "${containers}" -eq 2 ]; then
        echo "✓ Docker: Both containers running"
        return 0
    else
        echo "✗ Docker: Only ${containers}/2 containers running"
        return 1
    fi
}

main() {
    local failed=0

    echo "=== Ar3mis Cloud Health Check ==="
    echo "Time: $(date)"
    echo ""

    check_api || failed=1
    check_database || failed=1
    check_caddy || failed=1
    check_disk_space || failed=1
    check_memory || failed=1
    check_docker || failed=1

    echo ""
    if [ ${failed} -eq 0 ]; then
        echo "=== ALL CHECKS PASSED ==="
        exit 0
    else
        echo "=== SOME CHECKS FAILED ==="
        exit 1
    fi
}

main "$@"