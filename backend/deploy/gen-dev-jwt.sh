#!/usr/bin/env sh
set -e
cd "$(dirname "$0")"
if [ ! -f dev-jwt.pem ]; then
  openssl genrsa -out dev-jwt.pem 2048
  echo "Created deploy/dev-jwt.pem (gitignored). Use AUTH_JWT_PRIVATE_KEY_PATH pointing here."
else
  echo "deploy/dev-jwt.pem already exists; remove it to regenerate."
fi
