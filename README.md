# RestCommander
HTTP REST-API layer on top of scripts with a simple web dashboard.

## Features
* REST-API:  
    * HTTP and HTTPS.  
    * IP wildcard access.  
    * CAPTCHA.  
    * Static and dynamic authentication tokens.
* Web dashboard:  
    * Configurable Title, banner text, and footer.  
    * Extensible: You can serve your own frontend codes or replace RestCommander files.
* Reports: A generic reporting system for your scripts. You can search in reports from REST-API or simply from web dashboard.  
* Dynamic configuration reload. So you can change anything (even port number) without restart.  
* Single executable for macOS, Windows, and GNU/Linux.  

For more information see [**Quick Start**](https://github.com/pouriya/restcommander/blob/master/DOCUMENTATION.md#quick-start) or [**Documentation**](https://github.com/pouriya/restcommander/blob/master/DOCUMENTATION.md).  

## Installation
Run the following in your terminal to download the latest version:
```shell
curl -sSfL https://github.com/pouriya/restcommander/releases/download/latest/install.sh | sh
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
docker pull ghcr.io/pouriya/restcommander && docker tag ghcr.io/pouriya/restcommander pouriya/restcommander 
```

## Contributing
[Backend Contributing](https://github.com/pouriya/restcommander/blob/master/CONTRIBUTING.md)  
[Frontend Contributing](https://github.com/pouriya/restcommander/blob/master/www/CONTRIBUTING.md)  
