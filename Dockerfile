FROM rust:1.57
WORKDIR /usr/src/teamer
COPY . .
RUN rustup update nightly
CMD ["cargo", "run"]
