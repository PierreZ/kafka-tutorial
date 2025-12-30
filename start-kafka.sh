#!/bin/bash
set -euo pipefail

ADVERTISED_HOST="${1:-localhost}"
KAFKA_PORT="${2:-9092}"
ADMIN_PASS="${3:-admin-secret}"
CONTAINER_NAME="kafka-tutorial"

# Build JAAS config with team passwords following pattern team-N-secret
JAAS_CONFIG="org.apache.kafka.common.security.plain.PlainLoginModule required username=\"admin\" password=\"$ADMIN_PASS\" user_admin=\"$ADMIN_PASS\""
for i in $(seq 1 15); do
    JAAS_CONFIG="$JAAS_CONFIG user_team-$i=\"team-$i-secret\""
done
JAAS_CONFIG="$JAAS_CONFIG;"

echo "=== Starting Kafka ==="
echo "Advertised host: $ADVERTISED_HOST:$KAFKA_PORT"

# Stop existing container if running
if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "Stopping existing container..."
    docker rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true
fi

docker run -d \
    --name "$CONTAINER_NAME" \
    -p "${KAFKA_PORT}:9092" \
    -e KAFKA_CFG_PROCESS_ROLES=broker,controller \
    -e KAFKA_CFG_NODE_ID=1 \
    -e KAFKA_KRAFT_CLUSTER_ID=MkU3OEVBNTcwNTJENDM2Qg \
    -e KAFKA_CFG_CONTROLLER_QUORUM_VOTERS=1@localhost:9093 \
    -e KAFKA_CFG_CONTROLLER_LISTENER_NAMES=CONTROLLER \
    -e KAFKA_CFG_LISTENERS=SASL_PLAINTEXT://:9092,CONTROLLER://:9093 \
    -e KAFKA_CFG_ADVERTISED_LISTENERS=SASL_PLAINTEXT://${ADVERTISED_HOST}:${KAFKA_PORT} \
    -e KAFKA_CFG_LISTENER_SECURITY_PROTOCOL_MAP=SASL_PLAINTEXT:SASL_PLAINTEXT,CONTROLLER:PLAINTEXT \
    -e KAFKA_CFG_INTER_BROKER_LISTENER_NAME=SASL_PLAINTEXT \
    -e KAFKA_CFG_SASL_ENABLED_MECHANISMS=PLAIN \
    -e KAFKA_CFG_SASL_MECHANISM_INTER_BROKER_PROTOCOL=PLAIN \
    -e "KAFKA_CFG_LISTENER_NAME_SASL__PLAINTEXT_PLAIN_SASL_JAAS_CONFIG=$JAAS_CONFIG" \
    -e KAFKA_CFG_AUTHORIZER_CLASS_NAME=org.apache.kafka.metadata.authorizer.StandardAuthorizer \
    -e KAFKA_CFG_SUPER_USERS=User:admin \
    -e KAFKA_CFG_ALLOW_EVERYONE_IF_NO_ACL_FOUND=false \
    -e KAFKA_CFG_NUM_PARTITIONS=2 \
    -e KAFKA_CFG_DEFAULT_REPLICATION_FACTOR=1 \
    -e KAFKA_CFG_OFFSETS_TOPIC_REPLICATION_FACTOR=1 \
    -e KAFKA_CFG_TRANSACTION_STATE_LOG_REPLICATION_FACTOR=1 \
    -e KAFKA_CFG_TRANSACTION_STATE_LOG_MIN_ISR=1 \
    -e KAFKA_CFG_AUTO_CREATE_TOPICS_ENABLE=false \
    -e KAFKA_CFG_LOG_RETENTION_HOURS=168 \
    bitnami/kafka:3.9

echo "Container started. Run ./setup-kafka.sh to create topics and ACLs."
