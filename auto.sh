#!/usr/bin/env bash

export PROJDIR=${PROJDIR:-$(git rev-parse --show-toplevel)}
export IN_NIX_SHELL=${IN_NIX_SHELL:-nopes}
export PYTHONPATH=${PROJDIR}/dj:${PROJDIR}

export PATH=${PROJDIR}/venv/bin:${PROJDIR}/.cargo/bin:$PATH:${PROJDIR}/node_modules/.bin

export CARGO_HOME="${PROJDIR}/.cargo";

export STARSHIP_CONFIG="${PROJDIR}/.starship.toml"

setup() {
    pushd2 /

    test -f venv/bin/python || python -m venv venv
    test -f venv/bin/doit || pip install -r requirements.txt
    test -f .git/hooks/pre-commit || pre-commit install

    mkdir -p .cargo;

    eval "$(starship init zsh)"

    if [[ ! -f ".cargo/.cachix" ]] then
      # use cachix to install binaries instead of compiling if possible
      # not sure if adding cachix in shell.nix is the right way or this
      export NIX_IGNORE_SYMLINK_STORE=1
      if [ -e ~/.nix-profile/etc/profile.d/nix.sh ];
        then . ~/.nix-profile/etc/profile.d/nix.sh;
      else
        echo "nix not installed? this would all fail."
      fi
      ~/.nix-profile/bin/nix-env -iA cachix -f https://cachix.org/api/v1/install
      ~/.nix-profile/bin/cachix use srid
      touch ".cargo/.cachix"
    fi

    doit
    popd2
}

pushd2() {
    PUSHED=$(pwd)
    cd "${PROJDIR}""$1" >> /dev/null || return
}

popd2() {
    cd "${PUSHED:-$PROJDIR}" >> /dev/null || return
    unset PUSHED
}

o() {
    cd "${PROJDIR}" || return
}


alias serve="cargo run -- --test"
alias open=/usr/bin/open
alias pbcopy=/usr/bin/pbcopy
alias gst="git status"
alias gd="git diff"
alias gdc="git diff --cached"
alias gda="git diff HEAD"
alias gp="git push"
alias less=bat
alias cat=bat
alias ls=exa
alias lc="tokei -s lines | bat"
alias cc="cargo fmt && cargo check"
alias git-clean='git fetch -p && for branch in $(git for-each-ref --format '\''%(refname) %(upstream:track)'\'' refs/heads | awk '\''$2 == "[gone]" {sub("refs/heads/", "", $1); print $1}'\''); do git branch -D $branch; done'
alias git-clean-dry-run='git fetch -p && for branch in $(git for-each-ref --format '\''%(refname) %(upstream:track)'\'' refs/heads | awk '\''$2 == "[gone]" {sub("refs/heads/", "", $1); print $1}'\''); do echo git branch -D $branch; done'
setup
