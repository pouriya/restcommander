import {ApiOpts, Api} from './api.js'
import {maybeRemoveElement, setAttributes} from './utils.js'
import {setConfiguration} from './configuration.js'

async function maybeSetupCaptcha() {
    var captcha = await new Api(ApiOpts).captcha(true)
    if (captcha === false) {
        return
    }

    // Make new Elements:
    var captchaImageDivElement = document.createElement('div')
    setAttributes(captchaImageDivElement, {'id': 'captcha-image', 'class': 'mb-3 text-center'})
    var captchaImageElement = document.createElement('img')
    setAttributes(
        captchaImageElement,
            {
                'src': 'data:image/png;base64,' + captcha.image,
                'class': 'img-fluid',
                'alt': 'CAPTCHA',
                'style': 'max-width: 100%; height: auto;'
            }
        )
    captchaImageDivElement.appendChild(captchaImageElement)

    var captchaRenewElement = document.createElement('button')
    setAttributes(
        captchaRenewElement,
        {
            'id': 'captcha-renew',
            'type': 'button',
            'class': 'btn btn-outline-secondary w-100 mb-3'
        }
    )
    captchaRenewElement.innerHTML = 'Renew CAPTCHA'

    var captchaIdDivElement = document.createElement('div')
    setAttributes(captchaIdDivElement, {'id': 'captcha-id', 'class': 'd-none'})
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
    setAttributes(captchaTextDivElement, {'id': 'captcha-text', 'class': 'mb-3'})
    var captchaTextLabelElement = document.createElement('label')
    setAttributes(captchaTextLabelElement, {'for': 'captcha-text-value', 'class': 'form-label visually-hidden'})
    captchaTextLabelElement.innerHTML = 'CAPTCHA'
    var captchaTextElement = document.createElement('input')
    setAttributes(
        captchaTextElement,
        {
            'type': 'text',
            'id': 'captcha-text-value',
            'name': 'captcha-text-value',
            'class': 'form-control form-control-lg',
            'placeholder': 'CAPTCHA*',
            'required': 'required'
        }
    )
    captchaTextDivElement.appendChild(captchaTextLabelElement)
    captchaTextDivElement.appendChild(captchaTextElement)

    var oldCaptchaImage = document.getElementById('captcha-image')
    var oldCaptchaRenew = document.getElementById('captcha-renew')
    var oldCaptchaId = document.getElementById('captcha-id')
    var oldCaptchaText = document.getElementById('captcha-text')
    
    if (oldCaptchaImage) oldCaptchaImage.replaceWith(captchaImageDivElement)
    if (oldCaptchaRenew) oldCaptchaRenew.replaceWith(captchaRenewElement)
    if (oldCaptchaId) oldCaptchaId.replaceWith(captchaIdDivElement)
    if (oldCaptchaText) oldCaptchaText.replaceWith(captchaTextDivElement)
    
    // Show captcha elements
    captchaImageDivElement.classList.remove('d-none')
    captchaImageDivElement.classList.add('mb-3')
    captchaRenewElement.classList.remove('d-none')
    captchaRenewElement.classList.add('mb-3')
    captchaTextDivElement.classList.remove('d-none')
    captchaTextDivElement.classList.add('mb-3')
    captchaRenewElement.addEventListener('click', captchaRenewClickEventListener)
}


function hideLoginErrorElement() {
    var loginErrorElement = document.getElementById('login-error')
    loginErrorElement.innerHTML = ''
    if (loginErrorElement.classList.contains('d-none')) {
        return
    }
    loginErrorElement.classList.add('d-none')
    loginErrorElement.classList.remove('d-block')
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
    loginErrorElement.classList.remove('d-none')
    loginErrorElement.classList.add('d-block')
    await maybeSetupCaptcha()
}

async function main() {
    setConfiguration({'login-title': null, 'footer': null})
    document.body.classList.remove('invisible')
    document.body.classList.add('visible')
    document.getElementById('login-form').addEventListener('submit', loginSubmitEventListener)
    maybeSetupCaptcha()
}
window.main = main