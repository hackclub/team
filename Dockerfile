FROM rust:latest as build

# Compile
RUN rm -rf /tmp/team/

COPY . /tmp/team/
RUN rm -rf /tmp/team/target/

WORKDIR /tmp/team/

RUN cargo build --release

# Copy the binary into a new container for a smaller docker image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl-dev ca-certificates

COPY --from=build /tmp/team/target/release/team /

RUN mkdir /tmp/team

USER root

ENV RUST_LOG=info
ENV RUST_BACKTRACE=full

CMD ["/team"]
