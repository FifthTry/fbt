#!/usr/bin/env bash

source auto.sh
test -f ~/.zshrc && source ~/.zshrc
export HISTFILE=~/.zsh_history
export PS1="]$PS1"
