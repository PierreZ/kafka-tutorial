{ config, lib, pkgs, ... }:

{
  # One-shot service to create Kafka topics on first boot
  systemd.services.kafka-create-topics = {
    description = "Create Kafka tutorial topics";
    after = [ "apache-kafka.service" ];
    requires = [ "apache-kafka.service" ];
    wantedBy = [ "multi-user.target" ];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      User = "root";
    };

    path = [ pkgs.apacheKafka ];

    script = ''
      set -euo pipefail

      # Use BOOTSTRAP_PORT env var if set (for testing), otherwise default to 443
      BOOTSTRAP_PORT="''${BOOTSTRAP_PORT:-443}"
      BOOTSTRAP_SERVER="localhost:$BOOTSTRAP_PORT"
      COMMAND_CONFIG="/etc/kafka/admin.properties"
      MAX_RETRIES=30
      RETRY_INTERVAL=5

      echo "Waiting for Kafka to be ready..."

      # Wait for Kafka to be ready
      for i in $(seq 1 $MAX_RETRIES); do
        if kafka-broker-api-versions.sh \
            --bootstrap-server "$BOOTSTRAP_SERVER" \
            --command-config "$COMMAND_CONFIG" 2>/dev/null; then
          echo "Kafka is ready!"
          break
        fi

        if [ "$i" -eq "$MAX_RETRIES" ]; then
          echo "Kafka did not become ready in time"
          exit 1
        fi

        echo "Waiting for Kafka... (attempt $i/$MAX_RETRIES)"
        sleep $RETRY_INTERVAL
      done

      # Create 'new_users' topic
      echo "Creating topic 'new_users'..."
      kafka-topics.sh --create \
        --bootstrap-server "$BOOTSTRAP_SERVER" \
        --command-config "$COMMAND_CONFIG" \
        --topic new_users \
        --partitions 3 \
        --replication-factor 1 \
        --if-not-exists

      # Create 'actions' topic
      echo "Creating topic 'actions'..."
      kafka-topics.sh --create \
        --bootstrap-server "$BOOTSTRAP_SERVER" \
        --command-config "$COMMAND_CONFIG" \
        --topic actions \
        --partitions 3 \
        --replication-factor 1 \
        --if-not-exists

      # List topics to verify
      echo "Verifying topics..."
      kafka-topics.sh --list \
        --bootstrap-server "$BOOTSTRAP_SERVER" \
        --command-config "$COMMAND_CONFIG"

      echo "Topic creation complete!"

      # Create ACLs for each team
      echo "Creating ACLs for teams..."

      for team_num in $(seq 1 15); do
        team="team-$team_num"
        echo "Setting up ACLs for $team..."

        # READ access to 'new_users' topic
        kafka-acls.sh --bootstrap-server "$BOOTSTRAP_SERVER" \
          --command-config "$COMMAND_CONFIG" \
          --add \
          --allow-principal "User:$team" \
          --operation Read \
          --operation Describe \
          --topic new_users

        # WRITE access to 'actions' topic
        kafka-acls.sh --bootstrap-server "$BOOTSTRAP_SERVER" \
          --command-config "$COMMAND_CONFIG" \
          --add \
          --allow-principal "User:$team" \
          --operation Write \
          --operation Describe \
          --topic actions

        # Consumer group access (READ on group matching team name)
        kafka-acls.sh --bootstrap-server "$BOOTSTRAP_SERVER" \
          --command-config "$COMMAND_CONFIG" \
          --add \
          --allow-principal "User:$team" \
          --operation Read \
          --operation Describe \
          --group "$team"
      done

      # List ACLs to verify
      echo "Verifying ACLs..."
      kafka-acls.sh --bootstrap-server "$BOOTSTRAP_SERVER" \
        --command-config "$COMMAND_CONFIG" \
        --list

      echo "ACL setup complete!"
    '';
  };
}
