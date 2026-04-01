{
  description = "Flake for rust development";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
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
      buildInputs = with pkgs; [
          cargo rustc rustfmt clippy rust-analyzer
          # deps
          pkg-config
          alsa-lib
      ];
      shellHook = ''
        export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH
        # export WINIT_UNIX_BACKEND=wayland 
      '';
      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    };
  };
}
