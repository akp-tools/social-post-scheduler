# build tailscale container
FROM mcr.microsoft.com/vscode/devcontainers/universal:linux as builder
USER root
WORKDIR /app
COPY . ./
ENV TSFILE=tailscale_1.26.1_amd64.tgz
RUN mkdir -p binaries && \
  wget https://pkgs.tailscale.com/stable/${TSFILE} && \
  tar xzf ${TSFILE} --strip-components=1 -C binaries
COPY . ./

FROM mcr.microsoft.com/vscode/devcontainers/rust:0-bullseye
USER root

RUN apt-get update && apt-get install -y curl gpg dnsutils
COPY tailscaled /etc/init.d/tailscaled
RUN chmod +x /etc/init.d/tailscaled
COPY --from=builder /app/binaries/tailscaled /usr/sbin/tailscaled
COPY --from=builder /app/binaries/tailscale /usr/bin/tailscale

RUN mkdir -p /var/run/tailscale /var/cache/tailscale /var/lib/tailscale
