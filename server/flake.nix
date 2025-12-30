{
  description = "NixOS Kafka Tutorial Server - OpenStack Image";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixos-generators, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      packages.${system} = {
        # OpenStack QCOW2 image
        openstack = nixos-generators.nixosGenerate {
          inherit system;
          format = "openstack";
          modules = [
            ./configuration.nix
          ];
          specialArgs = {
            inherit self;
          };
        };

        # For local testing with QEMU
        qcow = nixos-generators.nixosGenerate {
          inherit system;
          format = "qcow";
          modules = [
            ./configuration.nix
          ];
          specialArgs = {
            inherit self;
          };
        };

        # Default package is the OpenStack image
        default = nixos-generators.nixosGenerate {
          inherit system;
          format = "openstack";
          modules = [
            ./configuration.nix
          ];
          specialArgs = {
            inherit self;
          };
        };
      };

      # NixOS tests for verifying the Kafka tutorial server
      checks.${system} = {
        # Full integration test of the Kafka tutorial server
        kafka-tutorial = pkgs.nixosTest (import ./tests/kafka-test.nix);
      };
    };
}
