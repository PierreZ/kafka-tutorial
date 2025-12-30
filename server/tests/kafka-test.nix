{ pkgs, lib, ... }:

# NixOS test for the Kafka Tutorial server
# This test verifies:
# 1. Kafka starts successfully in KRaft mode
# 2. Topics (new_users, actions) are created
# 3. Producing and consuming messages works
# 4. SASL authentication works correctly
# 5. The full ETL pipeline can be simulated

let
  # Test user credentials (from kafka.nix)
  testUsername = "team-1";
  testPassword = "team-1-secret";
  adminUsername = "admin";
  adminPassword = "admin-secret";

  # Sample user data matching the tutorial format
  sampleUser = builtins.toJSON {
    email = "test_user@hotmail.com";
    credit_card_number = "4111111111111111";
    company_name = "Test Company";
    company_slogan = "Testing the Future";
    industry = "Technology";
    user_name = "test_user";
    avatar = "https://example.com/avatar.png";
    name = "Test User";
    profession = "engineer";
    field = "IT";
    premium = true;
    credit = 15;
    time_zone = "Europe/Paris";
    user_agent = "Mozilla/5.0 (X11; Linux x86_64)";
    pack = "premium";
  };

  # Sample action message
  sampleAction = builtins.toJSON {
    customer = "test_user@hotmail.com";
    type = "CONTACT_CUSTOMER";
    reason = "LEGACY_EMAIL_PROVIDER";
    team = "team-1";
  };

  # Client properties for authenticated access
  teamClientProperties = pkgs.writeText "team-client.properties" ''
    security.protocol=SASL_PLAINTEXT
    sasl.mechanism=PLAIN
    sasl.jaas.config=org.apache.kafka.common.security.plain.PlainLoginModule required username="${testUsername}" password="${testPassword}";
  '';

  adminClientProperties = pkgs.writeText "admin-client.properties" ''
    security.protocol=SASL_PLAINTEXT
    sasl.mechanism=PLAIN
    sasl.jaas.config=org.apache.kafka.common.security.plain.PlainLoginModule required username="${adminUsername}" password="${adminPassword}";
  '';

