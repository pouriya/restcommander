_This is a contributing guideline for backend code contribution. For frontend code contributing see [here](https://github.com/pouriya/restcommander/blob/master/www/CONTRIBUTING.md)._  

# To contributors
I ❤️ PR from everyone, and I appreciate your help but before opening a PR, make an issue and describe your feature, bug, etc.  


## Set up a development environment
If you have `Rust` toolchain and GNU `make` installed, You just need to run `make start-dev` inside project root directory. It makes a new directory `tmp` inside project and starts a configured RestCommander inside it.  
Test directory structure:  
```shell
tree tmp
```
```text
tmp
├── cert.pem
├── config.toml
├── key.pem
├── password-file.sha512
├── scripts
│       ├── test
│       └── test.yml
└── www
    ├── api.js
    ├── bootstrap.bundle.min.js
    ├── bootstrap.min.css
    ├── commands.html
    ├── commands.js
    ├── CONTRIBUTING.md
    ├── favicon.ico
    ├── index.html
    ├── index.js
    ├── login.html
    ├── login.js
    ├── restcommander-background-image.jpg
    └── utils.js
```
The server will start in debug mode on `https://127.0.0.1:1995/`. For more info see other `Makefile` targets.  
