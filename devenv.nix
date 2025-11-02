{pkgs, ...}: {
  packages = with pkgs; [
    # Toolchain
    rustup
    espup

    # Upload/debug tool
    probe-rs

    # Code formatting tools
    treefmt
    alejandra
    mdl
    rustfmt

    # Release tools
    release-plz
  ];

  env = {
    DBX_CONTAINER_IMAGE = "ubuntu:24.04";
    DBX_CONTAINER_NAME = "crabby-tildagon-build-env";
  };
}
