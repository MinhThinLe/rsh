{
    description = "A really simple shell";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    };

    outputs = { self, nixpkgs }: {
        devShells = builtins.mapAttrs (system: pkgs: {
            default = pkgs.mkShell {
                nativeBuildInputs = with pkgs; [
                    cargo
                    clippy
                    rust-analyzer
                    rustc
                    rustfmt
                ];

                shellHook = ''
                    if [ -x "$(command -v zsh)" ]; then
                        exec zsh
                    fi
                '';
            };
        }) nixpkgs.legacyPackages;
    };
}
