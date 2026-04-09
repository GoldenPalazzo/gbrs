{
  description = "Flake for rust development";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  

  outputs = { self, fenix, nixpkgs }: let
    toolchain = with fenix.packages.x86_64-linux; combine [
      stable.rustc
      stable.cargo
      stable.rustfmt
      stable.clippy
      stable.rust-src

      targets.thumbv6m-none-eabi.stable.rust-std
    ];
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
    runtimeLibs = with pkgs; [
      wayland
      libxkbcommon
      libx11
      libxcursor
      libxrandr
      libxi
      libGL
      vulkan-loader
      alsa-lib
    ];
  in {
    devShells."x86_64-linux".default = pkgs.mkShell {
      buildInputs = [
          toolchain
          # deps
          pkgs.rust-analyzer
          pkgs.pkg-config
          pkgs.alsa-lib
      ];
      shellHook = ''
        export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH
        # export WINIT_UNIX_BACKEND=wayland 
      '';
      # env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      env.RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
    };
  };
}
