FROM nomorechokedboy/web-project:seaorm-actix-root as root

FROM rust AS builder

ARG workdir=/app
ENV USER=non-root
ENV UID=1000

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}" \
    && update-ca-certificates \ 
    && USER=root cargo new ${workdir}
WORKDIR ${workdir}

COPY Cargo.toml .
COPY --from=root /app/Cargo.lock .
COPY packages/entity ./packages/entity 
COPY packages/migration ./packages/migration

RUN cargo b -r

COPY src ./src
COPY templates ./templates
RUN touch ${workdir}/src/main.rs \ 
    && cargo b -r

FROM busybox AS deps

ARG BUSYBOX_VERSION=1.31.0-i686-uclibc
ADD https://busybox.net/downloads/binaries/$BUSYBOX_VERSION/busybox_WGET /wget
RUN chmod a+x /wget

FROM gcr.io/distroless/cc AS runtime

COPY --from=deps /wget /usr/bin/wget
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /

# Copy our build
COPY --from=builder /app/target/release/core .

# Use an unprivileged user.
USER non-root:non-root

CMD ["./core"]