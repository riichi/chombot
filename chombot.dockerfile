FROM rust:1.72.1 as builder

RUN USER=root cargo new --bin chombot

WORKDIR ./chombot

ADD . ./

RUN cargo build --bin chombot --release

FROM debian:bookworm-slim
ARG APP=/app

RUN apt-get update \
    && apt-get install -y ca-certificates

ENV APP_USER=chombot

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /chombot/target/release/chombot ${APP}/chombot

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./chombot"]
