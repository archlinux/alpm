#!/usr/bin/env -S just --working-directory . --justfile
# Load project-specific properties from the `.env` file

set dotenv-load := true

# Whether to run ignored tests (set to "true" to run ignored tests)

ignored := "false"

# The output directory for documentation artifacts

output_dir := "output"

# Lists all available recipes.
default:
    just --list

# Adds pre-commit and pre-push git hooks
[private]
add-hooks:
    #!/usr/bin/env bash
    set -euo pipefail

    echo just run-pre-commit-hook > .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit

    echo just run-pre-push-hook > .git/hooks/pre-push
    chmod +x .git/hooks/pre-push

    cat > .git/hooks/prepare-commit-msg <<'EOL'
    #!/bin/sh

    COMMIT_MSG_FILE=$1
    COMMIT_SOURCE=$2

    SOB=$(git var GIT_COMMITTER_IDENT | sed -n 's/^\(.*>\).*$/Signed-off-by: \1/p')
    git interpret-trailers --in-place --trailer "$SOB" "$COMMIT_MSG_FILE"
    if test -z "$COMMIT_SOURCE"; then
        /usr/bin/perl -i.bak -pe 'print "\n" if !$first_line++' "$COMMIT_MSG_FILE"
    fi
    EOL
    chmod +x .git/hooks/prepare-commit-msg

# Updates the local cargo index and displays which crates would be updated
[private]
dry-update:
    cargo update --dry-run --verbose

# Ensures that one or more required commands are installed
[private]
ensure-command +command:
    #!/usr/bin/env bash
    set -euo pipefail

    read -r -a commands <<< "{{ command }}"

    for cmd in "${commands[@]}"; do
        if ! command -v "$cmd" > /dev/null 2>&1 ; then
            printf "Couldn't find required executable '%s'\n" "$cmd" >&2
            exit 1
        fi
    done

# Retrieves the configured target directory for cargo.
[private]
get-cargo-target-directory:
    just ensure-command cargo jq
    cargo metadata --format-version 1 | jq -r  .target_directory

# Gets names of all workspace members
[private]
get-workspace-members:
    cargo metadata --format-version=1 |jq -r '.workspace_members[] | capture("/(?<name>[a-z-]+)#.*").name'

# Gets metadata version of a workspace member
[private]
get-workspace-member-version package:
    #!/usr/bin/env bash
    set -euo pipefail

    readonly version="$(cargo metadata --format-version=1 |jq -r --arg pkg {{ package }} '.workspace_members[] | capture("/(?<name>[a-z-]+)#(?<version>[0-9.]+)") | select(.name == $pkg).version')"

    if [[ -z "$version" ]]; then
        printf "No version found for package %s\n" {{ package }} >&2
        exit 1
    fi

    printf "$version\n"

# Checks if a string matches a workspace member exactly
[private]
is-workspace-member package:
    #!/usr/bin/env bash
    set -euo pipefail

    mapfile -t workspace_members < <(just get-workspace-members 2>/dev/null)

    for name in "${workspace_members[@]}"; do
        if [[ "$name" == {{ package }} ]]; then
            exit 0
        fi
    done
    exit 1

# Runs checks and tests before creating a commit.
[private]
run-pre-commit-hook: check docs test test-docs

# Runs checks before pushing commits to remote repository.
[private]
run-pre-push-hook: check-commits

# Builds the documentation book using mdbook and stages all necessary rustdocs alongside
[group('build')]
build-book:
    #!/usr/bin/env bash
    set -euo pipefail

    just ensure-command cargo jq mdbook mdbook-mermaid

    target_dir="$(just get-cargo-target-directory)"
    readonly output_dir="{{ output_dir }}"
    readonly rustdoc_dir="$output_dir/docs/rustdoc/"
    mapfile -t workspace_members < <(just get-workspace-members 2>/dev/null)

    just docs
    mdbook-mermaid install resources/docs/
    mdbook build resources/docs/

    # move rust docs to their own namespaced dir
    mkdir -p "$rustdoc_dir"
    for name in "${workspace_members[@]}"; do
        cp -r "$target_dir/doc/${name//-/_}" "$rustdoc_dir"
    done
    cp -r "$target_dir/doc/"{search.desc,src,static.files,trait.impl,type.impl} "$rustdoc_dir"
    cp -r "$target_dir/doc/"*.{js,html} "$rustdoc_dir"

