FROM alpine:edge

RUN apk add --no-cache cgit git fcgiwrap nginx spawn-fcgi wget

COPY cgitrc /etc/cgitrc
COPY nginx.conf /etc/nginx/nginx.conf
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 80
ENTRYPOINT ["/entrypoint.sh"]
