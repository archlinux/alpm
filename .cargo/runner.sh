#!/usr/bin/env bash
#
# Implements a custom target runner.
# This is used transparently by cargo-nextest:
# https://nexte.st/docs/features/target-runners/
# It is defined as a standard feature in cargo:
# https://doc.rust-lang.org/cargo/reference/config.html#targettriplerunner
#
# When this is used via cargo-nextest, this runner script is called in two different scenarios (with differing arguments):
#
# - Once to list all tests provided by a test executable (`<file> --list --format terse`),
# - another time to run each individual test that the executable provides (`<file> <test-name> --nocapture --exact`).
#
# For details see: https://nexte.st/docs/design/custom-test-harnesses/#manually-implementing-a-test-harness
set -euo pipefail

readonly test_executable_path="${1:-}"
readonly first_test_argument="${2:-}"
# Use Arch Linux's container registry when running in CI (indicated by the presence of the `CI` environment variable).
if [[ -z "${CI+x}" ]]; then
  arch_container="archlinux:latest"
else
  arch_container="registry.archlinux.org/archlinux/archlinux-docker:base-master"
fi

# If the test file name contains "containerized" and the invocation is not for listing the test, run the test in a container.
# Otherwise run on the host.
if [[ "$test_executable_path" == *containerized* ]] && [[ "$first_test_argument" != "--list" ]]; then
  target_dir="$(just get-cargo-target-directory)"
  readonly podman_run_options=(
    --env RUST_LOG=info
    --env RUST_BACKTRACE=1
    --rm
    # Mounts the current working directory into the container.
    --volume "$PWD:/test:ro,z"
    # Mounts the user's cargo target directory into the container.
    --volume "$target_dir/debug:/usr/local/bin:ro,z"
    # Mount the test executable into the container at the same location as on the host
    --volume "$test_executable_path:$test_executable_path:ro,z"
    "$arch_container"
  )

  podman run "${podman_run_options[@]}" "$@"
else
  "$@"
fi
