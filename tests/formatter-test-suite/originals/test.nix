{ pkgs ? import <nixpkgs> {} }:

# ── CASE 1: Basic derivation ──────────────────────────────────────────────
pkgs.mkDerivation rec {
  pname = "my-app";
  version = "1.0.0";

  src = pkgs.fetchFromGitHub {
    owner   = "myuser";
    repo    = "my-app";
    rev     = "v${version}";
    sha256  = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  };

  buildInputs = with pkgs; [
    nodejs
    yarn
    git
  ];

  nativeBuildInputs = with pkgs; [
    makeWrapper
  ];

  buildPhase = ''
    yarn install --frozen-lockfile
    yarn build
  '';

  installPhase = ''
    mkdir -p $out/bin $out/lib
    cp -r dist/* $out/lib/
    makeWrapper ${pkgs.nodejs}/bin/node $out/bin/my-app \
      --add-flags "$out/lib/server.js"
  '';

  meta = with pkgs.lib; {
    description = "My test application";
    license = licenses.mit;
    maintainers = [ maintainers.alice ];
  };
}

# ── CASE 2: Let expression and attribute set ──────────────────────────────
let
  config = {
    host    = "localhost";
    port    = 8080;
    debug   =   true;
    workers = 4;
  };

  mkConfig = { host, port, debug ? false, workers ? 1 }:
    "host=${host} port=${toString port} debug=${toString debug}";

in {
  # ── CASE 3: With expression ───────────────────────────────────────────
  shell = with pkgs; mkShell {
    buildInputs = [
      nodejs_20
      python311
      rustup
      git
      curl
    ];

    shellHook = ''
      echo "Development shell activated"
      export NODE_ENV=development
    '';
  };

  # ── CASE 4: Function with default args ────────────────────────────────
  greet = name: greeting:
    "${greeting}, ${name}!";

  # ── CASE 5: List operations ───────────────────────────────────────────
  numbers = builtins.genList (x: x * 2) 10;
  evens = builtins.filter (n: builtins.mod n 2 == 0) numbers;
}
