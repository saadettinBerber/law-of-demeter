FROM rust:1.99.1-bookworm@sha256:e6f2cff35d0f309acf8433b4f05878ed77092602c5065cc81801177da0bb3a9c AS build

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY skill/law-of-demeter/SKILL.md skill/law-of-demeter/SKILL.md
RUN cargo build --locked --release -p law-of-demeter-cli \
    && strip target/release/law-of-demeter

FROM maven:3.9.11-eclipse-temurin-21-noble@sha256:6fdc855a6ed81d288ca7ca37ac6ff5e9308b612485c0801d70b25a858c83d237

COPY --from=build /build/target/release/law-of-demeter /usr/local/bin/law-of-demeter
WORKDIR /workspace
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/law-of-demeter"]
CMD []
