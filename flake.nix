{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    rust-overlay,
  }:
    utils.lib.eachDefaultSystem (system: let
      overlays = [ (import rust-overlay) ];
      pkgs = (import nixpkgs) {
        inherit system overlays;
      };
      lib = pkgs.lib;

      msrv = (fromTOML (readFile ./Cargo.toml)).package.rust-version;
      inherit (builtins) fromTOML readFile;
      toolchain = pkgs.rust-bin.stable.latest.default;
      msrvToolchain = pkgs.rust-bin.stable."${msrv}".default;

      naersk' = pkgs.callPackage naersk {
        rustc = toolchain;
        cargo = toolchain;
      };
      msrvNaersk = pkgs.callPackage naersk {
        rustc = msrvToolchain;
        cargo = msrvToolchain;
      };

      src = lib.sources.sourceByRegex (lib.cleanSource ./.) ["Cargo.*" "(src|benches|tests|examples|data)(/.*)?"];
      nearskOpt = {
        pname = "vmdl";
        root = src;
      };
      exampleBuildInputs = with pkgs; [
        freetype
        pkg-config
        cmake
        fontconfig
        xorg.libX11
        xorg.libXcursor
        xorg.libXrandr
        xorg.libXi
        glew-egl
        egl-wayland
        libGL
        openssl
      ];
    in rec {
      packages = {
        check = naersk'.buildPackage (nearskOpt // {
          mode = "check";
        });
        clippy = naersk'.buildPackage (nearskOpt // {
          mode = "clippy";
        });
        test = naersk'.buildPackage (nearskOpt // {
          release = false;
          mode = "test";
          nativeBuildInputs = exampleBuildInputs;
        });
        msrv = msrvNaersk.buildPackage (nearskOpt // {
          mode = "check";
        });
      };
      devShells.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          rustc
          cargo
          bacon
          cargo-edit
          cargo-outdated
          clippy
          cargo-audit
          cargo-msrv
          cargo-semver-checks
        ] ++ exampleBuildInputs;

        LD_LIBRARY_PATH = with pkgs; "/run/opengl-driver/lib/:${lib.makeLibraryPath ([libGL libGLU])}";
      };
    });
}
