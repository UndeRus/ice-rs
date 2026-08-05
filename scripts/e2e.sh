#!/usr/bin/env bash
#
# Запускает E2E-тесты против настоящего Murmur, поднимая его сам.
#
# Зачем скрипт, а не один `cargo nextest`:
#
#   * Murmur не переживает `Meta::getServer` и остановку/запуск виртуального
#     сервера в пределах одного процесса — падает фатальной ошибкой потоков
#     Qt/SQL (`requested database does not belong to the calling thread`).
#     Поэтому набор разбит на группы, и между ними сервер перезапускается.
#   * Каждая группа получает **чистую** БД в temp-каталоге: Murmur сам создаёт
#     схему и виртуальный сервер 1 на пустой базе, так что засеивать нечего.
#     Это же значит, что тесты не пачкают чужие данные.
#
# Использование:
#   scripts/e2e.sh                # все группы
#   scripts/e2e.sh events auth    # только указанные
#
# Переменные:
#   MURMUR_BIN   путь к бинарю Murmur (иначе ищется автоматически)
#   ICE_PORT     порт Ice (по умолчанию 6602, чтобы не мешать своему серверу)
#   KEEP_TMP=1   не удалять temp-каталоги (для разбора падений)

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

ICE_PORT="${ICE_PORT:-6602}"
# Порт для Mumble-клиентов: сдвигаем, чтобы не конфликтовать с реальным сервером.
CLIENT_PORT=$((ICE_PORT + 1))

# ── поиск бинаря ───────────────────────────────────────────────────────────
find_murmur() {
    if [[ -n "${MURMUR_BIN:-}" ]]; then
        # Проверяем сразу: иначе ошибка всплывёт как «Murmur не запустился» уже
        # внутри группы, что куда менее понятно.
        if [[ ! -x "$MURMUR_BIN" ]]; then
            echo "MURMUR_BIN=$MURMUR_BIN не существует или не исполняем" >&2
            return 1
        fi
        echo "$MURMUR_BIN"
        return
    fi
    local candidate
    for candidate in \
        "$REPO_ROOT"/.mumble-server/server-* \
        "$REPO_ROOT"/.mumble-server/murmurd \
        "$REPO_ROOT"/.mumble-server/mumble-server
    do
        if [[ -x "$candidate" ]]; then
            echo "$candidate"
            return
        fi
    done
    for candidate in mumble-server murmurd; do
        if command -v "$candidate" >/dev/null 2>&1; then
            command -v "$candidate"
            return
        fi
    done
    return 1
}

MURMUR="$(find_murmur)" || {
    cat >&2 <<'EOF'
Не нашёл бинарь Murmur.

Укажите путь: MURMUR_BIN=/путь/к/mumble-server scripts/e2e.sh
Либо положите его в .mumble-server/ — скачать можно со
https://github.com/mumble-voip/mumble/releases (артефакт mumble-server).
EOF
    exit 2
}
echo "Murmur: $MURMUR"

# ── управление сервером ────────────────────────────────────────────────────
MURMUR_PID=""
TMPDIR_CURRENT=""

cleanup_server() {
    if [[ -n "$MURMUR_PID" ]] && kill -0 "$MURMUR_PID" 2>/dev/null; then
        kill "$MURMUR_PID" 2>/dev/null
        # Ждём до 5 с, потом уже жёстко.
        for _ in $(seq 1 50); do
            kill -0 "$MURMUR_PID" 2>/dev/null || break
            sleep 0.1
        done
        kill -9 "$MURMUR_PID" 2>/dev/null
    fi
    MURMUR_PID=""
    if [[ -n "$TMPDIR_CURRENT" && "${KEEP_TMP:-0}" != "1" ]]; then
        rm -rf "$TMPDIR_CURRENT"
    fi
    TMPDIR_CURRENT=""
}
trap cleanup_server EXIT INT TERM

