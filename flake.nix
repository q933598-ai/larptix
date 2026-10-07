{
  description = "Larptrix desktop messenger";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system:
          f (import nixpkgs {
            inherit system;
            config.allowUnfree = true;
          }));
    in
    {
      packages = forAllSystems (pkgs:
        let
          larptrix = pkgs.stdenvNoCC.mkDerivation {
            pname = "larptrix";
            version = "0.4.0";
            src = self;

            nativeBuildInputs = [ pkgs.makeWrapper ];
            dontBuild = true;

            installPhase = ''
              runHook preInstall

              appDir="$out/lib/larptrix"
              mkdir -p "$appDir" "$out/bin" "$out/share/applications"

              cp -r desktop/electron "$appDir/electron"
              cp -r desktop/bootstrap "$appDir/bootstrap"
              cp desktop/package.json "$appDir/package.json"

              makeWrapper ${pkgs.electron}/bin/electron "$out/bin/larptrix" \
                --add-flags "$appDir" \
                --add-flags "--ozone-platform-hint=auto"

              cat > "$out/share/applications/larptrix.desktop" <<EOF
              [Desktop Entry]
              Name=Larptrix
              Comment=End-to-end encrypted Larptrix messenger
              Exec=$out/bin/larptrix
              Terminal=false
              Type=Application
              Categories=Network;InstantMessaging;
              StartupWMClass=Larptrix
              EOF

              runHook postInstall
            '';

            meta = with pkgs.lib; {
              description = "End-to-end encrypted Larptrix desktop messenger";
              homepage = "https://github.com/q933598-ai/larptix";
              license = licenses.mit;
              platforms = platforms.linux;
              mainProgram = "larptrix";
            };
          };
        in {
          default = larptrix;
          larptrix = larptrix;
        });

      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${pkgs.system}.default}/bin/larptrix";
        };
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = [
            pkgs.electron
            pkgs.nodejs_22
            pkgs.cargo
            pkgs.rustc
            pkgs.pkg-config
          ];

          shellHook = ''
            echo "Larptrix development shell"
            echo "Run: npm --prefix desktop run dev"
            echo "Build Linux package: npm --prefix desktop run build:linux"
          '';
        };
      });
    };
}
