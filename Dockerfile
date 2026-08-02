FROM python:3.14-slim-trixie

ARG RUST_VERSION=1.97.1

ENV DEBIAN_FRONTEND=noninteractive \
    PATH=/root/.cargo/bin:${PATH} \
    PYTHONUNBUFFERED=1 \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        git \
        locales \
        make \
        pkg-config \
    && sed -i 's/^# \(en_US.UTF-8 UTF-8\)/\1/' /etc/locale.gen \
    && sed -i 's/^# \(de_DE.UTF-8 UTF-8\)/\1/' /etc/locale.gen \
    && sed -i 's/^# \(cs_CZ.UTF-8 UTF-8\)/\1/' /etc/locale.gen \
    && locale-gen \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain ${RUST_VERSION} \
    && rustup component add rustfmt clippy \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
COPY . .

# Normalize the copied checkout before Git-blob integrity verification.
RUN git config core.autocrlf false && git reset --hard HEAD

RUN make setup \
    && make build

CMD ["make", "verify"]
