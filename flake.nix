{
  description = "Fabrica — GPUI application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        libs = with pkgs; [
          # X11
          libX11
          libxcb
          libXcursor
          libXrandr
          libXi

          # Keyboard
          libxkbcommon

          # Wayland
          wayland

          # Graphics (Vulkan + OpenGL)
          vulkan-loader
          libGL

          # Fonts
          fontconfig
          freetype

          # Crypto / Network
          openssl
          zlib
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustc
            cargo
          ];

          buildInputs = libs;

          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libs}";
          LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libs}";

          shellHook = ''
            # Drop into fish if interactive and not already in fish
            if [ -t 0 ] && command -v fish >/dev/null 2>&1 && [ -z "''${FISH_VERSION:-}" ]; then
              exec fish
            fi
          '';
        };
      }
    );
}
