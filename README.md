# RestCommander
Place your own scripts behind a REST-API server and run them from a simple web dashboard.  

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