# Build local documentation
[group('build')]
docs:
    RUSTDOCFLAGS='-D warnings' cargo doc --document-private-items --no-deps

# Render `manpages`, `shell_completions` or `specifications` (`kind`) of a given package (`pkg`).
[group('build')]
generate kind pkg:
    #!/usr/bin/bash

    set -Eeuo pipefail

    readonly output_dir="{{ output_dir }}"
    mkdir --parents "$output_dir"

    kind="{{ kind }}"

    script="$(mktemp --suffix=.ers)"
    readonly script="$script"

    # remove temporary script file on exit
    cleanup() (
      if [[ -n "${script:-}" ]]; then
        rm -f "$script"
      fi
    )

    trap cleanup EXIT

    case "$kind" in
        manpages|shell_completions)
            sed "s/PKG/{{ pkg }}/;s#PATH#$PWD/{{ pkg }}#g;s/KIND/{{ kind }}/g" > "$script" < .rust-script/allgen.ers
            ;;
        specifications)
            sed "s/PKG/{{ pkg }}/;s#PATH#$PWD/{{ pkg }}#g;s/KIND/{{ kind }}/g" > "$script" < .rust-script/mandown.ers
            # override kind, as we are in fact generating manpages
            kind="manpages"
            ;;
        *)
            printf 'Only "manpages", "shell_completions" or "specifications" are supported targets.\n'
            exit 1
    esac

    chmod +x "$script"
    "$script" "$output_dir/$kind"

# Generates shell completions
[group('build')]
generate-completions:
    just generate shell_completions alpm-buildinfo
    just generate shell_completions alpm-mtree
    just generate shell_completions alpm-pkginfo
    just generate shell_completions alpm-srcinfo

# Generates all manpages and specifications
[group('build')]
generate-manpages-and-specs:
    just generate manpages alpm-buildinfo
    just generate manpages alpm-mtree
    just generate manpages alpm-pkginfo
    just generate manpages alpm-srcinfo
    just generate specifications alpm-buildinfo
    just generate specifications alpm-mtree
    just generate specifications alpm-package
    just generate specifications alpm-pkginfo
    just generate specifications alpm-srcinfo
    just generate specifications alpm-state-repo
    just generate specifications alpm-types

# Checks source code formatting
[group('check')]
check-formatting:
    just ensure-command taplo

    just --unstable --fmt --check
    # We're using nightly to properly group imports, see rustfmt.toml
    cargo +nightly fmt -- --check

    taplo format --check

# Runs all check targets
[group('check')]
check: check-spelling check-formatting check-shell-code check-rust-code check-rust-derives check-unused-deps check-dependencies check-licenses check-links

