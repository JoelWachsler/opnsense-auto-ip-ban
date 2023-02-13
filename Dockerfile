FROM debian:stable-slim

# need this for cert updates
RUN apt-get update && apt-get install -y curl && mkdir -p /app
WORKDIR /app

COPY ./ip_ban ./
COPY ./config ./config

ENTRYPOINT ["./ip_ban"]
