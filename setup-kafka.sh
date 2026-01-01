#!/bin/bash
set -euo pipefail

ADMIN_USER="${1:-admin}"
ADMIN_PASS="${2:-admin-secret}"
CONTAINER_NAME="kafka-tutorial"

# Check container is running
if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "ERROR: Container '$CONTAINER_NAME' is not running"
    echo "Start it with: ./start-kafka.sh <advertised-host>"
    exit 1
fi

echo "=== Kafka Tutorial Setup ==="

# Create admin properties inside container
docker exec "$CONTAINER_NAME" bash -c "cat > /tmp/admin.properties << EOF
security.protocol=SASL_PLAINTEXT
sasl.mechanism=PLAIN
sasl.jaas.config=org.apache.kafka.common.security.plain.PlainLoginModule required username=\"$ADMIN_USER\" password=\"$ADMIN_PASS\";
EOF"

# Wait for Kafka to be ready
echo "Waiting for Kafka to be ready..."
for i in $(seq 1 30); do
    if docker exec "$CONTAINER_NAME" kafka-broker-api-versions.sh \
        --bootstrap-server localhost:9092 \
        --command-config /tmp/admin.properties >/dev/null 2>&1; then
        echo "Kafka is ready!"
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo "ERROR: Kafka did not become ready in time"
        echo "Check logs with: docker logs $CONTAINER_NAME"
        exit 1
    fi
    echo "Waiting... ($i/30)"
    sleep 2
done

# Create topics
echo ""
echo "=== Creating Topics ==="
docker exec "$CONTAINER_NAME" kafka-topics.sh --create \
    --bootstrap-server localhost:9092 \
    --command-config /tmp/admin.properties \
    --topic new_users --partitions 2 --replication-factor 1 --if-not-exists

docker exec "$CONTAINER_NAME" kafka-topics.sh --create \
    --bootstrap-server localhost:9092 \
    --command-config /tmp/admin.properties \
    --topic actions --partitions 2 --replication-factor 1 --if-not-exists

# Create log-compacted topics for step 5 (watchlist) and leaderboard state
docker exec "$CONTAINER_NAME" kafka-topics.sh --create \
    --bootstrap-server localhost:9092 \
    --command-config /tmp/admin.properties \
    --topic watchlist --partitions 2 --replication-factor 1 \
    --config cleanup.policy=compact --if-not-exists

docker exec "$CONTAINER_NAME" kafka-topics.sh --create \
    --bootstrap-server localhost:9092 \
    --command-config /tmp/admin.properties \
    --topic scorer_state --partitions 1 --replication-factor 1 \
    --config cleanup.policy=compact --if-not-exists

# Create ACLs
echo ""
echo "=== Creating ACLs ==="
for team_num in $(seq 1 15); do
    team="team-$team_num"
    echo "Setting up $team..."

    docker exec "$CONTAINER_NAME" kafka-acls.sh \
        --bootstrap-server localhost:9092 \
        --command-config /tmp/admin.properties \
        --add --allow-principal "User:$team" \
        --operation Read --operation Describe \
        --topic new_users --force >/dev/null

    docker exec "$CONTAINER_NAME" kafka-acls.sh \
        --bootstrap-server localhost:9092 \
        --command-config /tmp/admin.properties \
        --add --allow-principal "User:$team" \
        --operation Write --operation Describe \
        --topic actions --force >/dev/null

    # Allow teams to write to watchlist (step 5)
    docker exec "$CONTAINER_NAME" kafka-acls.sh \
        --bootstrap-server localhost:9092 \
        --command-config /tmp/admin.properties \
        --add --allow-principal "User:$team" \
        --operation Write --operation Describe \
        --topic watchlist --force >/dev/null

    docker exec "$CONTAINER_NAME" kafka-acls.sh \
        --bootstrap-server localhost:9092 \
        --command-config /tmp/admin.properties \
        --add --allow-principal "User:$team" \
        --operation Read --operation Describe \
        --group "$team" --force >/dev/null
done

# Create leaderboard user ACLs for scorer_state topic
echo "Setting up leaderboard user..."
docker exec "$CONTAINER_NAME" kafka-acls.sh \
    --bootstrap-server localhost:9092 \
    --command-config /tmp/admin.properties \
    --add --allow-principal "User:leaderboard" \
    --operation Read --operation Write --operation Describe \
    --topic scorer_state --force >/dev/null

docker exec "$CONTAINER_NAME" kafka-acls.sh \
    --bootstrap-server localhost:9092 \
    --command-config /tmp/admin.properties \
    --add --allow-principal "User:leaderboard" \
    --operation Read --operation Describe \
    --topic actions --force >/dev/null

docker exec "$CONTAINER_NAME" kafka-acls.sh \
    --bootstrap-server localhost:9092 \
    --command-config /tmp/admin.properties \
    --add --allow-principal "User:leaderboard" \
    --operation Read --operation Describe \
    --topic watchlist --force >/dev/null

docker exec "$CONTAINER_NAME" kafka-acls.sh \
    --bootstrap-server localhost:9092 \
    --command-config /tmp/admin.properties \
    --add --allow-principal "User:leaderboard" \
    --operation Read --operation Describe \
    --group "leaderboard" --resource-pattern-type prefixed --force >/dev/null

echo ""
echo "=== Setup Complete ==="
echo "Topics: new_users, actions, watchlist (compacted), scorer_state (compacted)"
echo "Teams: team-1 through team-15 (password: team-N-secret)"
echo "Leaderboard user: leaderboard (password: leaderboard-secret)"
