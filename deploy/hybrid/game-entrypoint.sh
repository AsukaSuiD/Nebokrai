#!/bin/sh
set -eu

wait_for_peer() {
    host="$1"
    port="$2"
    echo "GameServer: ожидание ${host}:${port}"
    until nc -z "$host" "$port"; do
        sleep 1
    done
}

wait_for_peer nebokrai_world 1747
wait_for_peer nebokrai_billing 8188

echo "GameServer: запуск gameserver.exe через Wine"
if [ ! -f "$WINEPREFIX/system.reg" ]; then
    export NEBOKRAI_INITIALIZE_WINE=1
else
    export NEBOKRAI_INITIALIZE_WINE=0
fi

export DISPLAY=:99
Xvfb "$DISPLAY" -screen 0 1280x1024x24 -nolisten tcp >/tmp/xvfb.log 2>&1 &
xvfb_pid=$!
trap 'kill "$xvfb_pid" 2>/dev/null || true' EXIT

while [ ! -S /tmp/.X11-unix/X99 ]; do
    if ! kill -0 "$xvfb_pid" 2>/dev/null; then
        cat /tmp/xvfb.log >&2
        exit 1
    fi
    sleep 1
done

if [ "$NEBOKRAI_INITIALIZE_WINE" = 1 ]; then
    echo "GameServer: инициализация 32-битного Wine prefix"
    wineboot --init
fi

wine /runtime/gameserver.exe
