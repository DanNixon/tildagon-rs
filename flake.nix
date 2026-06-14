{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = {nixpkgs, ...}: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs; [
        # Toolchain
        rustup
        espup

        # Upload/debug tool
        probe-rs-tools

        # Code formatting tools
        treefmt
        alejandra
        mdl
        rustfmt
        typos

        # Release tools
        release-plz
      ];

      DBX_CONTAINER_IMAGE = "ubuntu:26.04";
      DBX_CONTAINER_NAME = "tildagon-rs-build-environment";
    };
  };
}
