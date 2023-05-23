# Step 1: Build stage
# Use the official Rust image as a base
FROM rust:latest AS build

# Set the current working directory inside the Docker image
WORKDIR /usr/src/myapp

# Copy the source code into the Docker image
COPY . .

# Build the application
RUN cargo build --release

# Step 2: Runtime stage
# Start from a new image to create a smaller final image
FROM registry.devops.rivtower.com/cita-cloud/storage_rocksdb:latest as storage_rocksdb
FROM registry.devops.rivtower.com/cita-cloud/storage_opendal:latest as storage_opendal
FROM debian:buster-slim

# Set the current working directory
WORKDIR /usr/src/myapp

# Copy the binary from the build stage to the runtime stage
COPY --from=build /usr/src/myapp/target/release/converter .
COPY --from=build /usr/src/myapp/config_opendal.toml .
COPY --from=build /usr/src/myapp/config_rocksdb.toml .
COPY --from=storage_rocksdb /usr/bin/storage ./storage_rocksdb
COPY --from=storage_opendal /usr/bin/storage ./storage_opendal
