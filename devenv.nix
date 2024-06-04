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
    ESPUP_EXPORT_FILE = "./.export-esp.sh";

    DBX_CONTAINER_IMAGE = "ubuntu:24.04";
    DBX_CONTAINER_NAME = "crabby-tildagon-build-env";
  };
}
