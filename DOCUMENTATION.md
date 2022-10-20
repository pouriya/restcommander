# RestCommander
RestCommander is a simple REST-API layer on top of one or more scripts. From the request URL it detects which script it should run. It captures HTTP query-string parameters, headers, and body and after deserializing, merging, and validating inputs, passes them to the script. The script can read these inputs from [stdin](https://en.wikipedia.org/wiki/Standard_streams#Standard_input_(stdin)) or [environment variables](https://en.wikipedia.org/wiki/Environment_variable) to do different things. The script [stdout](https://en.wikipedia.org/wiki/Standard_streams#Standard_input_(stdin)) (whatever the script prints) is captured by RestCommander and that's the REST-API response body! Also, different script [exit-codes](https://en.wikipedia.org/wiki/Exit_status) causes different HTTP response status code.  
Additionally, RestCommander captures script [stderr](https://en.wikipedia.org/wiki/Standard_streams#Standard_error_(stderr)) for logging and some other operational things.  
A script can be stateless or stateful and RestCommander can capture current script state.  

```text
    ┌─────────────────────────────────────────────────────────┬──────────────────┐
    │                                                         │      Client      │
    │  REST-API Client or Web dashboard                       └──────────────────┤
    │                                                                            │
    └───1─────────────────────────────────▲──────────────────────────────────────┘
        │                                 9
        ▼                                 │
       HTTP                               │
       POST                           HTTP│Response
/api/run/foo/bar/baz                      │
        │                                 │
        │                                 │
        │                                 │
    ┌───▼─────────────────────────────────▲───────────────────┬──────────────────┐
    │ Authentication                      │                   │  RestCommander   │
    │   ▼                                 8                   └──────────────────┤
    │ Search for foo/bar/baz     Wrap to HTTP body                               │
    │   ▼                                 │            Capture log messages      │
    │ Validate input options              │      6────────────────────────────7──►
    │   │     │                           │      │                               │
    └───2─────3───────────────────────────▲──────▲───────────────────────────────┘
        │     │                           │      │
        ▼     │                           │      │
  Environment │                           │      │
   Variables  ▼                        STDOUT  STDERR
        │   STDIN                         ▲      ▲
        │   (JSON)                        │      │
        │     │                           │      │
    ┌───▼─────▼───────────────────────────5──────4────────────┬──────────────────┐
    │                                                         │      Script      │
    │ Reading options from environment variables              └──────────────────┤
    │ or STDIN and do something useful                                           │
    │                                                                            │
    └────────────────────────────────────────────────────────────────────────────┘
```
_Above ASCII diagram is generated via [asciiflow](https://asciiflow.com)_  


## Features
* REST-API:  
    * HTTP and HTTPS.  
    * IP wildcard access.  
    * CAPTCHA.  
    * Static and dynamic authentication tokens.  
* Web dashboard:  
    * Configurable Title, banner text, and footer.  
    * Extensible: You can serve your own front-end codes or replace RestCommander's.  
* Dynamic configuration reload. So you can change anything (even port number) without restarting service.  
* Single executable for macOS, Windows, and GNU/Linux.

## Installation
Run the following in your terminal to download the latest version:  
```shell
curl --proto '=https' --tlsv1.2 -sSfL https://github.com/pouriya/restcommander/releases/download/latest/install.sh | sh
```
or download latest version:
* GNU/Linux:
    * Musl (Statically linked):       [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-unknown-linux-musl-ubuntu-22.04)
    * GNU (Dynamic linking to glibc): [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-unknown-linux-gnu-ubuntu-22.04)
    * Debian package with `systemd` service (`.deb` file):  
      Configuration files are located in `/etc/restcommander` and script files will be loaded from `/srv/restcommander/scripts`.
        * Musl (Statically linked):       [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-unknown-linux-musl-ubuntu-22.04.deb)
        * GNU (Dynamic linking to glibc): [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-unknown-linux-gnu-ubuntu-22.04.deb)
* macOS:
    * v11: [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-apple-darwin-macos-11)
    * v12: [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-apple-darwin-macos-12)
* Windows:
    * MSVC: [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-pc-windows-msvc-windows-2022.exe)
    * GNU:  [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-pc-windows-gnu-windows-2022.exe)

### Docker
#### DockerHub
```shell
docker pull pouriya/restcommander
```
#### GitHub Container Registry
```shell
docker pull ghcr.io/pouriya/restcommander
```


# Configuration
For simplicity, Every configuration option has a default value. So you do not need to configure everything.  
RestCommander can be configured from commandline options via `playground` subcommand:
```shell
$ restcommander playground -h
USAGE:
    restcommander playground [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --commands-root-directory <commands-root-directory>
            Root directory to load command files and directories and their information files [env:
            RESTCOMMANDER_COMMANDS_ROOT_DIRECTORY=]  [default: /p/RestCommander]
        --logging-level-name <logging-level-name>
            Logging level name [env: RESTCOMMANDER_LOGGING_LEVEL_NAME=]  [default: info]

        --server-api-token <server-api-token>
            hardcoded HTTP bearer token that does not expire [env: RESTCOMMANDER_SERVER_API_TOKEN=]

        --server-captcha-case-sensitive <server-captcha-case-sensitive>
            Make CAPTCHA case-sensitive [env: RESTCOMMANDER_CAPTCHA_CASE_SENSITIVE=]

        --server-captcha-file <server-captcha-file>
            A file for saving captcha id/values [env: RESTCOMMANDER_SERVER_CAPTCHA_FILE=]

        --server-host <server-host>
            HTTP server listen address [env: RESTCOMMANDER_SERVER_HOST=]  [default: 127.0.0.1]

        --server-ip-whitelist <server-ip-whitelist>...
            List of IP addresses that can interact with REST-API. Wildcard characters like * are allowed [env:
            RESTCOMMANDER_CAPTCHA_CASE_SENSITIVE=]
        --server-password-file <server-password-file>
            A file containing sha512 of your user password [env: RESTCOMMANDER_SERVER_PASSWORD_FILE=]  [default: ]

        --server-password-sha512 <server-password-sha512>
            sha512 of you user password [env: RESTCOMMANDER_SERVER_PASSWORD_SHA512=]  [default: ]

        --server-port <server-port>
            HTTP server listen port number [env: RESTCOMMANDER_SERVER_PORT=]  [default: 1995]

        --server-tls-cert-file <server-tls-cert-file>
            HTTP server TLS certificate file [env: RESTCOMMANDER_SERVER_TLS_CERT_FILE=]

        --server-tls-key-file <server-tls-key-file>
            HTTP server TLS private-key file [env: RESTCOMMANDER_SERVER_TLS_KEY_FILE=]

        --server-token-timeout <server-token-timeout>
            Timeout for dynamically generated HTTP bearer tokens in seconds [env: RESTCOMMANDER_SERVER_TOKEN_TIMEOUT=]
            [default: 604800]
        --server-username <server-username>
            HTTP server basic authentication username [env: RESTCOMMANDER_SERVER_USERNAME=]  [default: ]

        --www-enabled <www-enabled>
            Enable/Disable the web dashboard [env: RESTCOMMANDER_WWW_ENABLED=]

        --www-static-directory <www-static-directory>
            A directory to serve your own web files under `/static/*` HTTP path [env:
            RESTCOMMANDER_WWW_STATIC_DIRECTORY=]  [default: ]
```  
Use `--help` instead of `-h` to get a more detailed help message.  
`playground` subcommand is not recommended for long-time running because you can't reload the configuration without restarting the entire service.  
You can get a complete TOML configuration settings with `restcommander sample config` command and use it as a configuration file:  
```shell
$ restcommander sample config > cfg.toml
$ vim cfg.toml # Edit configuration (if needed)
$ restcommander config cfg.toml
```
See the [TOML configuration sample](https://github.com/pouriya/restcommander/blob/master/samples/config.toml) for more info.  

# Contributing
[Backend Contributing](https://github.com/pouriya/restcommander/blob/master/CONTRIBUTING.md)  
[FrontEnd Contributing](https://github.com/pouriya/restcommander/blob/master/www/CONTRIBUTING.md)

# Specification
When you start RestCommander, It starts loading all executable files from configured `root_directory` and its sub-directories recursively. It does not know anything about them. It does not know what options they need to run. It does not know they are stateful or stateless. It would be great if it could run them with some specific options to get their information, but it's very dangerous. Imagine if you wrongly configure `root_directory` to a directory that contains some executable that you don't want RestCommander run them.  
So for every script it tries to read a `<SCRIPT_NAME>.yaml` file and loads that information from that file. The file content is in form of:  
```yaml
description: "<DESCRIPTION>"
version: "<VERSION>"
state: <STATE>
options: <OPTIONS>
```
* **DESCRIPTION**: Script description. The default value is empty string.  
* **VERSION**: Script version. The default value is empty string.  
* **STATE**: If the script is stateful, this field tells RestCommander how to fetch script's current state. So this field is optional (for stateless scripts you don't need to define it).
    Its values are in form of:
    ```yaml
    options: <STATE OPTIONS>
    ```
    or
    ```yaml
    constant: "<CONSTANT>"
    ```
    * **STATE OPTIONS**: List of options that will be passed to the script.  
        Example:
        ```yaml
        state:
          options:
            - foo
            - bar
        ```
        For example if above YAML configuration is for script `some/path/script.sh`, RestCommander will run `some/path/script.sh foo bar` to get its current state.   
    * **CONSTANT**: A string. This only changes when you change the `.yml` file. (It mainly used for test purposes).  
        Example:
        ```yaml
        state:
          constant: "The state of script"
        ```
* **OPTIONS**:
TODO

# REST API
TODO

## `/api`
### `/api/public`
#### `/api/public/captcha`
Method: **GET**  
Success:
```json
{"id": "<CAPTCHA_UUID>", "image": "<BASE64_PNG_IMAGE>"}
```

Failures:  
* **406**: If the CAPTCHA feature is not configured or RestCommander does not have appropriate permissions to update configured CAPTCHA file.

#### `/api/public/configuration`
Method: **GET**  
Success:
It's a JSON object containing all `www.configuration` key/values from your [TOML configuration file](https://github.com/pouriya/restcommander/blob/master/samples/config.toml).  
```json
{"...":  "..."}
```

Failures: Not failures.

## `/api/auth`
### `/api/auth/test`
You can test if your RestCommander service supports authentication or not and if it supports, You can test your bearer token too.  
Method: **GET**  
Success: Nothing
Failures:  
* **401**: Refer to authentication failures.

## `/api/auth/token`
Fetching new bearer token via your configured username/password inside your [TOML configuration file](https://github.com/pouriya/restcommander/blob/master/samples/config.toml).  
Method: **POST**  
If CAPTCHA is enabled, you should first get one and here you need to set `Content-Type` header to `application/x-www-form-urlencoded` and put your `<CAPTCHA_ID>=<CAPTCHA_TEXT>` inside request body.  
Success:
```json
{"token": "<YOUR_BEARER_TOKEN>"}
```

Failures:  
* **401**: Refer to authentication failures.  

## `/api/commands`
Fetching commands tree.  
Method: **GET**  
Success:
```json
{
  "name": "<NAME>",
  "http_path": "<HTTP_PATH>",
  "is_directory": <IS_DIRECTORY>,
  "info": <INFO>,
  "commands": <COMMANDS>
}
```
* `<NAME>`: script of folder name.  
* `<HTTP_PATH>`:  HTTP path to run this command.  
* `<IS_DIRECTORY>`: A boolean value. `true` if this object is for a directory.  
* `<INFO>`: Only present id `is_directory` is `true` and it is another object in form of:  
    ```json
    {
      "description": "<DESCRIPTION>",
      "version": "<VERSION>",
      "support_state": <SUPPORT_STATE>,
      "options": <OPTIONS>
    }
    ```
    * `<DESCRIPTION>`: Command description.  
    * `<VERSION>`: Command version. (optional)  
    * `<SUPPORT_STATE>`: `true` if this command support `/api/state` endpoint.  
    * `<OPTIONS>`: Object of accepted options in form of:  
        ```json
        {
           "description": "<DESCRIPTION>",
           "required": <REQUIRED>,
           "value_type": <VALUE_TYPE>,
           "default_value": <DEFAULT_VALUE>,
           "size": <SIZE>,
        }
        ```
        * `<DESCRIPTION>`:  Option description.  
        * `<REQUIRED>`: `true` if the option is required.  
        * `<VALUE_TYPE>`:  one of `"string"` | `"integer"` | `"float"` | `"bool"` | `{"accepted_value_list": ["...", "..."]}`.  
        * `<DEFAULT_VALUE>`:  The default value of option. (optional if the option itself is not required).  
        * `<SIZE>`: Another object in form of `{"min": NUMBER, "max": NUMBER}`. The whole object and its keys are optional.  
* `<COMMANDS>`: Another object containing the same structure. Only present if `is_directory` is `true` and the directory contains other commands or directories.  

Failures:  
* **401**: Refer to authentication failures.  

## `/api/setPassword`
Method: **POST**  
Request header `Content-Type` should be set to `application/json` and a body in form of `{"password": "<NEW_PASSWORD>"}` is required.  
Success: Nothing.  
Failures:  
* **400**: If provided password is empty.  
* **401**: Refer to authentication failures.  
* **503**: If `password_file` is not configured and RestCommander is started just with `pasword_sha512` (a hardcoded password).  
* **500**: If RestCommander does not have appropriate permissions to update password file.  


## `/api/reload`
### `/api/reload/commands`
Method: **GET**
Success: Nothing (So you have to fetch new commands from `/api/commands` endpoint).  
Failures:  
* **401**: Refer to authentication failures.  
* **500**: If RestCommander could not reload scripts.  

### `/api/reload/config`
Method: **GET**
Success: Nothing.  
Failures:
* **401**: Refer to authentication failures.
* **500**: If RestCommander could not reload configuration.  

## `/api/run/<PATH_TO_COMMAND>`
For example if your script is in `foo/bar` sub-directory of your configured `root_directory` and its filename is `baz` (it's `foo/bar/baz`), Then you have to call `/api/run/foo/bar/baz` endpoint.  
Method: **POST**  
Request body for each command is different, and it depends on command's configured input options in its YAML file.  
The command's process exit-status causes different HTTP status-code:  
**0**     -> **200** (OK)  
**1**     -> **500** (INTERNAL_SERVER_ERROR)  
**2**     -> **400** (BAD_REQUEST)  
**3**     -> **403** (FORBIDDEN)  
**4**     -> **404** (NOT_FOUND)  
**5**     -> **503** (SERVICE_UNAVAILABLE)  
**6**     -> **406** (NOT_ACCEPTABLE)  
**7**     -> **501** (NOT_IMPLEMENTED)  
**8**     -> **409** (CONFLICT)  
**9**     -> **408** (REQUEST_TIMEOUT)  
OTHER     -> **500** (INTERNAL_SERVER_ERROR)  
Note that HTTP response body is also captured from command's `stdin`. RestCommander tries to make a JSON object from command's `stdin` and if it could not, Then the whole string is returned in `result` of response object.  
For example a script exited with exit-status `2` and printed `{"foo": "bar"}`, The HTTP response status-code will be `400` and the response body will be `{"ok": false, "result": {"foo": "bar"}}`.  


## `/api/state/<PATH_TO_COMMAND>`
Method: **GET**  
HTTP response body is also captured from command's `stdin` as its current state.
TODO
