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
FROM debian:buster-slim

# Set the current working directory
WORKDIR /usr/src/myapp

# Copy the binary from the build stage to the runtime stage
COPY --from=build /usr/src/myapp/target/release/myapp .

# Expose the port your app runs on
# EXPOSE 8080

# Run the binary
ENTRYPOINT ["./myapp"]
CMD ["arg1", "arg2"]
