FROM docker.io/library/ubuntu:latest

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y --no-install-recommends cgit git fcgiwrap nginx spawn-fcgi wget curl python3 && \
    rm -rf /var/lib/apt/lists/*

COPY cgitrc /etc/cgitrc
COPY nginx.conf /etc/nginx/nginx.conf
COPY entrypoint.sh /entrypoint.sh
COPY render-all-cgit.py /render-all-cgit.py
RUN chmod +x /entrypoint.sh /render-all-cgit.py

EXPOSE 80
ENTRYPOINT ["/entrypoint.sh"]
