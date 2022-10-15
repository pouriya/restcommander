import {ApiOpts, Api} from './api.js'
import {maybeRemoveElement, setAttributes} from './utils.js'

async function main() {
    const configuration = await new Api(ApiOpts).configuration(true)
    if (configuration !== false) {
        if ('service_name' in configuration) {
            document.getElementById('service-name').innerHTML = configuration.service_name
            document.title = configuration.service_name
        } else {
            console.log('Could not found `service_name` in server configuration')
        }
        if ('banner_title' in configuration) {
            document.getElementById('banner-title').innerHTML = configuration.banner_title
        } else {
            console.log('Could not found `banner_title` in server configuration')
        }
        if ('banner_text' in configuration) {
            document.getElementById('banner-text').innerHTML = configuration.banner_text
        } else {
            console.log('Could not found `banner_text` in server configuration')
        }
        if ('footer' in configuration) {
            document.getElementById('footer').innerHTML = configuration.footer
        } else {
            console.log('Could not found `footer` in server configuration')
        }
    }
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
    document.body.className = 'visible ' + document.body.className.replace('invisible', '')
}

window.main = main