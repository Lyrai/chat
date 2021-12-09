FROM rust:1.57
WORKDIR /usr/src/teamer
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir src
RUN touch ./src/main.rs
RUN echo "fn main() {}" > ./src/main.rs
RUN mkdir .cargo
RUN cargo vendor > .cargo/config
RUN cargo build --release
RUN rm ./src/main.rs
COPY ./src src
COPY ./Rocket.toml .
RUN cargo build --release
RUN cargo install --path . --verbose
CMD ["cargo", "run", "--release"]
