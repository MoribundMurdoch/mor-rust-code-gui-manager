#!/usr/bin/env bash
set -euo pipefail

cat \
  assets/css/base.css \
  assets/css/layout.css \
  assets/css/controls.css \
  assets/css/graph.css \
  assets/css/node_file_manager.css \
  assets/css/automation.css \
  assets/css/editor.css \
  assets/css/runelite_sidebar.css \
  assets/css/plugins.css \
  > assets/purple_ink.css

echo "Built assets/purple_ink.css"