# Checks commit messages for correctness
[group('check')]
check-commits:
    #!/usr/bin/env bash
    set -euo pipefail

    just ensure-command codespell cog committed rg

    readonly default_branch="${CI_DEFAULT_BRANCH:-main}"

    if ! git rev-parse --verify "origin/$default_branch" > /dev/null 2>&1; then
        printf "The default branch '%s' does not exist!\n" "$default_branch" >&2
        exit 1
    fi

    tmpdir="$(mktemp --dry-run --directory)"
    readonly check_tmpdir="$tmpdir"
    mkdir -p "$check_tmpdir"

    # remove temporary dir on exit
    cleanup() (
      if [[ -n "${check_tmpdir:-}" ]]; then
        rm -rf "${check_tmpdir}"
      fi
    )

    trap cleanup EXIT

    for commit in $(git rev-list "origin/${default_branch}.."); do
        printf "Checking commit %s\n" "$commit"

        commit_message="$(git show -s --format=%B "$commit")"
        codespell_config="$(mktemp --tmpdir="$check_tmpdir")"

        # either use the commit's .codespellrc or create one
        if git show "$commit:.codespellrc" > /dev/null 2>&1; then
            git show "$commit:.codespellrc" > "$codespell_config"
        else
            printf "[codespell]\nskip = .cargo,.git,target,.env,Cargo.lock\nignore-words-list = crate\n" > "$codespell_config"
        fi

        if ! rg -q "Signed-off-by: " <<< "$commit_message"; then
            printf "Commit %s ❌️\n" "$commit" >&2
            printf "The commit message lacks a \"Signed-off-by\" line.\n" >&2
            printf "%s\n" \
                "  Please use:" \
                "    git rebase --signoff main && git push --force-with-lease" \
                "  See https://developercertificate.org/ for more details." >&2
            exit 1
        elif ! codespell --config "$codespell_config" - <<< "$commit_message"; then
            printf "Commit %s ❌️\n" "$commit" >&2
            printf "The spelling of the commit message needs improvement.\n" >&2
            exit 1
        elif ! cog verify "$commit_message"; then
            printf "Commit %s ❌️\n" "$commit" >&2
            printf "%s\n" \
                "The commit message is not a conventional commit message:" \
                "$commit_message" \
                "See https://www.conventionalcommits.org/en/v1.0.0/ for more details." >&2
            exit 1
        elif ! committed "$commit"; then
            printf "Commit %s ❌️\n" "$commit" >&2
            printf "%s\n" \
                "The commit message does not meet the required standards:" \
                "$commit_message"
            exit 1
        else
            printf "Commit %s ✅️\n\n" "$commit"
        fi
    done

# Checks for issues with dependencies
[group('check')]
check-dependencies: dry-update
    cargo deny --all-features check

# Checks licensing status
[group('check')]
check-licenses:
    just ensure-command reuse
    reuse lint

# Check for stale links in documentation
[group('check')]
check-links:
    just ensure-command lychee
    lychee .

# Checks the Rust source code using cargo-clippy.
[group('check')]
check-rust-code:
    just ensure-command cargo cargo-clippy mold
    cargo clippy --all-features --all-targets --workspace -- -D warnings

# Checks shell code using shellcheck.
[group('check')]
check-shell-code:
    just check-shell-readme alpm-buildinfo
    just check-shell-readme alpm-mtree
    just check-shell-readme alpm-pkginfo
    just check-shell-readme alpm-srcinfo

    just check-shell-recipe 'test-readme alpm-buildinfo'
    just check-shell-recipe 'test-readme alpm-pkginfo'
    just check-shell-recipe 'test-readme alpm-srcinfo'
    just check-shell-recipe build-book
    just check-shell-recipe check-commits
    just check-shell-recipe check-unused-deps
    just check-shell-recipe ci-publish
    just check-shell-recipe 'generate shell_completions alpm-buildinfo'
    just check-shell-recipe install-pacman-dev-packages
    just check-shell-recipe 'is-workspace-member alpm-buildinfo'
    just check-shell-recipe 'prepare-release alpm-buildinfo'
    just check-shell-recipe 'release alpm-buildinfo'
    just check-shell-recipe flaky
    just check-shell-recipe test
    just check-shell-recipe 'ensure-command test'

    just check-shell-script alpm-srcinfo/tests/generate_srcinfo.bash
    just check-shell-script .cargo/runner.sh

# Checks the script examples of a project's README using shellcheck.
[group('check')]
check-shell-readme project:
    just ensure-command shellcheck tangler
    tangler bash < {{ project }}/README.md | shellcheck --shell bash -

# Checks justfile recipe relying on shell semantics using shellcheck.
[group('check')]
check-shell-recipe recipe:
    just ensure-command rg shellcheck
    just -vv -n {{ recipe }} 2>&1 | rg -v '===> Running recipe' | shellcheck -

# Checks a shell script using shellcheck.
[group('check')]
check-shell-script file:
    just ensure-command shellcheck
    shellcheck --shell bash {{ file }}

# Checks common spelling mistakes
[group('check')]
check-spelling:
    codespell

