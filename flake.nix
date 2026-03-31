{
  description = "Libera Reader Dev Shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    libs = with pkgs; [
      wayland vulkan-loader libxkbcommon fontconfig alsa-lib
      libX11 libXcursor libXrandr libXi libXinerama libxcb gtk3
      glib
      gdk-pixbuf zenity
    ];
  in {
    devShells.${system}.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [ pkg-config clang rustup ];
      buildInputs = libs;
      shellHook = ''
        export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath libs}:$LD_LIBRARY_PATH"
        export WINIT_UNIX_BACKEND=wayland
      '';
    };
  };
}
