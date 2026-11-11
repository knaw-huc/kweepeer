FROM alpine:latest

RUN apk update && apk add cargo libgcc && \
    adduser -D user && mkdir -p /etc/service/kweepeer && \
    cargo install --root /usr --git https://github.com/knaw-huc/kweepeer kweepeer &&\
    apk del git cargo && rm -rf /root/.cache /root/.cargo /tmp/*

VOLUME /data
EXPOSE 80
USER user
WORKDIR /data

ENTRYPOINT [ "/usr/bin/kweepeer", "--bind", "0.0.0.0:8080", "--config" ]