start_server() {
    local label="$1"
    TMPDIR_CURRENT="$(mktemp -d "${TMPDIR:-/tmp}/mumble-ice-e2e-XXXXXX")"
    # Свежая БД: Murmur сам заведёт схему и виртуальный сервер 1.
    cat > "$TMPDIR_CURRENT/murmur.ini" <<EOF
database=$TMPDIR_CURRENT/murmur.sqlite
logfile=$TMPDIR_CURRENT/murmur.log
loglevel=3
port=$CLIENT_PORT
ice="tcp -h 127.0.0.1 -p $ICE_PORT"

[ice]
icesecretread=
icesecretwrite=
EOF
    "$MURMUR" -ini "$TMPDIR_CURRENT/murmur.ini" -fg \
        > "$TMPDIR_CURRENT/stdout.log" 2>&1 &
    MURMUR_PID=$!

    # Ждём, пока откроется Ice-порт.
    local i
    for i in $(seq 1 100); do
        if ! kill -0 "$MURMUR_PID" 2>/dev/null; then
            echo "  Murmur не запустился ($label):" >&2
            tail -20 "$TMPDIR_CURRENT/stdout.log" >&2
            return 1
        fi
        if nc -z 127.0.0.1 "$ICE_PORT" 2>/dev/null; then
            return 0
        fi
        sleep 0.2
    done
    echo "  Ice-порт $ICE_PORT не открылся за 20 с ($label)" >&2
    tail -20 "$TMPDIR_CURRENT/stdout.log" >&2
    return 1
}

# ── группы ─────────────────────────────────────────────────────────────────
# Каждая группа — своя строка: имя и фильтр для nextest.
run_group() {
    local name="$1"; shift
    echo
    echo "── $name ─────────────────────────────────────────────"
    if ! start_server "$name"; then
        return 1
    fi
    local rc=0
    MUMBLE_ICE_ENDPOINT="127.0.0.1:$ICE_PORT" \
        cargo nextest run -p mumble-ice --run-ignored all --no-fail-fast "$@" || rc=$?
    if [[ $rc -ne 0 && "${KEEP_TMP:-0}" == "1" ]]; then
        echo "  логи сервера: $TMPDIR_CURRENT/stdout.log" >&2
        TMPDIR_CURRENT=""   # не удалять
    fi
    cleanup_server
    return $rc
}

ALL_GROUPS=(client events reattach auth)
declare -a REQUESTED=("$@")

# Опечатка в имени группы иначе привела бы к «зелёному» прогону, в котором не
# выполнилось ничего.
#
# Проверка длины обязательна: в bash 3.2 (системный на macOS) раскрытие пустого
# массива под `set -u` — это unbound variable.
if [[ ${#REQUESTED[@]} -gt 0 ]]; then
    for req in "${REQUESTED[@]}"; do
        ok=0
        for known in "${ALL_GROUPS[@]}"; do
            [[ "$req" == "$known" ]] && ok=1 && break
        done
        if [[ $ok -eq 0 ]]; then
            echo "Неизвестная группа: $req" >&2
            echo "Доступны: ${ALL_GROUPS[*]}" >&2
            exit 2
        fi
    done
fi

want() {
    [[ ${#REQUESTED[@]} -eq 0 ]] && return 0
    local g
    for g in "${REQUESTED[@]}"; do
        [[ "$g" == "$1" ]] && return 0
    done
    return 1
}

FAILED=()

# Исходящая сторона.
if want client; then
    run_group "client (исходящие вызовы)" --test e2e || FAILED+=("client")
fi

# Колбеки, кроме деструктивного: он останавливает виртуальный сервер.
if want events; then
    run_group "events (колбеки)" --test e2e_events -E 'not test(reattaches)' \
        || FAILED+=("events")
fi

# Переподписка: отдельная группа, потому что stop/start виртуального сервера
# может увести Murmur в фатальную ошибку.
if want reattach; then
    run_group "reattach (перезапуск виртуального сервера)" \
        --test e2e_events -E 'test(reattaches)' || FAILED+=("reattach")
fi

# Аутентификатор: тесты внутри группы сериализованы через .config/nextest.toml —
# аутентификатор у сервера может быть только один.
if want auth; then
    run_group "auth (аутентификатор)" --test e2e_auth || FAILED+=("auth")
fi

echo
if [[ ${#FAILED[@]} -eq 0 ]]; then
    echo "✓ все группы прошли"
    exit 0
fi
echo "✗ упали группы: ${FAILED[*]}" >&2
echo "  подробности: KEEP_TMP=1 scripts/e2e.sh ${FAILED[*]}" >&2
exit 1
