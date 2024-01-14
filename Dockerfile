FROM messense/rust-musl-cross:x86_64-musl as chef
ENV SQLX_OFFLINE=true
RUN cargo install cargo-chef
WORKDIR /adamfortunecom

FROM chef AS planner
# Copy source code from previous stage
COPY . .
# Generate info for caching dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /adamfortunecom/recipe.json recipe.json
# Build & cache dependencies
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
# Copy source code from previous stage
COPY . .
# Build application
RUN cargo build --release --target x86_64-unknown-linux-musl


# Build the frontend
FROM node:alpine AS node_build
WORKDIR /adamfortunecom_frontend
RUN mkdir frontend
COPY ./frontend/package.json ./frontend/package.json
COPY ./frontend/package-lock.json ./frontend/package-lock.json
WORKDIR /adamfortunecom_frontend/frontend
RUN npm install
WORKDIR /adamfortunecom_frontend
COPY frontend frontend
WORKDIR /adamfortunecom_frontend/frontend
RUN npm run build
WORKDIR /adamfortunecom_frontend


# Create a new stage with a minimal image
FROM scratch
COPY --from=node_build /adamfortunecom_frontend/dist /dist
COPY --from=builder /adamfortunecom/target/x86_64-unknown-linux-musl/release/adamfortunecom /adamfortunecom/rustbin
ENTRYPOINT ["/adamfortunecom/rustbin"]
EXPOSE 8080