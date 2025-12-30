{ pkgs, ... }:

pkgs.testers.runNixOSTest {
  name = "kafka-tutorial-server";

  nodes.server = { config, lib, pkgs, ... }: {
    imports = [
      ../modules/kafka.nix
      ../modules/kafka-bootstrap.nix
    ];

    # Override for VM testing (use non-privileged port 9092)
    services.apache-kafka.settings = {
      "listeners" = lib.mkForce "SASL_PLAINTEXT://0.0.0.0:9092,CONTROLLER://localhost:9093";
      "advertised.listeners" = lib.mkForce "SASL_PLAINTEXT://0.0.0.0:9092";
    };

    # Update bootstrap script to use port 9092
    systemd.services.kafka-create-topics.environment.BOOTSTRAP_PORT = "9092";

    # Increase VM resources for Kafka
    virtualisation = {
      memorySize = 2048;
      cores = 2;
    };

    # Ensure Kafka tools are available
    environment.systemPackages = [ pkgs.apacheKafka ];
  };

  testScript = ''
    # Helper to create client property files
    def create_client_props(machine, name, username, password):
        machine.succeed(f"""
          cat > /tmp/{name}.properties << 'EOF'
security.protocol=SASL_PLAINTEXT
sasl.mechanism=PLAIN
sasl.jaas.config=org.apache.kafka.common.security.plain.PlainLoginModule required username="{username}" password="{password}";
EOF
        """)

    # Start server and wait for services
    server.start()
    server.wait_for_unit("apache-kafka.service")
    server.wait_for_unit("kafka-create-topics.service")

    # Create client property files for testing
    create_client_props(server, "team1", "team-1", "team-1-secret")
    create_client_props(server, "team8", "team-8", "team-8-secret")
    create_client_props(server, "team15", "team-15", "team-15-secret")
    create_client_props(server, "invalid", "team-1", "wrong-password")

    # === TOPIC TESTS ===
    with subtest("Topics are created correctly"):
        server.succeed("kafka-topics.sh --list --bootstrap-server localhost:9092 --command-config /etc/kafka/admin.properties | grep -E '^new_users$'")
        server.succeed("kafka-topics.sh --list --bootstrap-server localhost:9092 --command-config /etc/kafka/admin.properties | grep -E '^actions$'")

    # === AUTHENTICATION TESTS ===
    with subtest("Valid credentials authenticate successfully"):
        server.succeed("echo 'auth-test' | kafka-console-producer.sh --bootstrap-server localhost:9092 --topic actions --producer.config /tmp/team1.properties")

    with subtest("Invalid credentials fail authentication"):
        server.fail("echo 'test' | timeout 10 kafka-console-producer.sh --bootstrap-server localhost:9092 --topic actions --producer.config /tmp/invalid.properties 2>&1")

    # === ACL TESTS ===
    # Seed data for consumer tests
    server.succeed("echo 'seed-message' | kafka-console-producer.sh --bootstrap-server localhost:9092 --topic new_users --producer.config /etc/kafka/admin.properties")

    for team_num in [1, 8, 15]:
        team = f"team{team_num}"
        group = f"team-{team_num}"

        with subtest(f"team-{team_num} can produce to actions"):
            server.succeed(f"echo 'test-{team_num}' | kafka-console-producer.sh --bootstrap-server localhost:9092 --topic actions --producer.config /tmp/{team}.properties")

        with subtest(f"team-{team_num} CANNOT produce to new_users"):
            server.fail(f"echo 'test' | timeout 10 kafka-console-producer.sh --bootstrap-server localhost:9092 --topic new_users --producer.config /tmp/{team}.properties 2>&1")

        with subtest(f"team-{team_num} can consume from new_users with own group"):
            server.succeed(f"timeout 15 kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic new_users --group {group} --from-beginning --max-messages 1 --consumer.config /tmp/{team}.properties")

        with subtest(f"team-{team_num} CANNOT consume from actions"):
            server.fail(f"timeout 10 kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic actions --group {group} --from-beginning --max-messages 1 --consumer.config /tmp/{team}.properties 2>&1")

    with subtest("team-1 CANNOT use team-8 consumer group"):
        server.fail("timeout 10 kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic new_users --group team-8 --from-beginning --max-messages 1 --consumer.config /tmp/team1.properties 2>&1")

    # === ADMIN TESTS ===
    with subtest("Admin can produce to new_users"):
        server.succeed("echo 'admin-test' | kafka-console-producer.sh --bootstrap-server localhost:9092 --topic new_users --producer.config /etc/kafka/admin.properties")

    with subtest("Admin can consume from actions"):
        server.succeed("timeout 15 kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic actions --group admin-test --from-beginning --max-messages 1 --consumer.config /etc/kafka/admin.properties")
  '';
}
