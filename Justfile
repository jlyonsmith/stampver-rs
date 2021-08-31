coverage OPEN='':
  #!/usr/bin/env bash
  export RUSTFLAGS="-Zinstrument-coverage"
  export LLVM_PROFILE_FILE="$(pwd)/scratch/$(whoami)-%p-%m.profraw"
  cargo test --tests
  grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/ --excl-start '^//\s*\{grcov-excl-start\}' --excl-stop '^//\s*\{grcov-excl-end\}'
  cp ./target/debug/coverage/coverage.json ./coverage.json
  if [[ "{{OPEN}}" == "--open" ]]; then
    open target/debug/coverage/index.html
  fi

doc:
  cargo doc --open

release OPERATION='incrPatch':
  deno run --unstable --allow-run --allow-read https://raw.githubusercontent.com/jlyonsmith/deno-scripts/main/rust-release.ts {{OPERATION}}
