{ config, lib, pkgs, ... }:

{
  # Cloud-init for OpenStack metadata and SSH key injection
  services.cloud-init = {
    enable = true;
    network.enable = true;
  };

  # QEMU guest agent for better VM management
  services.qemuGuest.enable = true;

  # Networking - DHCP by default, cloud-init will configure
  networking = {
    useDHCP = true;
    firewall = {
      enable = true;
      allowedTCPPorts = [
        22    # SSH
        443   # Kafka (SASL_PLAINTEXT)
      ];
    };
  };

  # Growpart to expand root partition on first boot
  boot.growPartition = true;

  # Enable serial console for OpenStack console access
  boot.kernelParams = [ "console=ttyS0,115200" ];

  # Filesystem configuration for cloud images
  fileSystems."/" = lib.mkDefault {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
    autoResize = true;
  };

  # Time sync
  services.timesyncd.enable = true;

  # SSH configuration
  services.openssh = {
    enable = true;
    settings = {
      PermitRootLogin = "prohibit-password";
      PasswordAuthentication = false;
    };
  };

  # Default user (cloud-init will inject SSH keys)
  users.users.nixos = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    openssh.authorizedKeys.keys = []; # Populated by cloud-init
  };

  # Allow sudo without password for nixos user
  security.sudo.wheelNeedsPassword = false;
}
