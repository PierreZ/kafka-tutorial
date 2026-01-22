#!/usr/bin/env bash
set -euo pipefail

ADVERTISED_HOST="${1:-localhost}"
KAFKA_PORT="${2:-9092}"
ADMIN_PASS="${3:-admin-secret}"
CONTAINER_NAME="redpanda-tutorial"

echo "=== Starting Redpanda ==="
echo "Advertised host: $ADVERTISED_HOST:$KAFKA_PORT"

# Stop existing container if running
if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "Stopping existing container..."
    docker rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true
fi

docker run -d \
    --name "$CONTAINER_NAME" \
    -p "${KAFKA_PORT}:9092" \
    -p "9644:9644" \
    -e RP_BOOTSTRAP_USER="admin:${ADMIN_PASS}" \
    docker.redpanda.com/redpandadata/redpanda:latest \
    redpanda start \
    --mode dev-container \
    --smp 1 \
    --memory 1G \
    --kafka-addr "PLAINTEXT://0.0.0.0:29092,SASL://0.0.0.0:9092" \
    --advertise-kafka-addr "PLAINTEXT://localhost:29092,SASL://${ADVERTISED_HOST}:${KAFKA_PORT}" \
    --set redpanda.enable_sasl=true \
    --set redpanda.superusers=['admin'] \
    --set redpanda.default_num_partitions=2 \
    --set redpanda.auto_create_topics_enabled=false

echo "Container started. Run ./setup-redpanda.sh to create users, topics and ACLs."
