import {ApiOpts, Api} from './api.js'


const defaultConfiguration = {
    'title': 'mcpd',
    'login-title': 'Login to {{title}}',
    'banner-title': 'MCP Daemon',
    'banner-text': '{{title}} exposes your scripts as MCP tools and resources',
    'footer': 'Hosted on <a href="https://github.com/pouriya/mcpd" target="_blank"><b>GitHub</b></a>'
}

var configuration = false

async function setConfiguration(elements) {
    configuration = await new Api(ApiOpts).configuration(true)
    if (configuration !== false) {
        configuration = {...defaultConfiguration, ...configuration}
        for (const key in configuration) {
            if (key === 'title') {
                continue
            }
            configuration[key] = configuration[key].replaceAll('{{title}}', configuration['title'])
        }
        console.log('Merged and rendered configuration:', configuration)
        setTitle()
        setElements(elements)
        return true
    }
    return false
}

async function setTitle(maybeConfiguration) {
    if (maybeConfiguration === undefined) {
        maybeConfiguration = configuration
    }
    if (maybeConfiguration === false) {
        return false
    }
    if (maybeConfiguration['title'] !== '') {
        document.title = maybeConfiguration['title']
    }
}

async function setElements(elements, maybeConfiguration) {
    if (maybeConfiguration === undefined) {
        maybeConfiguration = configuration
    }
    if (maybeConfiguration === false) {
        return false
    }
    for (const key in elements) {
        if (key in maybeConfiguration) {
            var element = document.getElementById(key)
            if (element !== null) {
                console.log('Set value of element with id', key, 'to', maybeConfiguration[key])
                element.innerHTML = maybeConfiguration[key]
            } else {
                console.log('Could not found element with id', key, 'to set its value from configuration')
            }
        } else {
            console.log('Could not found element id', key, 'in configuration')
            if (elements[key] !== null) {
                console.log('Set provided default value for element with id', key, 'to', elements[key])
                element.innerHTML = elements[key]
            }
        }
    }
}

export {defaultConfiguration, configuration, setConfiguration, setTitle, setElements}
