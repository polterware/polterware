import { SECTION_MARKER } from "../shared/parsers/sections.js";

export function buildRemoteMetricsScript(): string {
  const m = SECTION_MARKER;
  return `#!/bin/sh
printf '${m} stat\\n'
cat /proc/stat 2>/dev/null || true
printf '${m} meminfo\\n'
cat /proc/meminfo 2>/dev/null || true
printf '${m} netdev\\n'
cat /proc/net/dev 2>/dev/null || true
printf '${m} loadavg\\n'
cat /proc/loadavg 2>/dev/null || true
printf '${m} uptime\\n'
cat /proc/uptime 2>/dev/null || true
printf '${m} df\\n'
df -P -k 2>/dev/null || true
printf '${m} ps\\n'
ps -eo pid,user,pcpu,pmem,comm --sort=-pcpu 2>/dev/null | head -25 || \\
  ps -eo pid,user,pcpu,pmem,comm 2>/dev/null | head -25 || true
printf '${m} thermal\\n'
for z in /sys/class/thermal/thermal_zone*/temp; do
  [ -f "$z" ] || continue
  d=$(dirname "$z")
  id=$(basename "$d")
  t=$(cat "$z" 2>/dev/null) || continue
  printf '%s:%s\\n' "$id" "$t"
done
printf '${m} nvidia\\n'
if command -v nvidia-smi >/dev/null 2>&1; then
  nvidia-smi --query-gpu=index,name,temperature.gpu,utilization.gpu,memory.used,memory.total --format=csv,noheader,nounits 2>/dev/null || true
fi
`;
}
