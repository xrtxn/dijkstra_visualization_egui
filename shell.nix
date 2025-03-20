let
  pkgs = import <nixpkgs> { };
  libPath =
    with pkgs;
    lib.makeLibraryPath [
      libGL
      libxkbcommon
      wayland
    ];
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    pkg-config
    alsa-lib
    systemd.dev
  ];

  LD_LIBRARY_PATH = libPath;
}
