FROM docker.io/archlinux:latest

RUN pacman -Syu --needed --noconfirm just

WORKDIR /alpm
COPY justfile .
RUN just dev-install && yes | pacman -Scc

# Use a nested target dir so that the container and
# the local development setup don't clash.
ENV CARGO_TARGET_DIR=/alpm/target/dev-container
# Allows our `just` recipes to detect that they already run inside the dev container.
ENV ALPM_DEV_CONTAINER=1

CMD ["bash"]
