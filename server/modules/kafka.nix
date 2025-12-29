{ config, lib, pkgs, ... }:

let
  # Generate JAAS config with all team credentials
  jaasConfig = ''
    org.apache.kafka.common.security.plain.PlainLoginModule required
      username="admin"
      password="admin-secret"
      user_admin="admin-secret"
      user_team-1="team-1-secret"
      user_team-2="team-2-secret"
      user_team-3="team-3-secret"
      user_team-4="team-4-secret"
      user_team-5="team-5-secret"
      user_team-6="team-6-secret"
      user_team-7="team-7-secret"
      user_team-8="team-8-secret"
      user_team-9="team-9-secret"
      user_team-10="team-10-secret"
      user_team-11="team-11-secret"
      user_team-12="team-12-secret"
      user_team-13="team-13-secret"
      user_team-14="team-14-secret"
      user_team-15="team-15-secret";
  '';

  # Admin client properties for topic creation
  adminProperties = pkgs.writeText "admin.properties" ''
    security.protocol=SASL_PLAINTEXT
    sasl.mechanism=PLAIN
    sasl.jaas.config=org.apache.kafka.common.security.plain.PlainLoginModule required username="admin" password="admin-secret";
  '';
in
{
  # Apache Kafka in KRaft mode
  services.apache-kafka = {
    enable = true;

    # KRaft cluster ID - fixed for reproducibility
    clusterId = "MkU3OEVBNTcwNTJENDM2Qg";

    # Auto-format log directories on first boot
    formatLogDirs = true;
    formatLogDirsIgnoreFormatted = true;

    settings = {
      # KRaft mode configuration
      "node.id" = 1;
      "process.roles" = "broker,controller";
      "controller.quorum.voters" = "1@localhost:9093";

      # Listeners configuration
      # Port 443 for external clients (SASL_PLAINTEXT)
      # Port 9093 for internal controller (PLAINTEXT)
      "listeners" = "SASL_PLAINTEXT://0.0.0.0:443,CONTROLLER://localhost:9093";
      "advertised.listeners" = "SASL_PLAINTEXT://0.0.0.0:443";
      "listener.security.protocol.map" = "SASL_PLAINTEXT:SASL_PLAINTEXT,CONTROLLER:PLAINTEXT";
      "inter.broker.listener.name" = "SASL_PLAINTEXT";
      "controller.listener.names" = "CONTROLLER";

      # SASL/PLAIN authentication
      "sasl.enabled.mechanisms" = "PLAIN";
      "sasl.mechanism.inter.broker.protocol" = "PLAIN";

      # JAAS configuration for SASL_PLAINTEXT listener
      "listener.name.sasl_plaintext.plain.sasl.jaas.config" = jaasConfig;

      # Super users (admin can do everything)
      "super.users" = "User:admin";

      # Topic defaults
      "num.partitions" = 3;
      "default.replication.factor" = 1;
      "offsets.topic.replication.factor" = 1;
      "transaction.state.log.replication.factor" = 1;
      "transaction.state.log.min.isr" = 1;

      # Disable auto topic creation - we create them explicitly
      "auto.create.topics.enable" = false;

      # Log retention
      "log.retention.hours" = 168; # 7 days

      # Log directories
      "log.dirs" = "/var/lib/kafka/logs";
    };
  };

  # Create admin properties file for CLI tools
  environment.etc."kafka/admin.properties".source = adminProperties;

  # Kafka needs to bind to port 443 (privileged port)
  # Allow the kafka service to bind to privileged ports
  systemd.services.apache-kafka.serviceConfig = {
    AmbientCapabilities = [ "CAP_NET_BIND_SERVICE" ];
  };
}
