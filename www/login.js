import {ApiOpts, Api} from './api.js'
import {maybeRemoveElement, setAttributes} from './utils.js'

async function maybeSetupCaptcha() {
    var captcha = await new Api(ApiOpts).captcha(true)
    if (captcha === false) {
        return
    }

    // Make new Elements:
    var captchaImageDivElement = document.createElement('div')
    setAttributes(captchaImageDivElement, {'id': 'captcha-image', 'class': 'form-outline mb-4'})
    var captchaImageElement = document.createElement('img')
    setAttributes(
        captchaImageElement,
            {
                'src': 'data:image/png;base64,' + captcha.image,
                'class': 'img-fluid',
                'alt': 'CAPTCHA'
            }
        )
    captchaImageDivElement.appendChild(captchaImageElement)

    var captchaRenewElement = document.createElement('button')
    setAttributes(
        captchaRenewElement,
        {
            'id': 'captcha-renew',
            'type': 'button',
            'class': 'btn btn-outline-secondary btn-block mb-4 justify-content-center'
        }
    )
    captchaRenewElement.innerHTML = 'Renew CAPTCHA'

    var captchaIdDivElement = document.createElement('div')
    setAttributes(captchaIdDivElement, {'id': 'captcha-id', 'class': 'form-outline invisible d-none'})
    var captchaIdElement = document.createElement('input')
    setAttributes(
        captchaIdElement,
        {
            'value': captcha.id,
            'id': 'captcha-id-value',
            'name': 'captcha-id-value',
            'class': 'form-outline'
        }
    )
    captchaIdDivElement.appendChild(captchaIdElement)

    var captchaTextDivElement = document.createElement('div')
    setAttributes(captchaTextDivElement, {'id': 'captcha-text', 'class': 'form-outline mb-4'})
    var captchaTextElement = document.createElement('input')
    setAttributes(
        captchaTextElement,
        {
            'type': 'text',
            'id': 'captcha-text-value',
            'name': 'captcha-text-value',
            'class': 'form-control',
            'placeholder': 'CAPTCHA*',
            'required': 'required'
        }
    )
    captchaTextDivElement.appendChild(captchaTextElement)

    document.getElementById('captcha-image').replaceWith(captchaImageDivElement)
    document.getElementById('captcha-renew').replaceWith(captchaRenewElement)
    document.getElementById('captcha-id').replaceWith(captchaIdDivElement)
    document.getElementById('captcha-text').replaceWith(captchaTextDivElement)
    captchaRenewElement.addEventListener('click', captchaRenewClickEventListener)
}


function hideLoginErrorElement() {
    var loginErrorElement = document.getElementById('login-error')
    loginErrorElement.innerHTML = ''
    if (loginErrorElement.className.includes('invisible')) {
        return
    }
    loginErrorElement.className = 'invisible ' + loginErrorElement.className.replace('visible', '')
}

async function captchaRenewClickEventListener() {
    event.preventDefault()
    hideLoginErrorElement()
    await maybeSetupCaptcha()
}

async function loginSubmitEventListener(event) {
    event.preventDefault()
    hideLoginErrorElement()
    var inputs = new FormData(event.target);
    const username = inputs.get('username')
    const password = inputs.get('password')
    var captchaId = inputs.get('captcha-id-value')
    if (captchaId === null) {
        captchaId = undefined
    }
    var captchaText = inputs.get('captcha-text-value')
    if (captchaText === null) {
        captchaText = undefined
    }
    const authResult = await new Api(ApiOpts).auth(
        username,
        password,
        captchaId,
        captchaText,
        async function(result) {
            if (result.ok) {
                return true
            }
            return result.result
        }
    )
    if (authResult === true) {
        document.location = 'commands.html'
        return
    }
    var loginErrorElement = document.getElementById('login-error')
    loginErrorElement.innerHTML = authResult + ''
    loginErrorElement.className = 'visible ' + loginErrorElement.className.replace('invisible ', '')
    await maybeSetupCaptcha()
}

async function main() {
    const configuration = await new Api(ApiOpts).configuration(true)
    if (configuration !== false) {
        if ('service_name' in configuration) {
            if (configuration.service_name !== '') {
                document.getElementById('login-title').innerHTML = 'Login to ' + configuration.service_name
                document.title = configuration.service_name
            } else {
                document.getElementById('login-title').innerHTML = 'Login'
            }
        } else {
            console.log('Could not found `service_name` in server configuration')
            document.getElementById('login-title').innerHTML = 'Login'
        }
    }
    document.body.className = 'visible'
    document.getElementById('login-form').addEventListener('submit', loginSubmitEventListener)
    maybeSetupCaptcha()
}
window.main = main