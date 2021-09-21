#!/bin/bash

cargo build

for target in rescript ocaml erlang elixir graphql; do
  ../target/debug/lore \
    codegen \
    --target $target \
    --output-dir ./gen/$target \
    *.lore
done
