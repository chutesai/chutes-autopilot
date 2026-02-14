#!/bin/sh
# Entrypoint script for Caddy that sets CADDY_PROTOCOL based on CADDY_TLS.

CADDY_TLS=${CADDY_TLS:-true}

if [ "$CADDY_TLS" = "false" ]; then
    export CADDY_PROTOCOL="http://"
else
    export CADDY_PROTOCOL=""
fi

exec caddy run --config /etc/caddy/Caddyfile --adapter caddyfile

