#!/usr/bin/env -S just --working-directory . --justfile
# Load project-specific properties from the `.env` file

set dotenv-load := true

# Whether to run ignored tests (set to "true" to run ignored tests)

ignored := "false"

# Runs all checks and tests. Since this is the first recipe it is run by default.
run-pre-commit-hook: check test

# Runs all check targets
check: check-spelling check-formatting lint check-unused-deps check-dependencies check-licenses check-links

# Faster checks need to be executed first for better UX.  For example
# codespell is very fast. cargo fmt does not need to download crates etc.

# Installs all tools required for development
dev-install: install-pacman-dev-packages install-rust-dev-tools

# Installs development packages using pacman
install-pacman-dev-packages:
    # All packages are set in the `.env` file
    pacman -Syu --needed --noconfirm $PACMAN_DEV_PACKAGES

# Installs all Rust tools required for development
install-rust-dev-tools:
    rustup default stable
    rustup component add clippy
    rustup toolchain install nightly
    rustup component add --toolchain nightly rustfmt

# Checks commit messages for correctness
check-commits:
    #!/usr/bin/env bash
    set -euo pipefail

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
        else
            printf "Commit %s ✅️\n\n" "$commit"
        fi
    done

# Runs checks before pushing commits to remote repository.
run-pre-push-hook: check-commits

# Checks common spelling mistakes
check-spelling:
    codespell

# Gets names of all workspace members
get-workspace-members:
    cargo metadata --format-version=1 |jq -r '.workspace_members[] | capture("/(?<name>[a-z-]+)#.*").name'

# Checks if a string matches a workspace member exactly
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

    for name in $(just get-workspace-members); do
        cargo machete "$name"
    done

# Checks source code formatting
check-formatting:
    just --unstable --fmt --check
    # We're using nightly to properly group imports, see rustfmt.toml
    cargo +nightly fmt -- --check

# Updates the local cargo index and displays which crates would be updated
dry-update:
    cargo update --dry-run --verbose

# Lints the source code
lint:
    tangler bash < alpm-buildinfo/README.md | shellcheck --shell bash -

    just lint-recipe 'test-readme alpm-buildinfo'
    just lint-recipe check-commits
    just lint-recipe check-unused-deps
    just lint-recipe ci-publish
    just lint-recipe 'generate shell_completions alpm-buildinfo'
    just lint-recipe 'is-workspace-member alpm-buildinfo'
    just lint-recipe 'release alpm-buildinfo'
    just lint-recipe flaky

    cargo clippy --all -- -D warnings

# Check justfile recipe for shell issues
lint-recipe recipe:
    just -vv -n {{ recipe }} 2>&1 | rg -v '===> Running recipe' | shellcheck -

# Checks for issues with dependencies
check-dependencies: dry-update
    cargo deny --all-features check

# Checks licensing status
check-licenses:
    reuse lint

# Runs all unit tests. By default ignored tests are not run. Run with `ignored=true` to run only ignored tests
test:
    {{ if ignored == "true" { "cargo test --all -- --ignored" } else { "cargo test --all && RUSTFLAGS='-D warnings' cargo doc --no-deps" } }}

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

# Adds pre-commit and pre-push git hooks
add-hooks:
    #!/usr/bin/env bash
    set -euo pipefail

    echo just run-pre-commit-hook > .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit

    echo just run-pre-push-hook > .git/hooks/pre-push
    chmod +x .git/hooks/pre-push

# Check for stale links in documentation
check-links:
    lychee -- '**/*.md' '**/*.rs'

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

render-script := '''
    //! ```cargo
    //! [dependencies]
    //! pkg = { path = "PATH", package = "PKG" }
    //! clap_allgen = "0.2.1"
    //! ```

    fn main() -> Result<(), Box<dyn std::error::Error>> {
        clap_allgen::render_KIND::<pkg::cli::Cli>(
            &std::env::args().collect::<Vec<_>>()[1],
        )?;
        Ok(())
    }
'''

# Render `manpages` or `shell_completions` (`kind`) of a given package (`pkg`).
generate kind pkg:
    #!/usr/bin/bash

    set -Eeuo pipefail

    readonly output_dir="${CARGO_TARGET_DIR:-$PWD/output}"
    mkdir --parents "$output_dir"

    readonly kind="{{ kind }}"

    case "$kind" in
      manpages|shell_completions)
          ;;
      *)
          printf 'Only "manpages" and "shell_completions" are supported.\n'
          exit 1
    esac

    script="$(mktemp --suffix=.rs)"
    sed "s/PKG/{{ pkg }}/;s#PATH#$PWD/{{ pkg }}#g;s/KIND/{{ kind }}/g" > "$script" <<< '{{ render-script }}'
    rust-script "$script" "$output_dir/{{ kind }}"
    rm --force "$script"

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

# Prepares the release of a crate by updating dependencies, incrementing the crate version and creating a changelog entry
prepare-release package:
    release-plz update -u -p {{ package }}

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