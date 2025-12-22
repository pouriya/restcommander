import {ApiOpts, Api} from './api.js'
import {setAttributes} from './utils.js'
import {setConfiguration} from './configuration.js'

async function main() {
    setConfiguration({'banner-title': null, 'banner-text': null, 'footer': null})
    const testAuth = await new Api(ApiOpts).testAuth(true)
    var nextPageLink = '/static/commands.html'
    var nextPageName = 'Commands'
    if (testAuth === false) {
        nextPageLink = '/static/login.html'
        nextPageName = 'Login'
    }
    var nextPageLinkElement = document.getElementById('next-page-link')
    nextPageLinkElement.setAttribute('href', nextPageLink)
    nextPageLinkElement.innerHTML = nextPageName
    document.body.className = 'visible ' + document.body.className.replace('invisible ', '')
}

window.main = main