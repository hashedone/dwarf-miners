FROM rust:alpine

# Add clippy
RUN rustup component add clippy

# Setup circleci user
RUN adduser --disabled-password --shell /bin/bash circleci circleci

USER circleci

CMD ["/bin/sh"]
