#! /usr/bin/env python3

# For logging
from sys import stderr

# Logging:
# Log messages are captured from stderr.
# Each Log message line should be in form of:
# <LOG_LEVEL><SPACE><LOG_TEXT>
# LOG_LEVEL: "TRACE", "DEBUG", "WARNING", "INFO", or "ERROR"
print("INFO This is a log message.", file=stderr)

# With exit-code 0 it gives HTTP 200 status code with body {"ok":true, "result":"Hello World!"}
print("Hello World!")
# If you change exit_code, it gives HTTP 4** ot 5** status code with body {"ok":false, "reason":"Hello World!"}
# Process exit-code to HTTP status-code mapping:
# 0     -> 200 (OK)
# 1     -> 500 (INTERNAL_SERVER_ERROR)
# 2     -> 400 (BAD_REQUEST)
# 3     -> 403 (FORBIDDEN)
# 4     -> 404 (NOT_FOUND)
# 5     -> 503 (SERVICE_UNAVAILABLE)
# 6     -> 406 (NOT_ACCEPTABLE)
# 7     -> 501 (NOT_IMPLEMENTED)
# 8     -> 409 (CONFLICT)
# 9     -> 408 (REQUEST_TIMEOUT)
# OTHER -> 500 (INTERNAL_SERVER_ERROR)
exit_code = 0
exit(exit_code)
