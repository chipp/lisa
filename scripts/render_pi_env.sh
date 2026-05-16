#!/usr/bin/env bash
set -euo pipefail

output_dir="${1:-build/pi-env}"
op_account="UKDQEQIPJVEARKGQPBMCLFP5MQ"

if ! command -v op >/dev/null 2>&1; then
  echo "1Password CLI is required: op" >&2
  exit 69
fi

read_secret() {
  local ref="$1"

  op read --account "$op_account" "$ref"
}

install -d -m 700 "$output_dir"

cat > "$output_dir/elisa.env" <<EOF
RUST_LOG=elisa=debug,roborock=debug,info
MQTT_ADDRESS=mqtts://lisa.chipp.dev:8880
MQTT_USER=elisa
MQTT_PASS=$(read_secret "op://Private/elisa mqtt/password")
ROBOROCK_IP=10.0.1.150
ROBOROCK_DUID=$(read_secret "op://Private/Vacuum Roborock/username")
ROBOROCK_LOCAL_KEY=$(read_secret "op://Private/Vacuum Roborock/credential")
EOF

cat > "$output_dir/elisheba.env" <<EOF
RUST_LOG=elisheba=debug,sonoff=debug,info
MQTT_ADDRESS=mqtts://lisa.chipp.dev:8880
MQTT_USER=elisheba
MQTT_PASS=$(read_secret "op://Private/elisheba mqtt/password")
KEYS=$(read_secret "op://Private/elisheba devices/notesPlain")
EOF

cat > "$output_dir/isabel.env" <<EOF
RUST_LOG=isabel=debug,info
MQTT_ADDRESS=mqtts://lisa.chipp.dev:8880
MQTT_USER=isabel
MQTT_PASS=$(read_secret "op://Private/isabel mqtt/password")
DB_PATH=/var/lib/lisa/isabel/isabel.db
EOF

chmod 600 "$output_dir"/*.env

printf 'Rendered env files into %s\n' "$output_dir"
