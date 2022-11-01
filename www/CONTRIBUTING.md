# Foreword
Before I started to implement the web dashboard, I knew nothing about JS! I wrote the entire front code within a week (in my free times). I do know that the frontend code is shitty but **it works**.  

## To contributors
I really appreciate you for helping but there is one and only one rule to know. **I do NOT want to use any other frontend libraries and build tools except Bootstrap v5**. After all I'm the only one that maintains the project, and I do not have enough free time to learn new JS/CSS libraries and build tools. So if, you want to help, You should enhance/bug-fix current implementation. If the current implementation does not fit your needs, You are free to build your own dashboard and RestCommander will serve it for you!  

## Development
RestCommander backend code is written in Rust. I guess you are a frontend developer and not familiar with Rust and, it's difficult for you to set up a development environment for this project. So in this explanation we use Docker.  
Pull latest RestCommander version from DockerHub
```shell
docker pull pouriya/restcommander
```
or from GitHub Container Registry:
```shell
docker pull ghcr.io/pouriya/restcommander && docker tag ghcr.io/pouriya/restcommander pouriya/restcommander 
```
Fork RestCommander under your GitHub account and clone RestCommander source code:
```shell
git clone --depth=1 --branch=master git@github.com:<YOUR_USERNAME>/restcommander.git
```
Make a new directory for your development:
```shell
mkdir restcommander-front-codes
```
Copy frontend codes from cloned RestCommander repository to your newly created directory and remove unwanted files:
```shell
cp restcommander/www/* restcommander-front-codes/ 
rm -f restcommander-front-codes/*md restcommander-front-codes/bootstrap-version.txt
```
Make a new folder and a test script:
```shell
mkdir restcommander-scripts
docker run pouriya/restcommander sample script > restcommander-scripts/test
docker run pouriya/restcommander sample script-info > restcommander-scripts/test.yml
chmod a+x restcommander-scripts/test
```
Now start a docker container and make a new volumes to set its `www` and `scripts` directories to your newly created directories:  
```shell
docker run --init -it -p 1995:1995 -v /absolute/path/to/your/restcommander-front-codes/:/restcommander/www/ -v /absolute/path/to/your/restcommander-scripts/:/restcommander/scripts/ pouriya/restcommander
```
Now open [https://127.0.0.1:1995](https://127.0.0.1:1995) in your web browser, and you should see RestCommander web dashboard.  
After above steps, You are free to update any file inside your `restcommander-front-codes` directory and inside browser if you reload the pages, You will see your changes.  
If you were ready to make a PR, copy frontend codes back to your cloned RestCommander repository:  
```shell
cp restcommander-front-codes/* restcommander/www/ 
```
Then go inside repository and make a new branch related to your work (for example `enhance-login-page-css`):  
```shell
cd restcommander
git checkout -b enhance-login-page-css
```
Now you are ready to commit, push and make a PR.  
