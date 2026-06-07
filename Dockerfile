# MemoryKnight — Linux test image.
#
# Gives a reproducible Linux environment to validate the behavior that can't be
# exercised on macOS: Layer 1 (the `setrlimit` wall) making `malloc()` return
# NULL, and RSS that reflects real usage (no macOS-style compression).
#
# Build:   docker build -t mknight .
# Demo:    docker run --rm --memory=1g mknight           # wall catches the leaker
# Shell:   docker run --rm -it --memory=1g mknight bash  # poke around by hand
#
# (--memory=1g is an extra container-level safety net; mknight itself is the guard.)

FROM rust:slim

# C compiler for building the example programs.
RUN apt-get update \
    && apt-get install -y --no-install-recommends gcc \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build the release binary and the example C programs.
RUN cargo build --release \
    && gcc examples/leak.c -o examples/leak \
    && gcc examples/list_leak.c -o examples/list_leak \
    && gcc -O2 examples/grow.c -o examples/grow

# Put mknight on PATH for convenient interactive use.
RUN install -m 0755 target/release/mknight /usr/local/bin/mknight

# Default: demonstrate Layer 2 killing a runaway leak. The unbounded list grows
# past the 200 MB ceiling and mknight's monitor terminates it with a post-mortem;
# the setrlimit wall (at 1.25x max-ram) stays as a higher backstop.
CMD ["mknight", "run", "--max-ram", "200MB", "examples/list_leak"]
