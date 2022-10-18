# RestCommander
HTTP REST-API layer on top of scripts with a simple web dashboard.  

## Tour
RestCommander has a tiny docker image to show you how it works:
```shell
docker run -p1995:1995 ghcr.io/pouriya/restcommander:tour
```
After running above, Open [https://127.0.0.1:1995](https://127.0.0.1:1995) in your browser.  
Note that default username and password is `admin`.  

## Architecture
RestCommander is a simple REST-API layer on top of one or more scripts. From the request URL it detects which script it should run. It captures HTTP query-string parameters, headers, and body and after deserializing, merging, and validating inputs, passes them to the script. The script can read these inputs from [stdin](https://en.wikipedia.org/wiki/Standard_streams#Standard_input_(stdin)) or [environment variables](https://en.wikipedia.org/wiki/Environment_variable) to do different things. The script [stdout](https://en.wikipedia.org/wiki/Standard_streams#Standard_input_(stdin)) (whatever the script prints) is captured by RestCommander and that's the REST-API response body! Also, different script [exit-codes](https://en.wikipedia.org/wiki/Exit_status) causes different HTTP response status code.  
Additionally, RestCommander captures script [stderr](https://en.wikipedia.org/wiki/Standard_streams#Standard_error_(stderr)) for logging and some other operational things.  
A script can be stateless or stateful and RestCommander can capture current script state.  

## Features
* REST-API:  
    * HTTP and HTTPS.  
    * IP wildcard access.  
    * CAPTCHA.  
    * Static and dynamic authentication tokens.
* Web dashboard:  
    * Configurable Title, banner text, and footer.  
    * Extensible: You can serve your own front-code directory or replace RestCommander files.  
* Dynamic configuration reload. So you can change anything (even port number) without restart.  
* Single executable for macOS, Windows, and GNU/Linux.  

## Installation
Run the following in your terminal to download the latest version:
```shell
curl --proto '=https' --tlsv1.2 -sSfL https://github.com/pouriya/restcommander/releases/download/latest/install.sh | sh
```
or download latest version from here:  
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
`playground` subcommand is not recommended for production use because you can't reload the configuration without restarting the entire service.  
You can get a complete TOML configuration settings with `restcommander sample config` command and use it as a config file:
```shell
$ restcommander sample config > cfg.toml
$ vim cfg.toml # Edit configuration (if needed)
$ restcommander config cfg.toml
```
See the [TOML configuration sample](https://github.com/pouriya/restcommander/blob/master/samples/config.toml) for more info.  

# Documentation
[REST API](https://github.com/pouriya/restcommander/blob/master/REST_API.md)  
[Backend Contributing](https://github.com/pouriya/restcommander/blob/master/CONTRIBUTING.md)  
[FrontEnd Contributing](https://github.com/pouriya/restcommander/blob/master/www/CONTRIBUTING.md)  
