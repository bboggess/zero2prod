FROM rust:1.72.0 AS builder
WORKDIR /app
# Linker dependencies for our build process
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin zero2prod

# Runtime stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# Install HTTPS dependencies
RUN apt update -y \
	&& apt install -y --no-install-recommends openssl ca-certificates \
	&& apt autoremove -y \
	&& apt clean -y \
	&& rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY config config
ENV APP_ENVIRONMENT production
ENTRYPOINT [ "./zero2prod" ]
