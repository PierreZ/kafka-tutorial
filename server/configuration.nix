{ config, lib, pkgs, ... }:

{
  imports = [
    ./modules/openstack.nix
    ./modules/kafka.nix
    ./modules/kafka-bootstrap.nix
  ];

  # System configuration
  system.stateVersion = "24.11";

  # Hostname
  networking.hostName = "kafka-tutorial";

  # Timezone
  time.timeZone = "UTC";

  # System packages
  environment.systemPackages = with pkgs; [
    # Kafka CLI tools for debugging
    apacheKafka

    # Basic utilities
    vim
    htop
    jq
    curl
    wget
  ];

  # Enable nix flakes
  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  # Garbage collection
  nix.gc = {
    automatic = true;
    dates = "weekly";
    options = "--delete-older-than 7d";
  };

  # Documentation (use mkForce to override openstack-image.nix defaults)
  documentation = {
    enable = lib.mkForce false;
    man.enable = lib.mkForce false;
    nixos.enable = lib.mkForce false;
  };
}
