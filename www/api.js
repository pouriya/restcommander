import {getUrlWithBasePath} from './utils.js'

const ApiOpts = {
    url: await getUrlWithBasePath() + 'api/',
}

class Api {
    constructor(options) {
        this.options = options
    }

    async fetch(endpoint, filterFunction, method, extraHeaders, body) {
        if (method === undefined) {
           method = 'GET'
        }
        if (extraHeaders === undefined) {
           extraHeaders = {}
        }
        if (body === undefined) {
           body = ''
        }
        console.log(endpoint)
        var url = this.options.url + endpoint
        if (endpoint.startsWith('/') === true) {
            url = endpoint
        }
        var headers = {'Accept': 'application/json', 'Content-Type': 'application/json'}
        for (const extraHeaderKey in extraHeaders) {
           headers[extraHeaderKey] = extraHeaders[extraHeaderKey]
        }
        await console.log('Attempt to do a', method, 'call to', url, 'with', Object.keys(extraHeaders).length, 'extra headers and', body.length, 'bytes body')
        var requestOptions = {method: method, headers: headers}
        if (body !== '') {
           requestOptions.body = body
        }
        var result = {ok: false, result: 'No request is made', code: 0, status: 0, message: undefined}
        await fetch(url, requestOptions)
           .catch(
               async function (error) {
                   result.result = error.message
               }
           )
           .then(
               async function (response) {
                   if (response === undefined) {
                       return
                   }
                   const apiResult = await response.json()
                   result.status = response.status
                   result.message = response.statusText
                   result.ok = apiResult.ok
                   if (result.ok === false) {
                        result.result = 'A service error occurred.\nPlease contact service administrator for more information.'
                   }
                   if (apiResult.hasOwnProperty('result')) {
                        result.result = apiResult.result
                   }
                   if (apiResult.hasOwnProperty('code')) {
                       result.code = apiResult.code
                   }
                   if (result.result === null) {
                       if (result.status < 300) {
                           result.result = 'Done.'
                       } else if (result.status >= 400) {
                           result.result = 'Failed. For more information check server logs.'
                       } else {
                            console.log('Unhandled status code for defining `result.result`', result.status)
                       }
                   }
               }
           )
        if (result.ok) {
           await console.log(url, '=>', result.result)
        } else {
           await console.log(url, '=>', result.result, 'with error code', result.code)
        }
        if (filterFunction === true) {
            result = await this.unwrapResult(result)
        } else if (filterFunction !== undefined) {
            result = await filterFunction(result)
        }
        return result
    }

    async unwrapResult(result) {
        if (result.ok === true) {
           return result.result
        }
        return false
    }

    async captcha(filterFunction) {
        return this.fetch('public/captcha', filterFunction)
    }

    async configuration(filterFunction) {
        return this.fetch('public/configuration', filterFunction)
    }

    async auth(username, password, captchaId, captchaText, filterFunction) {
        const extraHeaders = {
           'Authorization': 'Basic ' + btoa(username + ':' + password),
           'Content-Type': 'application/x-www-form-urlencoded'
        }
        var body = ''
        if (captchaId !== undefined && captchaText !== undefined) {
           body = captchaId + '=' + captchaText
        }
        return this.fetch('auth/token', filterFunction, 'POST', extraHeaders, body)
    }

    async testAuth(filterFunction) {
        return this.fetch('auth/test', filterFunction)
    }

    async commands(filterFunction) {
        return this.fetch('commands', filterFunction)
    }

    async run(http_path, options, filterFunction) {
        return this.fetch(http_path, filterFunction, 'POST', {'X-RESTCOMMANDER-STATISTICS': "true"}, JSON.stringify(options))
    }

    async reloadConfig(filterFunction) {
        return this.fetch('reload/config', filterFunction)
    }

    async reloadCommands(filterFunction) {
        return this.fetch('reload/commands', filterFunction)
    }

    async setPassword(password, filterFunction) {
        return this.fetch('setPassword', filterFunction, 'POST', {}, JSON.stringify({'password': password}))
    }

    async state(http_path, filterFunction) {
        return this.fetch(http_path, filterFunction, 'GET', {'X-RESTCOMMANDER-STATISTICS': "true"})
    }

    async report(before_time, after_time, context, from, limit, filterFunction) {
        var body = {}
        if (before_time !== undefined && before_time !== null && before_time !== '') {
            body['before_time'] = before_time
        }
        if (after_time !== undefined && after_time !== null && after_time !== '') {
            body['after_time'] = after_time
        }
        if (context !== undefined && context !== null && context !== '') {
            body['context'] = context
        }
        if (from !== undefined && from !== null && from !== '') {
            body['from'] = from
        }
        if (limit !== undefined && limit !== null && limit !== '') {
            body['limit'] = limit
        }
        return this.fetch('report', filterFunction, 'POST', {}, JSON.stringify(body))
    }
}

export {ApiOpts, Api}