in
{
  name = "kafka-tutorial";

  meta = with lib.maintainers; {
    maintainers = [ ];
  };

  nodes = {
    server = { config, pkgs, ... }: {
      imports = [
        ../modules/kafka.nix
        ../modules/kafka-bootstrap.nix
      ];

      # Test-specific overrides
      virtualisation = {
        memorySize = 2048;
        cores = 2;
        diskSize = 4096;
      };

      # Override listeners for testing (use non-privileged port)
      services.apache-kafka.settings = {
        "listeners" = lib.mkForce "SASL_PLAINTEXT://0.0.0.0:9092,CONTROLLER://localhost:9093";
        "advertised.listeners" = lib.mkForce "SASL_PLAINTEXT://localhost:9092";
      };

      # Remove capability requirement for test (using port 9092)
      systemd.services.apache-kafka.serviceConfig.AmbientCapabilities = lib.mkForce [ ];

      # Update bootstrap service to use test port
      systemd.services.kafka-create-topics.script = lib.mkForce ''
        set -euo pipefail

        BOOTSTRAP_SERVER="localhost:9092"
        COMMAND_CONFIG="/etc/kafka/admin.properties"
        MAX_RETRIES=30
        RETRY_INTERVAL=2

        echo "Waiting for Kafka to be ready..."

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

        echo "Creating topic 'new_users'..."
        kafka-topics.sh --create \
          --bootstrap-server "$BOOTSTRAP_SERVER" \
          --command-config "$COMMAND_CONFIG" \
          --topic new_users \
          --partitions 3 \
          --replication-factor 1 \
          --if-not-exists

        echo "Creating topic 'actions'..."
        kafka-topics.sh --create \
          --bootstrap-server "$BOOTSTRAP_SERVER" \
          --command-config "$COMMAND_CONFIG" \
          --topic actions \
          --partitions 3 \
          --replication-factor 1 \
          --if-not-exists

        echo "Verifying topics..."
        kafka-topics.sh --list \
          --bootstrap-server "$BOOTSTRAP_SERVER" \
          --command-config "$COMMAND_CONFIG"

        echo "Topic creation complete!"
      '';

      environment.systemPackages = with pkgs; [
        apacheKafka
        jq
      ];

      # Make client properties available
      environment.etc."kafka/team-client.properties".source = teamClientProperties;
    };
  };

  testScript = ''
    import json

    start_all()

    with subtest("Wait for Kafka to start"):
        server.wait_for_unit("apache-kafka.service")
        server.wait_for_open_port(9092)

    with subtest("Wait for topic creation service"):
        server.wait_for_unit("kafka-create-topics.service")
        # Give it a moment to complete
        server.sleep(5)

    with subtest("Verify topics exist"):
        result = server.succeed(
            "kafka-topics.sh --list "
            "--bootstrap-server localhost:9092 "
            "--command-config /etc/kafka/admin.properties"
        )
        assert "new_users" in result, f"Topic 'new_users' not found in: {result}"
        assert "actions" in result, f"Topic 'actions' not found in: {result}"

    with subtest("Verify topic configuration - new_users has 3 partitions"):
        result = server.succeed(
            "kafka-topics.sh --describe --topic new_users "
            "--bootstrap-server localhost:9092 "
            "--command-config /etc/kafka/admin.properties"
        )
        assert "PartitionCount: 3" in result or "Partition: 2" in result, \
            f"Expected 3 partitions for new_users, got: {result}"

    with subtest("Test SASL authentication - team credentials work"):
        server.succeed(
            "kafka-topics.sh --list "
            "--bootstrap-server localhost:9092 "
            "--command-config /etc/kafka/team-client.properties"
        )

    with subtest("Produce message to new_users topic"):
        # Write the sample user JSON to a temp file and produce it
        server.succeed(
            f"echo '{sampleUser.replace("'", "\\'")}' | "
            "kafka-console-producer.sh "
            "--bootstrap-server localhost:9092 "
            "--topic new_users "
            "--producer.config /etc/kafka/team-client.properties"
        )

    with subtest("Consume message from new_users topic"):
        result = server.succeed(
            "timeout 10 kafka-console-consumer.sh "
            "--bootstrap-server localhost:9092 "
            "--topic new_users "
            "--from-beginning "
            "--max-messages 1 "
            "--consumer.config /etc/kafka/team-client.properties"
        )
        # Parse and verify the message content
        msg = json.loads(result.strip())
        assert msg["email"] == "test_user@hotmail.com", f"Email mismatch: {msg}"
        assert msg["premium"] == True, f"Premium flag mismatch: {msg}"

    with subtest("Produce action message to actions topic"):
        server.succeed(
            f"echo '{sampleAction.replace("'", "\\'")}' | "
            "kafka-console-producer.sh "
            "--bootstrap-server localhost:9092 "
            "--topic actions "
            "--producer.config /etc/kafka/team-client.properties"
        )

    with subtest("Consume action message from actions topic"):
        result = server.succeed(
            "timeout 10 kafka-console-consumer.sh "
            "--bootstrap-server localhost:9092 "
            "--topic actions "
            "--from-beginning "
            "--max-messages 1 "
            "--consumer.config /etc/kafka/team-client.properties"
        )
        msg = json.loads(result.strip())
        assert msg["type"] == "CONTACT_CUSTOMER", f"Type mismatch: {msg}"
        assert msg["reason"] == "LEGACY_EMAIL_PROVIDER", f"Reason mismatch: {msg}"
        assert msg["team"] == "team-1", f"Team mismatch: {msg}"

    with subtest("Test consumer group functionality"):
        # Create a consumer group and verify it exists
        server.succeed(
            "timeout 5 kafka-console-consumer.sh "
            "--bootstrap-server localhost:9092 "
            "--topic new_users "
            "--group test-consumer-group "
            "--from-beginning "
            "--max-messages 1 "
            "--consumer.config /etc/kafka/team-client.properties || true"
        )

        result = server.succeed(
            "kafka-consumer-groups.sh --list "
            "--bootstrap-server localhost:9092 "
            "--command-config /etc/kafka/admin.properties"
        )
        assert "test-consumer-group" in result, f"Consumer group not found: {result}"

    with subtest("Verify all team credentials work"):
        for team_num in [1, 5, 10, 15]:
            team_props = f'''
                security.protocol=SASL_PLAINTEXT
                sasl.mechanism=PLAIN
                sasl.jaas.config=org.apache.kafka.common.security.plain.PlainLoginModule required username="team-{team_num}" password="team-{team_num}-secret";
            '''
            server.succeed(
                f"echo '{team_props}' > /tmp/team-{team_num}.properties && "
                f"kafka-topics.sh --list "
                f"--bootstrap-server localhost:9092 "
                f"--command-config /tmp/team-{team_num}.properties"
            )

    with subtest("Test message with ETL pipeline simulation"):
        # Simulate the full ETL pipeline:
        # 1. Produce a user with hotmail email (triggers Team-1 filter)
        # 2. Consume and verify
        # 3. Produce corresponding action

        hotmail_user = json.dumps({
            "email": "etl_test@hotmail.com",
            "credit_card_number": "1234567890123456",
            "company_name": "ETL Test Corp",
            "company_slogan": "Testing ETL",
            "industry": "Testing",
            "user_name": "etl_tester",
            "avatar": "https://example.com/etl.png",
            "name": "ETL Tester",
            "profession": "tester",
            "field": "QA",
            "premium": False,
            "credit": 0,
            "time_zone": "UTC",
            "user_agent": "TestAgent/1.0",
            "pack": "free"
        })

        # Produce the user
        server.succeed(
            f"echo '{hotmail_user}' | "
            "kafka-console-producer.sh "
            "--bootstrap-server localhost:9092 "
            "--topic new_users "
            "--producer.config /etc/kafka/team-client.properties"
        )

        # Consume and verify it's a hotmail user
        result = server.succeed(
            "timeout 10 kafka-console-consumer.sh "
            "--bootstrap-server localhost:9092 "
            "--topic new_users "
            "--group etl-test-group "
            "--from-beginning "
            "--max-messages 2 "
            "--consumer.config /etc/kafka/team-client.properties | tail -1"
        )
        msg = json.loads(result.strip())
        assert "hotmail" in msg["email"], f"Expected hotmail email: {msg['email']}"

        # Produce the corresponding action
        action_msg = json.dumps({
            "customer": msg["email"],
            "type": "CONTACT_CUSTOMER",
            "reason": "LEGACY_EMAIL_PROVIDER",
            "team": "team-1"
        })
        server.succeed(
            f"echo '{action_msg}' | "
            "kafka-console-producer.sh "
            "--bootstrap-server localhost:9092 "
            "--topic actions "
            "--producer.config /etc/kafka/team-client.properties"
        )

    print("All Kafka tutorial tests passed!")
  '';
}
