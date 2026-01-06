#!/usr/bin/env sh
# ----------------------------------------------------------------------------------------------------------------------
# Initialise:
if [ "${DEBUG}" != "" ]
then
  echo "Debug mode is on"
  set -xe
else
  set -e
fi
# ----------------------------------------------------------------------------------------------------------------------
# Functions:

_print_usage(){
  echo "Usage:"
  echo "    <SCRIPT> <MCPD_EXECUTABLE> <ROOT_DIRECTORY> <TEMPLATE_ROOT_DIRECTORY>"
}

_replace_toml_value_in_file(){
  sed -i -E "s|$1 = (.*)|$2 = $3|g" $4
}
# ----------------------------------------------------------------------------------------------------------------------
# Checks:
if [ ! "$1" ]
then
  _print_usage
  exit 1
fi
if [ ! "$2" ]
then
  _print_usage
  exit 1
fi
if [ ! -f "$1" ]
then
  echo "Executable '$1' could not be found!"
  _print_usage
  exit 1
fi
# ----------------------------------------------------------------------------------------------------------------------
EXE=$1
ROOT_DIR=$2
TEMPLATE_ROOT_DIR=$3
mkdir -p "${ROOT_DIR}"
echo "Attempt to install ${EXE} inside ${ROOT_DIR}"
# ----------------------------------------------------------------------------------------------------------------------
echo ""
echo "Setup ${ROOT_DIR}/bin"
if [ -f "${ROOT_DIR}/bin/mcpd" ]
then
  echo "${ROOT_DIR}/bin/mcpd already exists!"
  echo "Make a backup and remove it and try again"
  exit 1
fi
mkdir -p "${ROOT_DIR}/bin"
cp "${EXE}" "${ROOT_DIR}/bin/mcpd"
EXE_NAME="$(basename ${EXE})"
EXE="${ROOT_DIR}/bin/mcpd"
TEMPLATE_EXE="${TEMPLATE_ROOT_DIR}/bin/mcpd"
chmod a+x "${EXE}"
echo "Check if executable works..."
if [ ! "$(${EXE} --version)" ]
then
  echo "Could not run \"${EXE} --version\". Executable does not work properly."
  exit 1
else
  echo "Executable ${EXE} works well"
fi
echo "Copied ${EXE_NAME} to ${EXE}"
# ----------------------------------------------------------------------------------------------------------------------
echo ""
CERT_DIR="${CONFIG_DIR}/certs"
TEMPLATE_CERT_DIR="${TEMPLATE_CONFIG_DIR}/certs"
mkdir -p "${CONFIG_DIR}/certs"
CERT_FILE="${CERT_DIR}/cert.pem"
TEMPLATE_CERT_FILE="${TEMPLATE_CERT_DIR}/cert.pem"
if [ ! -f "${CERT_FILE}" ]
then
  ${EXE} sample self-signed-cert > "${CERT_FILE}"
  echo "Created new certificate file ${CERT_FILE}"
else
  echo "Certificate file ${CERT_FILE} already exists"
fi
KEY_FILE="${CERT_DIR}/key.pem"
TEMPLATE_KEY_FILE="${TEMPLATE_CERT_DIR}/key.pem"
if [ ! -f "${KEY_FILE}" ]
then
  ${EXE} sample self-signed-key > "${KEY_FILE}"
  echo "Created new private-key file ${KEY_FILE}"
else
  echo "private-key file ${KEY_FILE} already exists"
fi
# ----------------------------------------------------------------------------------------------------------------------
echo ""
echo "Setup ${ROOT_DIR}/srv"
WWW_DIR="${ROOT_DIR}/srv/mcpd/www"
TEMPLATE_WWW_DIR="${TEMPLATE_ROOT_DIR}/srv/mcpd/www"
mkdir -p "${WWW_DIR}"
_replace_toml_value_in_file "static_directory" "static_directory" "\"${TEMPLATE_WWW_DIR}\"" "${EXAMPLE_CONFIG_FILE}"
SCRIPTS_DIR="${ROOT_DIR}/srv/mcpd/scripts"
TEMPLATE_SCRIPTS_DIR="${TEMPLATE_ROOT_DIR}/srv/mcpd/scripts"
mkdir -p "${SCRIPTS_DIR}"
if [ ! "$(ls -A ${SCRIPTS_DIR})" ]
then
  ${EXE} sample script > "${SCRIPTS_DIR}/sample"
  chmod a+x "${SCRIPTS_DIR}/sample"
  ${EXE} sample script-info > "${SCRIPTS_DIR}/sample.yml"
  echo "Created new sample script inside ${SCRIPTS_DIR}"
else
  echo "Scripts directory ${SCRIPTS_DIR} already exists and contains some files"
fi
_replace_toml_value_in_file "root_directory" "root_directory" "\"${TEMPLATE_SCRIPTS_DIR}\"" "${EXAMPLE_CONFIG_FILE}"
# ----------------------------------------------------------------------------------------------------------------------
echo ""
echo "Setup ${ROOT_DIR}/var"
LOG_DIR="${ROOT_DIR}/var/log/mcpd"
TEMPLATE_LOG_DIR="${TEMPLATE_ROOT_DIR}/var/log/mcpd"
mkdir -p "${LOG_DIR}"
if [ ! "$(ls -A ${LOG_DIR})" ]
then
  echo "Created new logging directory ${LOG_DIR}"
else
  echo "Logging directory ${LOG_DIR} already exists and contains some files"
fi
_replace_toml_value_in_file "output" "output" "\"stderr\" # or \"${TEMPLATE_LOG_DIR}\"" "${EXAMPLE_CONFIG_FILE}"
_replace_toml_value_in_file "report" "report" "\"stdout\" # or \"${TEMPLATE_LOG_DIR}/report.log\"" "${EXAMPLE_CONFIG_FILE}"
DATA_DIR="${ROOT_DIR}/var/lib/mcpd"
TEMPLATE_DATA_DIR="${TEMPLATE_ROOT_DIR}/var/lib/mcpd"
mkdir -p "${DATA_DIR}"
PASSWORD_FILE="${DATA_DIR}/password"
TEMPLATE_PASSWORD_FILE="${TEMPLATE_DATA_DIR}/password"
NEW_PASSWORD="$2a$12$uQmZCGsTkB5zjpEUmFfE7eOX4qxMjtcHwM72wbYbK3WInKYW/2eR2"
if [ ! -f "${PASSWORD_FILE}" ]
then
  echo ${NEW_PASSWORD} > "${PASSWORD_FILE}"
  echo "Created new password file ${PASSWORD_FILE} containing password 'admin'"
else
  echo "Password file ${PASSWORD_FILE} already exists"
fi
_replace_toml_value_in_file "#password_file" "password_file" "\"${TEMPLATE_PASSWORD_FILE}\"" "${EXAMPLE_CONFIG_FILE}"
_replace_toml_value_in_file "password_sha512" "#password_sha512" "\"${NEW_PASSWORD}\" # 'admin'" "${EXAMPLE_CONFIG_FILE}"
# ----------------------------------------------------------------------------------------------------------------------
echo ""
if [ ! -f "${CONFIG_FILE}" ]
then
  cp ${EXAMPLE_CONFIG_FILE} "${CONFIG_FILE}"
  echo "Created new configuration file ${CONFIG_FILE}"
else
  echo "Configuration file ${CONFIG_FILE} already exists"
fi
# ----------------------------------------------------------------------------------------------------------------------
echo ""
echo "Installed successfully"
