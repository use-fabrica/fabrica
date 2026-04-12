{
  description = "Fabrica — Coding agent harness (wgpu + glyphon + taffy)";

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
          # Windowing (winit)
          libX11
          libxcb
          libXcursor
          libXrandr
          libXi
          libxkbcommon
          wayland

          # GPU rendering (wgpu)
          vulkan-loader
          libGL

          # Text rendering (glyphon → cosmic-text)
          fontconfig
          freetype
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