# Checks for unused dependencies
[group('check')]
check-unused-deps:
    #!/usr/bin/env bash
    set -euxo pipefail
    just ensure-command cargo-machete

    for name in $(just get-workspace-members); do
        cargo machete "$name"
    done

# Checks for consistent sorting of rust derives
[group('check')]
check-rust-derives:
    cargo sort-derives --check

# Adds needed git configuration for the local repository
[group('dev')]
configure-git:
    # Enforce gpg signed keys for this repository
    git config commit.gpgsign true

    just add-hooks

# Installs all tools required for development
[group('dev')]
dev-install: install-pacman-dev-packages install-rust-dev-tools

# Installs all binaries of the workspace
[group('dev')]
install-workspace-binaries:
    #!/usr/bin/env bash
    set -euo pipefail

    mapfile -t workspace_members < <(just get-workspace-members 2>/dev/null)

    # Workspace members without a binary
    ignored_members=(
        alpm-common
        alpm-package
        alpm-parsers
        alpm-state-repo
        alpm-types
    )

    for name in "${workspace_members[@]}"; do
        # Make sure we don't try to install any of the ignored binaries.
        skip=false
        for ignored in "${ignored_members[@]}"; do
            if [[ "$name" == "$ignored" ]]; then
                skip=true
                break
            fi
        done

        if [ "$skip" = true ]; then
            continue
        fi

        echo "Installing $name"
        cargo install --locked --path "$name"
    done

# Fixes common issues. Files need to be git add'ed
[group('dev')]
fix:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! git diff-files --quiet ; then
        echo "Working tree has changes. Please stage them: git add ."
        exit 1
    fi

    codespell --write-changes
    just --unstable --fmt
    cargo clippy --fix --allow-staged

    # fmt must be last as clippy's changes may break formatting
    cargo +nightly fmt

# Installs development packages using pacman
[group('dev')]
install-pacman-dev-packages:
    #!/usr/bin/env bash
    set -euo pipefail

    # All packages are set in the `.env` file
    source .env

    # Read all packages into an array.
    read -r -d '' -a packages < <(printf '%s\0' "$PACMAN_PACKAGES")

    # Deduplicate using an associated array
    declare -A unique_packages
    for package in "${packages[@]}"; do
        if [[ ! "${unique_packages[$package]+_}" ]]; then
            unique_packages["$package"]=1
        fi
    done

    pacman -S --needed --noconfirm "${!unique_packages[@]}"
    # run0 pacman -S --needed --noconfirm "${!unique_packages[@]}"

# Installs all Rust tools required for development
[group('dev')]
install-rust-dev-tools:
    rustup default stable
    rustup component add clippy
    # Install nightly as we use it for formatting rules.
    rustup toolchain install nightly
    rustup component add --toolchain nightly rustfmt
    # llvm-tools-preview for code coverage
    rustup component add llvm-tools-preview

# Continuously run integration tests for a given number of rounds
[group('test')]
flaky test='just test-readme alpm-buildinfo' rounds='999999999999':
    #!/usr/bin/bash
    set -euo pipefail

    seq 1 {{ rounds }} | while read -r counter; do
      printf "Running flaky tests (%d/{{ rounds }})...\n" "$counter"
      sleep 1
      {{ test }}
      echo
    done

# Serves the documentation book using miniserve
[group('dev')]
serve-book: build-book
    just ensure-command miniserve
    miniserve --index=index.html {{ output_dir }}/docs

# Watches the documentation book contents and rebuilds on change using mdbook (useful for development)
[group('dev')]
watch-book:
    just ensure-command watchexec
    watchexec --exts md,toml,js --delay-run 5s -- just build-book

# Runs integration tests guarded by the `_containerized-integration-test` feature, located in modules named `containerized` (accepts `cargo nextest run` options via `options`).
[group('test')]
containerized-integration-tests *options:
    just ensure-command bash cargo cargo-nextest podman
    cargo nextest run --features _containerized-integration-test --filterset 'kind(test) and binary_id(/::containerized$/)' {{ options }}

