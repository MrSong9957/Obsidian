#!/bin/bash
./h & p=$!
sleep 0.2
grep -E 'VmHWM|VmRSS' /proc/$p/status
wait $p
