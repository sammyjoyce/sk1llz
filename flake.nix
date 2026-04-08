{
  description = "Development shell for the sk1llz CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { nixpkgs, ... }:
    let
      lib = nixpkgs.lib;
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSystem = lib.genAttrs systems;
    in
    {
      formatter = forEachSystem (system: (import nixpkgs { inherit system; }).nixpkgs-fmt);

      devShells = forEachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          python = pkgs.python3.withPackages (ps: [ ps.pyyaml ]);
          darwinFrameworks = with pkgs.darwin.apple_sdk.frameworks; [
            CoreFoundation
            Security
            SystemConfiguration
          ];
          shell = pkgs.mkShell {
            packages =
              with pkgs;
              [
                cargo
                clippy
                openssl
                pkg-config
                python
                rust-analyzer
                rustc
                rustfmt
                cacert
              ]
              ++ lib.optionals pkgs.stdenv.isDarwin darwinFrameworks;

            OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
            OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
            SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";

            shellHook = ''
              export CARGO_TERM_COLOR=always
              echo "sk1llz CLI dev shell ready."
              echo "  cd cli && cargo check"
              echo "  cd cli && cargo test"
              echo "  python3 scripts/generate_manifest.py"
            '';
          };
        in
        {
          default = shell;
          cli = shell;
        }
      );
    };
}
