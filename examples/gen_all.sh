#!/bin/bash

cargo build

for target in ocaml erlang elixir graphql; do
  ../target/debug/lore \
    codegen \
    --target $target \
    --output-dir ./gen/$target \
    *.lore
done
