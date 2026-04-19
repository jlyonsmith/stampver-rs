list:
  just --list

test:
  cargo test

cov-json:
  #!/usr/bin/env fish
  cargo clean
  cargo llvm-cov test --json --summary-only --output-path scratch/coverage-summary.json
  set cov_percent (cat scratch/coverage-summary.json | jq '.data[0].totals.lines.percent' | math --scale 2)
  set cov_color (if test $cov_percent -gt 80.0; echo green; else if test $cov_percent -lt 50.0; echo red; else echo yellow; end)
  echo '{"schemaVersion":1,"label":"coverage","message":"'$cov_percent'%","color":"'$cov_color'"}' > coverage.json

cov-html:
  # You might need to run `cargo clean` first to get coverage for the binaries
  cargo llvm-cov test --html --open

doc:
  cargo doc --open

release OPERATION='incrPatch':
  #!/usr/bin/env fish
  function info
    set_color green; echo "👉 "$argv; set_color normal
  end
  function warning
    set_color yellow; echo "🐓 "$argv; set_color normal
  end
  function error
    set_color red; echo "💥 "$argv; set_color normal
  end

  if test ! -e "Cargo.toml"
    error "Cargo.toml file not found"
    exit 1
  end

  info "Checking for uncommitted changes"

  if not git diff-index --quiet HEAD -- > /dev/null 2> /dev/null
    error "There are uncomitted changes - commit or stash them and try again"
    exit 1
  end

  set branch (string trim (git rev-parse --abbrev-ref HEAD 2> /dev/null))
  set name (basename (pwd))

  info "Starting release of '"$name"' on branch '"$branch"'"

  info "Checking out '"$branch"'"
  git checkout $branch

  info "Pulling latest"
  git pull

  mkdir scratch 2> /dev/null

  if not stampver {{OPERATION}} -u -i version.json5
    error "Unable to generation version information"
    exit 1
  end

  set tagName (cat "scratch/version.tag.txt")
  set tagDescription (cat "scratch/version.desc.txt")

  git rev-parse $tagName > /dev/null 2> /dev/null
  if test $status -ne 0; set isNewTag 1; end

  if set -q isNewTag
    info "'"$tagName"' is a new tag"
  else
    warning "Tag '"$tagName"' already exists and will not be moved"
  end

  if test -e 'justfile' -o -e 'Justfile'
    just cov-json
  else
    cargo test
  end

  if test $status -ne 0
    # Rollback
    git checkout $branch .
    error "Tests failed '"$name"' on branch '"$branch"'"
    exit 1
  end

  info "Staging version changes"
  git add :/

  info "Committing version changes"
  git commit -m $tagDescription

  if set -q isNewTag
    info "Tagging"
    git tag -a $tagName -m $tagDescription
  end

  info "Pushing to 'origin'"
  git push --follow-tags

  info "Finished release of '"$name"\' on branch '"$branch"'. You can publish the crate."
  exit 0

# Used to clean up if you accidentally miss something in a release and need to reset the tag
del-last-tag:
  #!/usr/bin/env fish
  set tagName (cat "scratch/version.tag.txt")

  git tag -d $tagName
  git push origin --delete $tagName
