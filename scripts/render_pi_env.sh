#!/usr/bin/env bash
set -euo pipefail

output_dir="${1:-build/pi-env}"
op_account="tapitapka"

if ! command -v op >/dev/null 2>&1; then
  echo "1Password CLI is required: op" >&2
  exit 69
fi

install -d -m 700 "$output_dir"

cat <<'EOF' | op inject --account "$op_account" --out-file "$output_dir/elisa.env"
RUST_LOG=elisa=debug,roborock=debug,info
MQTT_ADDRESS=mqtts://mq.chipp.dev:8880
MQTT_USER=elisa
MQTT_PASS={{ op://Private/elisa mqtt/password }}
ROBOROCK_IP=10.0.1.150
ROBOROCK_DUID={{ op://Private/Vacuum Roborock/username }}
ROBOROCK_LOCAL_KEY={{ op://Private/Vacuum Roborock/credential }}
EOF

cat <<'EOF' | op inject --account "$op_account" --out-file "$output_dir/elisheba.env"
RUST_LOG=elisheba=debug,sonoff=debug,info
MQTT_ADDRESS=mqtts://mq.chipp.dev:8880
MQTT_USER=elisheba
MQTT_PASS={{ op://Private/elisheba mqtt/password }}
KEYS={{ op://Private/elisheba devices/notesPlain }}
EOF

cat <<'EOF' | op inject --account "$op_account" --out-file "$output_dir/isabel.env"
RUST_LOG=isabel=debug,info
MQTT_ADDRESS=mqtts://mq.chipp.dev:8880
MQTT_USER=isabel
MQTT_PASS={{ op://Private/isabel mqtt/password }}
DB_PATH=/var/lib/lisa/isabel/isabel.db
EOF

chmod 600 "$output_dir"/*.env

printf 'Rendered env files into %s\n' "$output_dir"
