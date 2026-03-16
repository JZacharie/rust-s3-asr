# --- Phase de production (CI) ---
FROM debian:trixie-slim AS release-ci
ARG PREBUILT_BINARY
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY ${PREBUILT_BINARY} /app/rust-s3-asr
RUN chmod +x /app/rust-s3-asr
CMD ["/app/rust-s3-asr"]
