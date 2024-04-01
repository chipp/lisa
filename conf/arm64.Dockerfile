FROM alpine:3.19.1

RUN apk --no-cache add ca-certificates && update-ca-certificates
RUN apk --no-cache add tzdata && \
  cp /usr/share/zoneinfo/Europe/Vilnius /etc/localtime && \
  echo "Europe/Vilnius" > /etc/timezone && \
  apk del tzdata

WORKDIR /root/
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
ENV RUST_BACKTRACE=full

ARG BINARY

COPY ./build/${BINARY} .
CMD [ "${BINARY}" ]
