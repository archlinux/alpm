#!/usr/bin/env -S just --working-directory . --justfile
# Load project-specific properties from the `.env` file

set dotenv-load := true

# Whether to run ignored tests (set to "true" to run ignored tests)

ignored := "false"

# The output directory for documentation artifacts

output_dir := "output"

# Runs all checks and tests. Since this is the first recipe it is run by default.
run-pre-commit-hook: check docs test test-docs

# Runs all check targets
check: check-spelling check-formatting lint check-unused-deps check-dependencies check-licenses check-links

# Faster checks need to be executed first for better UX.  For example
# codespell is very fast. cargo fmt does not need to download crates etc.

# Installs all tools required for development
dev-install: install-pacman-dev-packages install-rust-dev-tools

# Installs development packages using pacman
install-pacman-dev-packages:
    # All packages are set in the `.env` file
    run0 pacman -S --needed --noconfirm $PACMAN_PACKAGES

# Installs all Rust tools required for development
install-rust-dev-tools:
    rustup default stable
    rustup component add clippy
    # Install nightly as we use it for formatting rules.
    rustup toolchain install nightly
    rustup component add --toolchain nightly rustfmt
    # llvm-tools-preview for code coverage
    rustup component add llvm-tools-preview

# Checks commit messages for correctness
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

# Runs checks before pushing commits to remote repository.
run-pre-push-hook: check-commits

# Checks common spelling mistakes
check-spelling:
    codespell

# Retrieves the configured target directory for cargo.
[private]
get-cargo-target-directory:
    just ensure-command cargo jq
    cargo metadata --format-version 1 | jq -r  .target_directory

# Gets names of all workspace members
[private]
get-workspace-members:
    cargo metadata --format-version=1 |jq -r '.workspace_members[] | capture("/(?<name>[a-z-]+)#.*").name'

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

# Checks for unused dependencies
check-unused-deps:
    #!/usr/bin/env bash
    set -euxo pipefail
    just ensure-command cargo-machete

    for name in $(just get-workspace-members); do
        cargo machete "$name"
    done

# Checks source code formatting
check-formatting:
    just ensure-command taplo

    just --unstable --fmt --check
    # We're using nightly to properly group imports, see rustfmt.toml
    cargo +nightly fmt -- --check

    taplo format --check

# Updates the local cargo index and displays which crates would be updated
dry-update:
    cargo update --dry-run --verbose

# Lints the source code
lint:
    just ensure-command shellcheck tangler

    tangler bash < alpm-buildinfo/README.md | shellcheck --shell bash -
    tangler bash < alpm-mtree/README.md | shellcheck --shell bash -
    tangler bash < alpm-pkginfo/README.md | shellcheck --shell bash -
    tangler bash < alpm-srcinfo/README.md | shellcheck --shell bash -

    shellcheck --shell bash alpm-srcinfo/tests/generate_srcinfo.bash

    just lint-recipe 'test-readme alpm-buildinfo'
    just lint-recipe 'test-readme alpm-pkginfo'
    just lint-recipe 'test-readme alpm-srcinfo'
    just lint-recipe build-book
    just lint-recipe check-commits
    just lint-recipe check-unused-deps
    just lint-recipe ci-publish
    just lint-recipe 'generate shell_completions alpm-buildinfo'
    just lint-recipe 'is-workspace-member alpm-buildinfo'
    just lint-recipe 'prepare-release alpm-buildinfo'
    just lint-recipe 'release alpm-buildinfo'
    just lint-recipe flaky
    just lint-recipe test
    just lint-recipe 'ensure-command test'

    cargo clippy --tests --all -- -D warnings

    just docs

# Check justfile recipe for shell issues
lint-recipe recipe:
    just -vv -n {{ recipe }} 2>&1 | rg -v '===> Running recipe' | shellcheck -

# Build local documentation
docs:
    RUSTDOCFLAGS='-D warnings' cargo doc --document-private-items --no-deps

# Checks for issues with dependencies
check-dependencies: dry-update
    cargo deny --all-features check

# Checks licensing status
check-licenses:
    just ensure-command reuse
    reuse lint

# Runs all unit tests. By default ignored tests are not run. Run with `ignored=true` to run only ignored tests
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
test-docs:
    just ensure-command cargo
    cargo test --doc

# Run end-to-end tests for README files of projects
test-readmes:
    just test-readme alpm-buildinfo
    just test-readme alpm-pkginfo
    just test-readme alpm-srcinfo

# Runs per project end-to-end tests found in a project README.md
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

# Adds needed git configuration for the local repository
configure-git:
    # Enforce gpg signed keys for this repository
    git config commit.gpgsign true

    just add-hooks

# Adds pre-commit and pre-push git hooks
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

# Check for stale links in documentation
check-links:
    just ensure-command lychee
    lychee .

# Fixes common issues. Files need to be git add'ed
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

# Render `manpages`, `shell_completions` or `specifications` (`kind`) of a given package (`pkg`).
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

# Generates all manpages and specifications
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
    just generate specifications alpm-types

# Generates shell completions
generate-completions:
    just generate shell_completions alpm-buildinfo
    just generate shell_completions alpm-mtree
    just generate shell_completions alpm-pkginfo
    just generate shell_completions alpm-srcinfo

# Continuously run integration tests for a given number of rounds
flaky test='just test-readme alpm-buildinfo' rounds='999999999999':
    #!/usr/bin/bash
    set -euo pipefail

    seq 1 {{ rounds }} | while read -r counter; do
      printf "Running flaky tests (%d/{{ rounds }})...\n" "$counter"
      sleep 1
      {{ test }}
      echo
    done

# Prepares the release of a crate by updating dependencies, incrementing the crate version and creating a changelog entry (optionally, the version can be set explicitly)
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

# Publishes a crate in the workspace from GitLab CI in a pipeline for tags
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

# Ensures that one or more required commands are installed
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

# Builds the documentation book using mdbook and stages all necessary rustdocs alongside
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

# Serves the documentation book using miniserve
serve-book: build-book
    just ensure-command miniserve
    miniserve --index=index.html {{ output_dir }}/docs

# Watches the documentation book contents and rebuilds on change using mdbook (useful for development)
watch-book:
    just ensure-command watchexec
    watchexec --exts md,toml,js --delay-run 5s -- just build-book
