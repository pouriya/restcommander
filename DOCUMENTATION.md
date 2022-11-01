# RestCommander
RestCommander is a simple REST-API layer on top of one or more scripts. From the request URL it detects which script it should run. It captures HTTP query-string parameters, headers, and body and after deserializing, merging, and validating inputs (options), passes them to the script. The script can read these options from [stdin](https://en.wikipedia.org/wiki/Standard_streams#Standard_input_(stdin)) or [environment variables](https://en.wikipedia.org/wiki/Environment_variable) to do different things. The script [stdout](https://en.wikipedia.org/wiki/Standard_streams#Standard_input_(stdin)) (whatever the script prints) is captured by RestCommander and that's the REST-API response body! Also, different script [exit-codes](https://en.wikipedia.org/wiki/Exit_status) cause different HTTP response status-codes.  
Additionally, RestCommander captures script's [stderr](https://en.wikipedia.org/wiki/Standard_streams#Standard_error_(stderr)) for logging and some other operational things.  
A script can be stateless or stateful and for stateful scripts RestCommander can capture current script state.  

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


## Table of contents
* [**Features**](#features)
* [**Installation**](#installation)
* [**Docker**](#docker)
    * [**DockerHub**](#dockerhub)
    * [**GitHub Container Registry**](#github-container-registry)
* [**Quick Start**](#quickstart)
* [**Configuration**](#configuration)
* [**Script Information format**](#script-information-format)
    * [**Examples**](#examples)
* [**REST API**](#rest-api)
    * [**Authentication**](#authentication)
    * [**Recommended authentication flow**](#recommended-authentication-flow)
    * [**/api**](#api)
    * [**/api/public**](#apipublic)
        * [**/api/public/captcha**](#apipubliccaptcha)
        * [**/api/public/configuration**](#apipublicconfiguration)
    * [**/api/auth**](#apiauth)
        * [**/api/auth/test**](#apiauthtest)
        * [**/api/auth/token**](#apiauthtoken)
    * [**/api/commands**](#apicommands)
    * [**/api/setPassword**](#apisetpassword)
    * [**/api/reload**](#apireload)
        * [**/api/reload/commands**](#apireloadcommands)
        * [**/api/reload/config**](#apireloadconfig)
    * [**/api/run/...**](#apirun)
    * [**/api/state/...**](#apistate)
* [**Contributing**](#contributing)

## Features
* REST-API:  
    * HTTP and HTTPS.  
    * IP wildcard access.  
    * CAPTCHA.  
    * Static and dynamic authentication tokens.  
* Web dashboard:  
    * Configurable Title, banner text, and footer.  
    * Extensible: You can serve your own frontend codes or replace RestCommander files.  
* Dynamic configuration reload. So you can change anything (even port number) without restarting service.  
* Single executable for macOS, Windows, and GNU/Linux.

## Installation
Run the following in your terminal to download the latest version:  
```shell
curl -sSfL https://github.com/pouriya/restcommander/releases/download/latest/install.sh | sh
```
or download latest version:
* GNU/Linux:
    * Musl (Statically linked):        [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-unknown-linux-musl-ubuntu-22.04)
    * GNU  (Dynamic linking to glibc): [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-unknown-linux-gnu-ubuntu-22.04)
    * Debian package with `systemd` service (`.deb` file):  
      Configuration files are located in `/etc/restcommander` and script files will be loaded from `/srv/restcommander/scripts`.
        * Musl (Statically linked):        [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-unknown-linux-musl-ubuntu-22.04.deb)
        * GNU  (Dynamic linking to glibc): [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-unknown-linux-gnu-ubuntu-22.04.deb)
* macOS:
    * v11: [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-apple-darwin-macos-11)
    * v12: [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-apple-darwin-macos-12)
* Windows:
    * MSVC: [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-pc-windows-msvc-windows-2022.exe)
    * GNU:  [download](https://github.com/pouriya/restcommander/releases/download/latest/restcommander-latest-x86_64-pc-windows-gnu-windows-2022.exe)

### Docker
* The main directory inside container is `/restcommander`.  
* The configuration file address is `/restcommander/config.toml`.
* You should copy your scripts inside `/restcommander/scripts`.  
* If you want to customize frontend code, You should copy your web assets to `/restcommander/www`.  

#### DockerHub
```shell
docker pull pouriya/restcommander
```
#### GitHub Container Registry
```shell
docker pull ghcr.io/pouriya/restcommander && docker tag ghcr.io/pouriya/restcommander pouriya/restcommander 
```


# Quick Start
Download RestCommander latest version from [installation](#installation) section.  
Open a new terminal.
Rename downloaded file (which contains version and OS info in its name) to `restcommander`:
```shell
mv restcommander-* restcommander
```
Create a new directory for your scripts:
```shell
mkdir scripts
```
Make a new script inside your script directory:
```shell
touch scripts/hello-world
echo '#! /usr/bin/env sh'  > scripts/hello-world
echo 'echo Hello World!'  >> scripts/hello-world
```
Make sure that the script is executable (for unix-like environments):  
```shell
chmod a+x scripts/hello-world
```
Test the script (It should print out `Hello World!`):
```shell
./scripts/hello-world
```
Run RestCommander on top of that directory:
```shell
./restcommander --www-enabled=true --commands-root-directory scripts
```
The output should be something like:
```text
2022/11/01 09:54:06.423939 WARN   restcommander::cmd::tree       No .yaml or .yml info file found for "scripts/hello-world.yml"
2022/11/01 09:54:06.424146 INFO   restcommander::http            Started server on http://127.0.0.1:1995/
```
Now open another terminal and run:  
```shell
curl -X POST -d="" http://127.0.0.1:1995/api/run/hello-world
```
The output should be:
```json
{"ok":true,"result":"Hello World!"}
```
And in RestCommander's terminal you see more logs:
```text
2022/11/01 09:58:08.993397 INFO   restcommander::cmd::runner     command "scripts/hello-world" exited with 0 exit-code
2022/11/01 09:58:08.993435 INFO   restcommander::http            127.0.0.1:41710 | "/api/run/hello-world" -> 200 (0.001284s)
```
Now in second terminal make a YAML info file for your script:  
```shell
touch scripts/hello-world.yaml
echo "description: A hello world example" >> scripts/hello-world.yaml 
```
Reload scripts via:  
```shell
curl http://127.0.0.1:1995/api/reload/commands
```
```json
{"ok":true,"result":null}
```
Now Open [http://127.0.0.1:1995/static/commands.html](http://127.0.0.1:1995/static/commands.html) inside your browser. You should see:  
<div style="text-align:center"><img alt="restcommander-quick-start-screenshot-1.png" src="https://github.com/pouriya/restcommander/releases/download/media/restcommander-quick-start-screenshot-1.png" /></div>

Click on `Hello World`:  
<div style="text-align:center"><img alt="restcommander-quick-start-screenshot-2.png" src="https://github.com/pouriya/restcommander/releases/download/media/restcommander-quick-start-screenshot-2.png" /></div>
 
Run the command:  
<div style="text-align:center"><img alt="restcommander-quick-start-screenshot-3.png" src="https://github.com/pouriya/restcommander/releases/download/media/restcommander-quick-start-screenshot-3.png" /></div>

Change script file contents:
```shell
echo '#! /usr/bin/env sh'                     > scripts/hello-world
echo 'echo INFO this is a log message >&2'   >> scripts/hello-world
echo 'echo "{\"foo\": {\"bar\": \"baz\"}}"'  >> scripts/hello-world
```
Now run the script again:  
```shell
curl -X POST -d='' http://127.0.0.1:1995/api/run/hello-world
```
```json
{"ok":true,"result":{"foo":{"bar":"baz"}}}
```
Additionally, you should see a log line like this:  
```text
2022/11/01 10:31:27.356416 INFO   restcommander::cmd::runner     "scripts/hello-world" -> this is a log message
```
Inside web dashboard run the script again:
<div style="text-align:center"><img alt="restcommander-quick-start-screenshot-4.png" src="https://github.com/pouriya/restcommander/releases/download/media/restcommander-quick-start-screenshot-4.png" /></div>


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
Use `--help` instead of `-h` to get more detailed help message for each option.  
`playground` subcommand is **not** recommended for long-time running because you can't reload the configuration without restarting the entire service.  
You can get a complete TOML configuration settings with `restcommander sample config` command and use it as a configuration file:  
```shell
# Get new configuration sample:
$ restcommander sample config > cfg.toml
# Edit configuration (if needed):
$ vim cfg.toml
# Start RestCommander from configuration file:
$ restcommander config cfg.toml
```
See the [TOML configuration sample](https://github.com/pouriya/restcommander/blob/master/samples/config.toml) for more info.

# Script information format
When you start RestCommander, It starts loading all executable files from configured `root_directory` and its sub-directories recursively. It does not know anything about them. It does not know what options they need to run. It does not know they are stateful or stateless. It would be great if it could run them with some specific options to get their information, but it's very dangerous. Imagine if you wrongly configure `root_directory` to a directory that contains some executables that you don't want RestCommander run them.  
So for every script it tries to read a `<SCRIPT_NAME>.yaml` file and get that information from that file. The file content is in form of:  
```yaml
description: "<DESCRIPTION>"
version: "<VERSION>"
state: <STATE>
options: <OPTIONS>
```
* **DESCRIPTION**: Script description. The default value is empty string.  
* **VERSION**: Script version. The default value is empty string.  
* **STATE**: If the script is stateful, this field tells RestCommander how to fetch script's current state. So this field is optional (for stateless scripts you don't need to define it).
    Its value is in form of:
    ```yaml
    options: <STATE_OPTIONS>
    ```
    or
    ```yaml
    constant: "<CONSTANT>"
    ```
    * **STATE_OPTIONS**: List of options that will be passed to the script.  
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
* **OPTIONS**: A YAML mapping in form of:
    ```yaml
    <OPTION_NAME>: <OPTION_DEFINITION>
    ```
    * **OPTION_NAME**: Option name. Note that RestCommander does not make any input option lowercase/uppercase. So you need to be careful in naming your input options.  
    * **OPTION_DEFINITION**: Another YAML mapping in form of:
        ```yaml
        description: <OPTION_DESCRIPTION>
        default_value: <OPTION_DEFAULT_VALUE>
        required: <REQUIRED>
        value_type: <OPTION_VALUE_TYPE>
        size: <OPTION_SIZE>
        ```
        * **OPTION_DESCRIPTION**: Description of this option. The default value is empty string.  
        * **OPTION_DEFAULT_VALUE**: The default value of this option. If the user did not set this option, the default value will be used. This field is optional if `required` is set to `true`.  
        * **REQUIRED**: `true` or `false`. If the option is required and no `default_value` is configured, the user has to set this option from client-side.  
        * **OPTION_VALUE_TYPE**: Type of value for this option. one of string literals `string`, `integer`, `float`, `bool`, or `enum`. Note that `enum` also has a value in form of:
            ```yaml
            enum:
              - foo
              - bar
              - baz
            ```
            The default value is ``any`` which can be anything.  
            Examples:  
            ```yaml
            value_type: string
            ```
            ```yaml
            value_type:
              enum: 
                - bob
                - alice
            ```
            ```yaml
            # So the value can be `true` or `false`
            value_type: bool
            ```
        * **OPTION_SIZE**: Another mapping in form of:
            ```yaml
            size:
              - min: <SIZE>
              - max: <SIZE>
            ```
            The **SIZE** can be integer or float. For options with `integer` or `float` value type, it can be < 0. 
            Defining the entire option or its keys are optional.  
            Examples:  
            ```yaml
            # The value can be a string up to 100 characters.
            value_type: string
            size:
              max: 100
            ```
            ```yaml
            value_type: integer
            size:
              min: -100
              max: 100
            ```

Get new YAML sample via `restcommander sample script-info`.  

#### Examples
```yaml
description: "Utility to set timezone"
state:
  # RestCommander will run `path/to/script current-timezone` to get current state
  options:
    - "current-timezone"
options:
  timezone:
    description: "A new timezone to set"
    value_type: string
    required: true
    size:
      # Minimum size of input for example "Asia/Tehran"
      min: 7
```
```yaml
description: "Manage foo service"
state:
  options:
    - "status"
options:
  action:
    value_type: 
      enum:
        - "start"
        - "restart"
        - "stop"
    default_value: "restart"
```
```yaml
description: "telnet to configured address"
options:
  host:
    description: "A hostname or IP address"
    value_type: "string"
    required: true
  port:
    description: "Port number"
    value_type: "integer"
    required: true
    size:
      min: 1
      max: 65535
```


# REST API
RestCommander has 4 main REST-API endpoints.
* [**/api/auth/token**](#apiauthtoken): Fetch a bearer token.  
* [**/api/commands**](#apicommands): Fetch service commands tree.  
* [**/api/run/...**](#apirun): Run a command.  
* [**/api/state/...**](#apistate): Fetch command state (if the script is stateful).  

## Authentication
You need a bearer token to work with RestCommander REST-API. You can configure `api_token` via commandline or inside your TOML configuration (This token does not get expired). Another way is to do an HTTP basic authentication to [/api/auth/token](#apiauthtoken) with your configured `username` and `password` (and `CAPTCHA` if configured) to get a new bearer token which will be expired after a configured time (see `token_timneout` in configuration).  
#### Recommended authentication flow
* Call [/api/auth/test](#apiauthtest) with no bearer token. If authentication is not configured, You get HTTP status-code `200` and no authentication is required.  
* If you got HTTP status-code `401`, Then authentication is required and you have to make a new request via your existing bearer token or get a new one from [/api/auth/token](#apiauthtoken).  


## /api
HTTP response for all endpoints are in form of:
Success:
```json
{"ok": true, "result": ...}
```
Failure:
```json
{"ok": false, "result": ...}
```
In failures the `result` value is the reason that why the failure occurs.  

### /api/public
There is no need to authenticate to use all endpoints under this endpoint.  

#### /api/public/captcha
Method: **GET**  
Success:
```json
{"id": "<CAPTCHA_UUID>", "image": "<BASE64_PNG_IMAGE>"}
```

Failures:  
* **406**: If the CAPTCHA feature is not configured or RestCommander does not have appropriate permissions to update configured CAPTCHA file.

#### /api/public/configuration
Method: **GET**  
Success:
It's a JSON object containing all `www.configuration` key/values from your [TOML configuration file](https://github.com/pouriya/restcommander/blob/master/samples/config.toml).  
```json
{"...":  "..."}
```

Failures: No failures.

## /api/auth

### /api/auth/test
You can test if your RestCommander service supports authentication or not and if it supports, You can test your bearer token too.  
Method: **GET**  
Success: Nothing.  
Failures:  
* **401**: Authentication failure.  

### /api/auth/token
Fetching new bearer token with your configured username/password inside your [TOML configuration file](https://github.com/pouriya/restcommander/blob/master/samples/config.toml).  
Method: **POST**  
If CAPTCHA is enabled, you should first get one from [/api/public/captcha](#apipubliccaptcha) and you need to set `Content-Type` header to `application/x-www-form-urlencoded` and put your `<CAPTCHA_ID>=<CAPTCHA_TEXT>` inside request body.  
Success:
```json
{"token": "<YOUR_BEARER_TOKEN>"}
```

Failures:  
* **401**: Authentication failure.   

## /api/commands
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
* `<INFO>`: Only present if `is_directory` is `true` and it is another object in form of:  
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
        * `<VALUE_TYPE>`:  one of `"string"` | `"integer"` | `"float"` | `"bool"` | `{"enum": ["...", "..."]}`.  
        * `<DEFAULT_VALUE>`:  The default value of option. (optional if the option itself is not required).  
        * `<SIZE>`: Another object in form of `{"min": NUMBER, "max": NUMBER}`. The whole object and its keys are optional.  
* `<COMMANDS>`: Another object containing the same structure. Only present if `is_directory` is `true` and the directory contains other commands or directories.  

Failures:  
* **401**: Authentication failure.

## /api/setPassword
Method: **POST**  
Request header `Content-Type` should be set to `application/json` and a body in form of `{"password": "<NEW_PASSWORD>"}` is required.  
Success: Nothing.  
Failures:  
* **400**: If provided password is empty.  
* **401**: Authentication failure.  
* **503**: If `password_file` is not configured and RestCommander is started just with `pasword_sha512` (a hardcoded password).  
* **500**: If RestCommander does not have appropriate permissions to update password file.  


## /api/reload
### /api/reload/commands
Method: **GET**  
Success: Nothing (So you have to fetch new commands from `/api/commands` endpoint).  
Failures:  
* **401**: Authentication failure.  
* **500**: If RestCommander could not reload scripts.  

### /api/reload/config
Method: **GET**  
Success: Nothing.  
Failures:
* **401**: Authentication failure.  
* **500**: If RestCommander could not reload configuration.  

## /api/run/...
For example if your script is in `foo/bar` sub-directory of your configured `commands.root_directory` and its filename is `baz` (it's `foo/bar/baz`), Then you have to send request to `/api/run/foo/bar/baz`.  
Method: **POST**  
You can set each command's input options to URL query-string, HTTP header (in form of `X-YOUR_OPTION_NAME`) or inside request body. Format of input options for each command is different, and it depends on command's configured input options in its YAML file.  
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
Note that HTTP response body is captured from command's `stdin`. RestCommander tries to make a JSON object from command's `stdin` and if it could not, Then the whole string is returned in `result` of response object.  
For example a script exited with exit-status `2` and printed `{"foo": "bar"}`, The HTTP response status-code will be `400` and the response body will be `{"ok": false, "result": {"foo": "bar"}}`.  


## /api/state/...
Method: **GET**  
If the command is stateful (according to its YAML options), RestCommander will run the command and HTTP response body is captured from command's `stdin`.  
Success: A JSON value which is current command state.  
Failures:
* **401**: Authentication failure.  
* **404**: Command is stateless and has no state.  

Other HTTP status-codes depend on command's exit-code which is the same as [/api/run/...](#apirun).  

# Contributing
[Backend Contributing](https://github.com/pouriya/restcommander/blob/master/CONTRIBUTING.md)  
[FrontEnd Contributing](https://github.com/pouriya/restcommander/blob/master/www/CONTRIBUTING.md)  
