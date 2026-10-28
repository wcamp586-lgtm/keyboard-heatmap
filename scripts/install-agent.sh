#!/bin/bash
# Installs the standalone macOS collection agent.
#
# The agent owns the keyboard event tap and is compiled once; the Tauri app only
# reads the aggregates. Keeping the agent outside the app bundle is what lets
# Input Monitoring survive rebuilds of the UI.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DATA="$HOME/Library/Application Support/com.local.keyboardheatmap"
LOG_DIR="$HOME/Library/Logs/com.local.keyboardheatmap"
LABEL="com.local.keyboardheatmap.agent"
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"

cd "$ROOT/src-tauri"
cargo build --release --bin keyboard-heatmap-agent

mkdir -p "$DATA/bin" "$LOG_DIR" "$HOME/Library/LaunchAgents"
cp target/release/keyboard-heatmap-agent "$DATA/bin/"
chmod +x "$DATA/bin/keyboard-heatmap-agent"

cat > "$PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>$LABEL</string>
  <key>ProgramArguments</key>
  <array><string>$DATA/bin/keyboard-heatmap-agent</string></array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>$LOG_DIR/agent.out.log</string>
  <key>StandardErrorPath</key><string>$LOG_DIR/agent.err.log</string>
</dict>
</plist>
EOF

launchctl bootout "gui/$(id -u)/$LABEL" 2>/dev/null || true
launchctl bootstrap "gui/$(id -u)" "$PLIST"

echo "agent installed."
echo "First run: allow 'keyboard-heatmap-agent' under System Settings → Privacy & Security → Input Monitoring."
