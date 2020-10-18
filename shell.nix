let
  system = import <nixpkgs> {};
  moz_overlay = import (
    builtins.fetchTarball {
        url = https://s3.ap-south-1.amazonaws.com/downloads.fifthtry.com/nixpkgs-mozilla-efda5b357451dbb0431f983cca679ae3cd9b9829.tar.gz;
        sha256 = "11wqrg86g3qva67vnk81ynvqyfj0zxk83cbrf0p9hsvxiwxs8469";
    }
  );
  nixpkgs = import (
    builtins.fetchTarball {
        url = https://s3.ap-south-1.amazonaws.com/downloads.fifthtry.com/nixpkgs-20.03.tar.gz;
        sha256 = "0yn3yvzy69nlx72rz2hi05jpjlsf9pjzdbdy4rgfpa0r0b494sfb";
    }
  ) {
    overlays = [ moz_overlay ];
    config = { allowUnfree = true; };
  };
  frameworks = nixpkgs.darwin.apple_sdk.frameworks;
  rust = (
    nixpkgs.rustChannelOf {
      rustToolchain = ./rust-toolchain;
    }
  ).rust.override {
      extensions = [
        "clippy-preview"
        "rust-src"
      ];
  };
in
  with nixpkgs;
  stdenv.mkDerivation {
    name = "amitu-env";
    buildInputs = [ rust ];

    nativeBuildInputs = [
      elmPackages.elm-format
      elmPackages.elm
      stdenv.cc.cc.lib
      clang
      llvm
      file
      nodejs
      starship
      geckodriver
      exa
      zsh
      wget
      locale
      vim
      less
      awscli
      which
      tree
      curl
      ripgrep
      tokei
      man
      bat
      git
      gitAndTools.diff-so-fancy
      heroku
      openssl
      pkgconfig
      perl
      nixpkgs-fmt
      cacert
      libiconv
      gnupg
      # ngrok

      (postgresql_12.withPackages (p: [ p.postgis ]))
      python38
      python38Packages.psycopg2
    ]
    ++ (
         stdenv.lib.optionals stdenv.isDarwin [
           frameworks.CoreServices
           frameworks.Security
           frameworks.CoreFoundation
           frameworks.Foundation
           frameworks.AppKit
         ]
    );

    RUST_BACKTRACE = 1;
    SOURCE_DATE_EPOCH = 315532800;

    shellHook = (
      if pkgs.stdenv.isDarwin then
        ''export NIX_LDFLAGS="-F${frameworks.AppKit}/Library/Frameworks -framework AppKit -F${frameworks.CoreServices}/Library/Frameworks -framework CoreServices -F${frameworks.CoreFoundation}/Library/Frameworks -framework CoreFoundation $NIX_LDFLAGS";''
      else
        ""
    )
      +
    ''
      export LD_LIBRARY_PATH=$(rustc --print sysroot)/lib:${stdenv.cc.cc.lib}/lib:$LD_LIBRARY_PATH;
      export LIBCLANG_PATH="${llvmPackages.libclang}/lib"
      export ZDOTDIR=`pwd`;
      export HISTFILE=~/.zsh_history
      export CARGO_TARGET_DIR=`pwd`/target-nix
      echo "Using ${python38.name}, ${elmPackages.elm.name}, ${rust.name} and ${postgresql_12.name}."
      unset MACOSX_DEPLOYMENT_TARGET
    '';
  }
