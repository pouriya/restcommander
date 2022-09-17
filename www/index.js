import {ApiOpts, Api} from './api.js'
import {maybeRemoveElement, setAttributes} from './utils.js'

async function main() {
    const configuration = await new Api(ApiOpts).configuration(true)
    if (configuration !== false) {
        if ('serviceName' in configuration) {
            document.getElementById('service-name').innerHTML = configuration.serviceName
            document.title = configuration.serviceName
        } else {
            console.log('Could not found `serviceName` in server configuration')
        }
        if ('bannerTitle' in configuration) {
            document.getElementById('banner-title').innerHTML = configuration.bannerTitle
        } else {
            console.log('Could not found `bannerTitle` in server configuration')
        }
        if ('bannerText' in configuration) {
            document.getElementById('banner-text').innerHTML = configuration.bannerText
        } else {
            console.log('Could not found `bannerText` in server configuration')
        }
        if ('bannerFooter' in configuration) {
            document.getElementById('footer').innerHTML = configuration.bannerFooter
        } else {
            console.log('Could not found `bannerFooter` in server configuration')
        }
    }
    const testAuth = await new Api(ApiOpts).testAuth(true)
    if (testAuth !== false) {
        document.location = '/static/commands.html'
        return
    }
    document.body.className = 'visible ' + document.body.className.replace('invisible', '')
}

window.main = main