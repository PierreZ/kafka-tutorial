#!/usr/bin/env bash
set -euo pipefail

ADMIN_USER="${1:-admin}"
ADMIN_PASS="${2:-admin-secret}"
CONTAINER_NAME="redpanda-tutorial"

# Auth flags for rpk commands
AUTH_FLAGS="-X user=${ADMIN_USER} -X pass=${ADMIN_PASS} -X sasl.mechanism=SCRAM-SHA-256"

# Check container is running
if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "ERROR: Container '$CONTAINER_NAME' is not running"
    echo "Start it with: ./start-redpanda.sh <advertised-host>"
    exit 1
fi

echo "=== Redpanda Tutorial Setup ==="

# Wait for Redpanda to be ready
echo "Waiting for Redpanda to be ready..."
for i in $(seq 1 30); do
    if docker exec "$CONTAINER_NAME" rpk cluster info $AUTH_FLAGS >/dev/null 2>&1; then
        echo "Redpanda is ready!"
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo "ERROR: Redpanda did not become ready in time"
        echo "Check logs with: docker logs $CONTAINER_NAME"
        exit 1
    fi
    echo "Waiting... ($i/30)"
    sleep 2
done

# Create users
echo ""
echo "=== Creating Users ==="
for team_num in $(seq 1 15); do
    team="team-$team_num"
    password="team-$team_num-secret"
    echo "Creating user: $team"
    docker exec "$CONTAINER_NAME" rpk security user create "$team" \
        -p "$password" \
        --mechanism scram-sha-256 \
        $AUTH_FLAGS >/dev/null 2>&1 || echo "  (user may already exist)"
done

# Create leaderboard user
echo "Creating user: leaderboard"
docker exec "$CONTAINER_NAME" rpk security user create leaderboard \
    -p "leaderboard-secret" \
    --mechanism scram-sha-256 \
    $AUTH_FLAGS >/dev/null 2>&1 || echo "  (user may already exist)"

# Create team-admin user
echo "Creating user: team-admin"
docker exec "$CONTAINER_NAME" rpk security user create team-admin \
    -p "team-admin-secret" \
    --mechanism scram-sha-256 \
    $AUTH_FLAGS >/dev/null 2>&1 || echo "  (user may already exist)"

# Create topics
echo ""
echo "=== Creating Topics ==="
docker exec "$CONTAINER_NAME" rpk topic create new_users \
    --partitions 5 \
    --replicas 1 \
    --topic-config retention.ms=3600000 \
    $AUTH_FLAGS 2>/dev/null || echo "Topic new_users may already exist"

docker exec "$CONTAINER_NAME" rpk topic create actions \
    --partitions 5 \
    --replicas 1 \
    --topic-config retention.ms=3600000 \
    $AUTH_FLAGS 2>/dev/null || echo "Topic actions may already exist"

# Create log-compacted topics
docker exec "$CONTAINER_NAME" rpk topic create team_stats \
    --partitions 5 \
    --replicas 1 \
    --topic-config cleanup.policy=compact \
    --topic-config segment.ms=60000 \
    $AUTH_FLAGS 2>/dev/null || echo "Topic team_stats may already exist"

docker exec "$CONTAINER_NAME" rpk topic create scorer_state \
    --partitions 1 \
    --replicas 1 \
    --topic-config cleanup.policy=compact \
    --topic-config segment.ms=60000 \
    $AUTH_FLAGS 2>/dev/null || echo "Topic scorer_state may already exist"

# Create ACLs
echo ""
echo "=== Creating ACLs ==="
for team_num in $(seq 1 15); do
    team="team-$team_num"
    echo "Setting up ACLs for $team..."

    # Read new_users topic
    docker exec "$CONTAINER_NAME" rpk acl create \
        --allow-principal "User:$team" \
        --operation read --operation describe \
        --topic new_users \
        $AUTH_FLAGS >/dev/null

    # Write to actions topic
    docker exec "$CONTAINER_NAME" rpk acl create \
        --allow-principal "User:$team" \
        --operation write --operation describe \
        --topic actions \
        $AUTH_FLAGS >/dev/null

    # Write to team_stats topic (step 5)
    docker exec "$CONTAINER_NAME" rpk acl create \
        --allow-principal "User:$team" \
        --operation write --operation describe \
        --topic team_stats \
        $AUTH_FLAGS >/dev/null

    # Consumer group access
    docker exec "$CONTAINER_NAME" rpk acl create \
        --allow-principal "User:$team" \
        --operation read --operation describe \
        --group "$team" \
        $AUTH_FLAGS >/dev/null
done

# Create leaderboard user ACLs
echo "Setting up ACLs for leaderboard user..."
docker exec "$CONTAINER_NAME" rpk acl create \
    --allow-principal "User:leaderboard" \
    --operation read --operation write --operation describe \
    --topic scorer_state \
    $AUTH_FLAGS >/dev/null

docker exec "$CONTAINER_NAME" rpk acl create \
    --allow-principal "User:leaderboard" \
    --operation read --operation describe \
    --topic new_users \
    $AUTH_FLAGS >/dev/null

docker exec "$CONTAINER_NAME" rpk acl create \
    --allow-principal "User:leaderboard" \
    --operation read --operation describe \
    --topic actions \
    $AUTH_FLAGS >/dev/null

docker exec "$CONTAINER_NAME" rpk acl create \
    --allow-principal "User:leaderboard" \
    --operation read --operation describe \
    --topic team_stats \
    $AUTH_FLAGS >/dev/null

docker exec "$CONTAINER_NAME" rpk acl create \
    --allow-principal "User:leaderboard" \
    --operation read --operation describe \
    --group "leaderboard" \
    --resource-pattern-type prefixed \
    $AUTH_FLAGS >/dev/null

# Leaderboard needs to read team consumer groups for lag tracking
docker exec "$CONTAINER_NAME" rpk acl create \
    --allow-principal "User:leaderboard" \
    --operation read --operation describe \
    --group "team-" \
    --resource-pattern-type prefixed \
    $AUTH_FLAGS >/dev/null

# Create team-admin ACLs (same as teams)
echo "Setting up ACLs for team-admin..."

# Read new_users topic
docker exec "$CONTAINER_NAME" rpk acl create \
    --allow-principal "User:team-admin" \
    --operation read --operation describe \
    --topic new_users \
    $AUTH_FLAGS >/dev/null

# Write to actions topic
docker exec "$CONTAINER_NAME" rpk acl create \
    --allow-principal "User:team-admin" \
    --operation write --operation describe \
    --topic actions \
    $AUTH_FLAGS >/dev/null

# Write to team_stats topic
docker exec "$CONTAINER_NAME" rpk acl create \
    --allow-principal "User:team-admin" \
    --operation write --operation describe \
    --topic team_stats \
    $AUTH_FLAGS >/dev/null

# Consumer group access
docker exec "$CONTAINER_NAME" rpk acl create \
    --allow-principal "User:team-admin" \
    --operation read --operation describe \
    --group "team-admin" \
    $AUTH_FLAGS >/dev/null

echo ""
echo "=== Setup Complete ==="
echo "Topics: new_users, actions, team_stats (compacted), scorer_state (compacted)"
echo "Teams: team-1 through team-15 (password: team-N-secret)"
echo "Team admin: team-admin (password: team-admin-secret)"
echo "Leaderboard user: leaderboard (password: leaderboard-secret)"
