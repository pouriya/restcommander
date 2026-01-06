import {getUrlWithBasePath, tryHash} from './utils.js'

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
        console.log('Attempt to do a', method, 'call to', url, 'with', Object.keys(extraHeaders).length, 'extra headers and', body.length, 'bytes body')
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
           console.log(url, '=>', result.result)
        } else {
           console.log(url, '=>', result.result, 'with error code', result.code)
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
        const hashResult = tryHash(password, 'password')
        const hashedPassword = hashResult.password
        const isHashed = hashResult.hash
        const extraHeaders = {
           'Authorization': 'Basic ' + btoa(username + ':' + hashedPassword),
           'Content-Type': 'application/x-www-form-urlencoded',
           'X-RESTCOMMANDER-PASSWORD-HASHED': isHashed.toString()
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

    async setPassword(password, previousPassword, filterFunction) {
        const hashResult = tryHash(password, 'password')
        const previousHashResult = tryHash(previousPassword, 'previous_password')
        return this.fetch('setPassword', filterFunction, 'POST', {}, JSON.stringify({
            'password': hashResult.password,
            'hash': hashResult.hash,
            'previous_password': previousHashResult.previous_password
        }))
    }

    // === MCP Methods ===
    async mcp(method, params = {}) {
        const request = { jsonrpc: "2.0", id: Date.now(), method, params }
        const url = this.options.url + 'mcp'
        const response = await fetch(url, {
            method: 'POST',
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            },
            credentials: 'include',
            body: JSON.stringify(request)
        })
        
        // Handle HTTP 204 (No Content) for notifications
        if (response.status === 204) {
            return { ok: true, result: null, status: 204 }
        }
        
        const jsonRpcResponse = await response.json()
        
        // Check for JSON-RPC error
        if (jsonRpcResponse.error) {
            return {
                ok: false,
                code: jsonRpcResponse.error.code,
                result: jsonRpcResponse.error.message,
                error: jsonRpcResponse.error,
                status: response.status
            }
        }
        
        // Return success with result
        return {
            ok: true,
            result: jsonRpcResponse.result,
            status: response.status
        }
    }

    async mcpInitialize() {
        return this.mcp('initialize', {})
    }

    async mcpToolsList() {
        // Returns raw MCP tools/list response
        return this.mcp('tools/list', {})
    }

    async mcpToolsCall(toolName, args = {}) {
        const result = await this.mcp('tools/call', { name: toolName, arguments: args })
        if (!result.ok) {
            return { ok: false, status: 500, result: result.result, code: result.code }
        }
        // Extract content from MCP response format
        const content = result.result.content || []
        const textContent = content.find(c => c.type === 'text')?.text || ''
        const isError = result.result.isError || false
        
        // Try to parse JSON if possible, otherwise use string as-is
        let parsedResult = textContent
        try {
            parsedResult = JSON.parse(textContent)
        } catch (e) {
            // Not JSON, use string as-is
            parsedResult = textContent
        }
        
        return {
            ok: !isError,
            status: isError ? 500 : 200,
            result: parsedResult,
            code: isError ? -32001 : 0
        }
    }

    async mcpResourcesList() {
        return this.mcp('resources/list', {})
    }

    async mcpResourcesRead(uri) {
        const result = await this.mcp('resources/read', { uri: uri })
        if (!result.ok) {
            return { ok: false, status: 500, result: result.result, code: result.code }
        }
        // Extract content from MCP response format
        const contents = result.result.contents || []
        const content = contents[0]
        if (!content) {
            return { ok: false, status: 404, result: 'No content found', code: -32004 }
        }
        // Parse JSON text content
        try {
            const parsed = JSON.parse(content.text)
            return { ok: true, status: 200, result: parsed, code: 0 }
        } catch (e) {
            return { ok: true, status: 200, result: content.text, code: 0 }
        }
    }
}

export {ApiOpts, Api}
