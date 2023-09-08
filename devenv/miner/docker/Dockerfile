FROM alpine:latest

RUN apk add --no-cache curl

COPY entrypoint.sh /bin/entrypoint.sh

RUN chmod a+x /bin/entrypoint.sh

ENTRYPOINT ["/bin/entrypoint.sh"]
