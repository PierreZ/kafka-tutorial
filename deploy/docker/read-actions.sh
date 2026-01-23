#!/bin/bash
set -euo pipefail

ADMIN_USER="${1:-admin}"
ADMIN_PASS="${2:-admin-secret}"
CONTAINER_NAME="${3:-redpanda-tutorial}"

docker exec "$CONTAINER_NAME" rpk topic consume actions \
  --user "$ADMIN_USER" --password "$ADMIN_PASS" \
  --sasl-mechanism SCRAM-SHA-256 \
  "${@:4}"
