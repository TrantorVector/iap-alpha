#!/bin/bash
# Script to generate development RSA keys for JWT RS256 signing

set -e

SECRETS_DIR="./secrets"
PRIVATE_KEY="$SECRETS_DIR/private_key.pem"
PUBLIC_KEY="$SECRETS_DIR/public_key.pem"

mkdir -p "$SECRETS_DIR"

if [ -f "$PRIVATE_KEY" ]; then
    echo "Existing private key found at $PRIVATE_KEY. Skipping generation."
else
    echo "Generating RSA private key..."
    openssl genrsa -out "$PRIVATE_KEY" 2048
    chmod 600 "$PRIVATE_KEY"
fi

if [ -f "$PUBLIC_KEY" ]; then
    echo "Existing public key found at $PUBLIC_KEY. Skipping extraction."
else
    echo "Extracting public key..."
    openssl rsa -in "$PRIVATE_KEY" -pubout -out "$PUBLIC_KEY"
fi

echo ""
echo "Keys generated successfully in $SECRETS_DIR"
echo ""
echo "To use these in your .env file, you can copy the paths:"
echo "JWT_PRIVATE_KEY_FILE=$PRIVATE_KEY"
echo "JWT_PUBLIC_KEY_FILE=$PUBLIC_KEY"
echo ""
echo "Or if you need them inline (escaped newlines):"
echo "JWT_PRIVATE_KEY=\"$(awk '{printf "%s\\n", $0}' "$PRIVATE_KEY")\""
echo "JWT_PUBLIC_KEY=\"$(awk '{printf "%s\\n", $0}' "$PUBLIC_KEY")\""
