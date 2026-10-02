FROM docker.io/library/ubuntu:latest

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y --no-install-recommends cgit git fcgiwrap nginx spawn-fcgi wget curl && \
    rm -rf /var/lib/apt/lists/*

COPY cgitrc /etc/cgitrc
COPY nginx.conf /etc/nginx/nginx.conf
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 80
ENTRYPOINT ["/entrypoint.sh"]
