# You can define AECO_PATH if you want AECO to be installed somewhere else.
# It needs to go somewhere that is writeable by the user so the launcher can
# update itself as well as the game.

if [[ -z "${AECO_PATH}" ]]; then
    TARGET_DIR="${HOME}/Atomix-ECO"
else
    TARGET_DIR="${AECO_PATH}"
fi

SAGA_DIR="${TARGET_DIR}/SAGA10"

if [ ! -d "${SAGA_DIR}" ]; then
    mkdir -p "${SAGA_DIR}"
fi

LAUNCHER_PATH="${SAGA_DIR}/aeco-launcher"

if [ ! -f "${LAUNCHER_PATH}" ]; then
    # Compressed b64 created by e.g. `cat <file> | gzip | base64 -w0`
    LAUNCHER_DATA=STAMP_LAUNCHER_HERE
    echo "${LAUNCHER_DATA}" | base64 -d | gunzip > "${LAUNCHER_PATH}"
fi

if [ ! -x "${LAUNCHER_PATH}" ]; then
    chmod u+x "${LAUNCHER_PATH}"
fi

"${LAUNCHER_PATH}"