# Runs all unit tests. By default ignored tests are not run. Run with `ignored=true` to run only ignored tests
[group('test')]
test:
    #!/usr/bin/env bash
    set -euxo pipefail

    readonly ignored="{{ ignored }}"

    if [[ "${ignored}" == "true" ]]; then
        cargo nextest run --workspace --run-ignored all
    else
        cargo nextest run --workspace
    fi

# Creates code coverage for all projects.
[group('test')]
test-coverage mode="nodoc":
    #!/usr/bin/env bash
    set -euxo pipefail

    target_dir="$(just get-cargo-target-directory)"

    # Clean any previous code coverage run.
    rm -rf "$target_dir/llvm-cov"
    mkdir -p "$target_dir/llvm-cov"

    just ensure-command cargo-llvm-cov cargo-nextest
    # Run nextest coverage
    cargo llvm-cov --no-report nextest

    # The chosen reporting style (defaults to without doctest coverage)
    reporting_style="without doctest coverage"

    # Options for cargo
    cargo_options=()

    # The dev-scripts aren't included in the test coverage report yet.
    # See <https://gitlab.archlinux.org/archlinux/alpm/alpm/-/issues/156>
    _ignored=(--ignore-filename-regex dev-scripts)

    # Options for creating cobertura coverage report with cargo-llvm-cov
    cargo_llvm_cov_cobertura_options=(
        --cobertura
        "${_ignored[@]}"
        --output-path "$target_dir/llvm-cov/cobertura-coverage.xml"
    )

    # Options for creating HTML coverage report with cargo-llvm-cov
    cargo_llvm_cov_html_options=(
        --html
        "${_ignored[@]}"
    )

    # Options for creating coverage report summary with cargo-llvm-cov
    cargo_llvm_cov_summary_options=(
        "${_ignored[@]}"
        --json
        --summary-only
    )

    if [[ "{{ mode }}" == "doc" ]]; then
        reporting_style="with doctest coverage"
        # The support for doctest coverage is a nightly feature
        cargo_options=(+nightly)
        cargo_llvm_cov_cobertura_options+=(--doctests)
        cargo_llvm_cov_html_options+=(--doctests)
        cargo_llvm_cov_summary_options+=(--doctests)

        # nextest coverage needs to be manually merged with doctest coverage:
        # https://nexte.st/docs/integrations/test-coverage/?h=doc#collecting-coverage-data-from-doctests
        cargo "${cargo_options[@]}" llvm-cov --no-report --doc
    fi

    printf "Creating report %s\n" "$reporting_style"

    # Create cobertura coverage report
    cargo "${cargo_options[@]}" llvm-cov report "${cargo_llvm_cov_cobertura_options[@]}"

    # Create HTML coverage report 
    cargo "${cargo_options[@]}" llvm-cov report "${cargo_llvm_cov_html_options[@]}"

    # Get total coverage percentage from summary
    percentage="$(cargo "${cargo_options[@]}" llvm-cov report "${cargo_llvm_cov_summary_options[@]}" | jq '.data[0].totals.lines.percent')"

    # Trim percentage to 4 decimal places.
    percentage=$(printf "%.4f\n" $percentage)

    # Writes to target/coverage-metrics.txt for Gitlab CI metric consumption.
    # https://docs.gitlab.com/ci/testing/metrics_reports/
    printf "Test-coverage ${percentage}\n" > "$target_dir/coverage-metrics.txt"
    printf "Test-coverage: ${percentage}%%\n"

# Runs all doc tests
[group('test')]
test-docs:
    just ensure-command cargo
    cargo test --doc

# Runs per project end-to-end tests found in a project README.md
[group('test')]
test-readme project:
    #!/usr/bin/env bash
    set -euxo pipefail

    CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"

    install_executables() {
        printf "Installing executables of %s...\n" "{{ project }}"
        cargo install --locked --path {{ project }}
    }

    install_executables

    PATH="$CARGO_HOME/bin:$PATH"
    printf "PATH=%s\n" "$PATH"

    cd {{ project }} && PATH="$PATH" tangler bash < README.md | bash -euxo pipefail -

