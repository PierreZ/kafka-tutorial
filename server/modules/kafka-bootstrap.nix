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

      BOOTSTRAP_SERVER="localhost:443"
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
    '';
  };
}
