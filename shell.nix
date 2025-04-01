let
  pkgs = import <nixpkgs> { };
  libPath =
    with pkgs;
    lib.makeLibraryPath [
      libGL
      libxkbcommon
      wayland
      glib
      cairo
      pango
    ];
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    pkg-config
    alsa-lib
    systemd.dev
    cairo
    gtk3
    gdk-pixbuf
    atk
    pango
  ];

  LD_LIBRARY_PATH = libPath;
}