# Run end-to-end tests for README files of projects
[group('test')]
test-readmes:
    just test-readme alpm-buildinfo
    just test-readme alpm-pkginfo
    just test-readme alpm-srcinfo

# Publishes a crate in the workspace from GitLab CI in a pipeline for tags
[group('release')]
ci-publish:
    #!/usr/bin/env bash
    set -euo pipefail

    # an auth token with publishing capabilities is expected to be set in GitLab project settings
    readonly token="${CARGO_REGISTRY_TOKEN:-}"
    # rely on predefined variable to retrieve git tag: https://docs.gitlab.com/ee/ci/variables/predefined_variables.html
    readonly tag="${CI_COMMIT_TAG:-}"
    readonly crate="${tag//\/*/}"
    readonly version="${tag#*/}"

    just ensure-command cargo mold

    if [[ -z "$tag" ]]; then
        printf "There is no tag!\n" >&2
        exit 1
    fi
    if [[ -z "$token" ]]; then
        printf "There is no token for crates.io!\n" >&2
        exit 1
    fi
    if ! just is-workspace-member "$crate" &>/dev/null; then
        printf "The crate %s is not a workspace member of the project!\n" "$crate" >&2
        exit 1
    fi

    current_member_version="$(just get-workspace-member-version "$crate" 2>/dev/null)"
    readonly current_member_version="$current_member_version"
    if [[ "$version" != "$current_member_version" ]]; then
        printf "Current version in metadata of crate %s (%s) does not match the version from the tag (%s)!\n" "$crate" "$current_member_version" "$version"
        exit 1
    fi

    printf "Found tag %s (crate %s in version %s).\n" "$tag" "$crate" "$version"
    cargo publish -p "$crate"

# Prepares the release of a crate by updating dependencies, incrementing the crate version and creating a changelog entry (optionally, the version can be set explicitly)
[group('release')]
prepare-release package version="":
    #!/usr/bin/env bash
    set -euo pipefail

    readonly package_name="{{ package }}"
    if [[ -z "$package_name" ]]; then
        printf "No package name provided!\n"
        exit 1
    fi
    readonly package_version="{{ version }}"
    branch_name=""

    just ensure-command git release-plz

    release-plz update -u -p "$package_name"

    # NOTE: When setting the version specifically, we are likely in a situation where `release-plz` did not detect a version change (e.g. when only changes to top-level files took place since last release).
    # In this case we are fine to potentially have no changes in the CHANGELOG.md or having to adjust it manually afterwards.
    if [[ -n "$package_version" ]]; then
        release-plz set-version "${package_name}@${package_version}"
    fi

    # make sure that the current version would be publishable, but ignore files not added to git
    cargo publish -p "$package_name" --dry-run --allow-dirty

    updated_package_version="$(just get-workspace-member-version "$package_name")"
    readonly updated_package_version="$updated_package_version"

    if [[ -n "$package_version" ]]; then
        branch_name="release/$package_name/$package_version"
    else
        branch_name="release/$package_name/$updated_package_version"
    fi
    git checkout -b "$branch_name"

    git add Cargo.* "$package_name"/{Cargo.toml,CHANGELOG.md}
    git commit --gpg-sign --signoff --message "chore: Upgrade $package_name crate to $updated_package_version"
    git push --set-upstream origin "$branch_name"

# Creates a release of a crate in the workspace by creating a tag and pushing it
[group('release')]
release package:
    #!/usr/bin/env bash
    set -euo pipefail

    package_version="$(just get-workspace-member-version {{ package }})"
    readonly package_version="$package_version"
    if [[ -z "$package_version" ]]; then
        exit 1
    fi
    readonly current_version="{{ package }}/$package_version"

    just ensure-command git

    if [[ -n "$(git tag -l "$current_version")" ]]; then
        printf "The tag %s exists already!\n" "$current_version" >&2
        exit 1
    fi

    printf "Creating tag %s...\n" "$current_version"
    git tag -s "$current_version" -m "$current_version"
    printf "Pushing tag %s...\n" "$current_version"
    git push origin refs/tags/"$current_version"
