import {ApiOpts, Api} from './api.js'
import {maybeRemoveElement, setAttributes} from './utils.js'

async function drawNavbar() {
    const commandsResult = await new Api(ApiOpts).commands(true)
    if (commandsResult === false) {
        return false
    }
    const commands = commandsResult.commands
    await console.log('Got', Object.keys(commands).length, 'command(s)')

    var navigationBarElement = document.getElementById('navigation-bar')
    navigationBarElement.innerHTML = ''
    doDrawNavbar(commands, navigationBarElement, 1)
}

async function doDrawNavbar(commands, parentElement, depth) {
    var count = 0;
    await console.log('depth', depth, '|', 'command count:', Object.keys(commands).length)
    for (const key in commands) {
        count++;
        const command = commands[key];
        const keyName = key.replaceAll('-', ' ').replaceAll('_', ' ')
        await console.log('count:', count, '|', 'keyName:', keyName)
        if (depth === 1) {
            const id = 'dropdownMenuLink-'+ depth.toString() + '-' + count.toString()
            var aElement = document.createElement('a')
            setAttributes(
                aElement,
                {
                    'class': 'btn border-0 btn-secondary dropdown-toggle text-capitalize',
                     'href': "#",
                     'role': 'button',
                     'id': id,
                     'data-bs-toggle': 'dropdown',
                     'data-bs-auto-close': 'outside',
                     'aria-expanded': 'false'
                }
            )
            aElement.innerHTML = keyName
            parentElement.appendChild(aElement)
            var ulElement = document.createElement('ul')
            setAttributes(
                ulElement,
                {
                    'class': 'dropdown-menu',
                    'aria-labelledby': id
                }
            )
            if (command.is_directory) {
                await doDrawNavbar(command.commands, ulElement, depth+1)
            } else {
                var liElement = document.createElement('li')
                var aElement = document.createElement('a')
                setAttributes(
                    aElement,
                    {
                        'class': 'dropdown-item text-capitalize',
                        'href': "#"
                    }
                )
                aElement.innerHTML = keyName
                await addCommandClickEventListener(keyName, command, aElement)
                liElement.appendChild(aElement)
                ulElement.appendChild(liElement)
            }
            parentElement.appendChild(ulElement)
        } else {
            const id = 'dropdownSubMenuLink-'+ depth.toString() + '-' + count.toString()
            if (command.is_directory) {
                var liElement = document.createElement('li')
                setAttributes(
                    liElement,
                    {
                        'class': 'dropend'
                    }
                )
                var aElement = document.createElement('a')
                setAttributes(
                    aElement,
                    {
                        'class': 'dropdown-item text-capitalize',
                        'href': "#",
                        'id': id,
                        'data-bs-toggle': 'dropdown',
                        'data-bs-auto-close': 'outside',
                        'aria-expanded': 'false'
                    }
                )
                aElement.innerHTML = keyName + ' »'
                liElement.appendChild(aElement)
                var ulElement = document.createElement('ul')
                setAttributes(
                    ulElement,
                    {
                        'class': 'dropdown-menu',
                        'aria-labelledby': id
                    }
                )
                await doDrawNavbar(command.commands, ulElement, depth+1)
                liElement.appendChild(ulElement)
                parentElement.appendChild(liElement)
            } else {
                var liElement = document.createElement('li')
                var aElement = document.createElement('a')
                setAttributes(
                    aElement,
                    {
                        'class': 'dropdown-item text-capitalize',
                        'href': "#"
                    }
                )
                aElement.innerHTML = keyName
                await addCommandClickEventListener(keyName, command, aElement)
                liElement.appendChild(aElement)
                parentElement.appendChild(liElement)
            }
        }
    }
    if (depth === 1) {
        count++
        appendSettings(parentElement, count)
    }
}

function appendSettings(element, count) {
    const id = 'dropdownMenuLink-1-' + count.toString()
    var aElement = document.createElement('a')
    setAttributes(
        aElement,
        {
            'class': 'btn border-0 btn-secondary dropdown-toggle text-capitalize',
             'href': "#",
             'role': 'button',
             'id': id,
             'data-bs-toggle': 'dropdown',
             'data-bs-auto-close': 'outside',
             'aria-expanded': 'false'
        }
    )
    aElement.innerHTML = 'Settings'
    element.appendChild(aElement)
    var ulElement = document.createElement('ul')
    setAttributes(
        ulElement,
        {
            'class': 'dropdown-menu',
            'aria-labelledby': id
        }
    )
    var LogoutLiElement = document.createElement('li')
    var LogoutAElement = document.createElement('a')
    setAttributes(
        LogoutAElement,
        {
            'class': 'dropdown-item text-capitalize',
            'href': "#",
            'id': 'settings-logout'
        }
    )
    LogoutAElement.innerHTML = 'Logout'
    LogoutAElement.onclick = async function() {
        document.cookie = 'token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:01 GMT;'
        document.location = 'index.html'
    }
    LogoutLiElement.appendChild(LogoutAElement)
    ulElement.appendChild(LogoutLiElement)

    var ReloadConfigLiElement = document.createElement('li')
    var ReloadConfigAElement = document.createElement('a')
    setAttributes(
        ReloadConfigAElement,
        {
            'class': 'dropdown-item text-capitalize',
            'href': "#"
        }
    )
    ReloadConfigAElement.innerHTML = 'Reload Configuration'
    ReloadConfigAElement.onclick = async function() {
        document.getElementById('command').innerHTML = ''
        updateResultBeforeRequest()
        var reloadResult = await new Api(ApiOpts).reloadConfig()
        if (reloadResult.ok === true) {
            reloadResult.result = 'Configuration reloaded successfully.'
        }
        updateResultAfterRequest(reloadResult)
        if (reloadResult.status === 401) {
            changeLogoutToLogin()
        }
    }
    ReloadConfigLiElement.appendChild(ReloadConfigAElement)
    ulElement.appendChild(ReloadConfigLiElement)

    var ReloadCommandsLiElement = document.createElement('li')
    var ReloadCommandsAElement = document.createElement('a')
    setAttributes(
        ReloadCommandsAElement,
        {
            'class': 'dropdown-item text-capitalize',
            'href': "#"
        }
    )
    ReloadCommandsAElement.innerHTML = 'Reload Commands'
    ReloadCommandsAElement.onclick = async function() {
        document.getElementById('command').innerHTML = ''
        updateResultBeforeRequest()
        var reloadResult = await new Api(ApiOpts).reloadCommands()
        if (reloadResult.ok === true) {
            reloadResult.result = 'Commands reloaded successfully.'
        }
        updateResultAfterRequest(reloadResult)
        if (reloadResult.ok === true) {
            drawNavbar()
        }
        if (reloadResult.status === 401) {
            changeLogoutToLogin()
        }
    }
    ReloadCommandsLiElement.appendChild(ReloadCommandsAElement)
    ulElement.appendChild(ReloadCommandsLiElement)

    var setPasswordLiElement = document.createElement('li')
    var setPasswordAElement = document.createElement('a')
    setAttributes(
        setPasswordAElement,
        {
            'class': 'dropdown-item text-capitalize',
            'href': "#"
        }
    )
    setPasswordAElement.innerHTML = 'Set New Password'
    setPasswordAElement.onclick = async function() {
        var commandElement = document.getElementById('command')
        commandElement.innerHTML = ''
        document.getElementById('command-result').innerHTML = ''

        var headerElement = document.createElement('h')
        setAttributes(
            headerElement,
            {'class': 'h3 my-4'}
        )
        headerElement.innerHTML = 'Change Password'
        commandElement.appendChild(headerElement)

        var textElement = document.createElement('p')
        setAttributes(
            textElement,
            {'class': 'my-3 text-start text-break'}
        )
        textElement.innerHTML = 'Set your new dashboard password.'
        commandElement.appendChild(textElement)

        var formElement = document.createElement('form')
        setAttributes(
            formElement,
            {'class': ''}
        )
        var passwordDivElement = document.createElement('div')
        var passwordInputElement = document.createElement('input')
        setAttributes(
            passwordInputElement,
            {
                'class': 'form-control',
                'type': 'password',
                'id': 'password',
                'name': 'password',
                'placeholder': 'Password*',
                'required': 'required'
            }
        )
        passwordDivElement.appendChild(passwordInputElement)
        formElement.appendChild(passwordDivElement)
        var setPasswordButtonElement = document.createElement('button')
        setAttributes(
            setPasswordButtonElement,
            {
                'class': 'btn btn-sm btn-primary btn-block mt-3 justify-content-center fw-bold',
                'type': 'submit'
            }
        )
        setPasswordButtonElement.innerHTML = 'Apply'
        formElement.appendChild(setPasswordButtonElement)
        commandElement.appendChild(formElement)
        async function submitHandler(event) {
            event.preventDefault()
            var inputs = new FormData(event.target);
            const password = inputs.get('password')
            updateResultBeforeRequest()
            const setPasswordResult = await new Api(ApiOpts).setPassword(password)
            if (setPasswordResult.ok === true) {
                setPasswordResult.result = 'Password Changed Successfully.'
            }
            updateResultAfterRequest(setPasswordResult)
            if (setPasswordResult.status === 401) {
                changeLogoutToLogin()
            }
        }
        formElement.addEventListener('submit', submitHandler)
    }
    setPasswordLiElement.appendChild(setPasswordAElement)
    ulElement.appendChild(setPasswordLiElement)

    var reportLiElement = document.createElement('li')
    var reportAElement = document.createElement('a')
    setAttributes(
        reportAElement,
        {
            'class': 'dropdown-item text-capitalize',
            'href': "#"
        }
    )
    reportAElement.innerHTML = 'Reports'
    reportAElement.onclick = async function() {
        var commandElement = document.getElementById('command')
        commandElement.innerHTML = ''
        document.getElementById('command-result').innerHTML = ''

        var headerElement = document.createElement('h')
        setAttributes(
            headerElement,
            {'class': 'h3 my-4'}
        )
        headerElement.innerHTML = 'Reports'
        commandElement.appendChild(headerElement)

        var textElement = document.createElement('p')
        setAttributes(
            textElement,
            {'class': 'my-3 text-start text-break'}
        )
        textElement.innerHTML = 'Filter service reports.'
        commandElement.appendChild(textElement)

        var formElement = document.createElement('form')
        setAttributes(
            formElement,
            {'class': 'text-start'}
        )

        var afterTimeDivElement = document.createElement('div')
        setAttributes(
            afterTimeDivElement,
            {'class': 'form-group'}
        )
        var afterTimeLabelElement = document.createElement('label')
        setAttributes(
            afterTimeLabelElement,
            {
                'for': 'after-time',
                'class': 'mt-3'
            }
        )
        afterTimeLabelElement.innerHTML = 'Show reports after this time (e.g. 2022-12-22 14:59:07)'
        var afterTimeInputElement = document.createElement('input')
        setAttributes(
            afterTimeInputElement,
            {
                'class': 'form-control',
                'type': 'text',
                'id': 'after-time',
                'name': 'after_time',
                'placeholder': 'YYYY-MM-DD HH:MM:SS',
            }
        )
        afterTimeDivElement.appendChild(afterTimeLabelElement)
        afterTimeDivElement.appendChild(afterTimeInputElement)
        formElement.appendChild(afterTimeDivElement)

        var beforeTimeDivElement = document.createElement('div')
        setAttributes(
            beforeTimeDivElement,
            {'class': 'form-group'}
        )
        var beforeTimeLabelElement = document.createElement('label')
        setAttributes(
            beforeTimeLabelElement,
            {
                'for': 'before-time',
                'class': 'mt-3'
            }
        )
        beforeTimeLabelElement.innerHTML = 'Show reports before this time (e.g. 2022-12-22 14:59:07)'
        var beforeTimeInputElement = document.createElement('input')
        setAttributes(
            beforeTimeInputElement,
            {
                'class': 'form-control',
                'type': 'text',
                'id': 'before-time',
                'name': 'before_time',
                'placeholder': 'YYYY-MM-DD HH:MM:SS',
            }
        )
        beforeTimeDivElement.appendChild(beforeTimeLabelElement)
        beforeTimeDivElement.appendChild(beforeTimeInputElement)
        formElement.appendChild(beforeTimeDivElement)

        var fromDivElement = document.createElement('div')
        setAttributes(
            fromDivElement,
            {'class': 'form-group'}
        )
        var fromLabelElement = document.createElement('label')
        setAttributes(
            fromLabelElement,
            {
                'for': 'from-time',
                'class': 'mt-3'
            }
        )
        fromLabelElement.innerHTML = 'Show reports from this IP:PORT address. * is allowed to wildcard check. (e.g. 192.168.*)'
        var fromInputElement = document.createElement('input')
        setAttributes(
            fromInputElement,
            {
                'class': 'form-control',
                'type': 'text',
                'id': 'from',
                'name': 'from',
                'placeholder': '127.0.0.1:*',
            }
        )
        fromDivElement.appendChild(fromLabelElement)
        fromDivElement.appendChild(fromInputElement)
        formElement.appendChild(fromDivElement)

        var limitDivElement = document.createElement('div')
        setAttributes(
            limitDivElement,
            {'class': 'form-group'}
        )
        var limitLabelElement = document.createElement('label')
        setAttributes(
            limitLabelElement,
            {
                'for': 'limit',
                'class': 'mt-3'
            }
        )
        limitLabelElement.innerHTML = 'Maximum number of reports to fetch.'
        var limitInputElement = document.createElement('input')
        setAttributes(
            limitInputElement,
            {
                'class': 'form-control',
                'type': 'number',
                'id': 'limit',
                'name': 'limit',
                'placeholder': '20',
                'value': 20,
                'min': 1,
            }
        )
        limitDivElement.appendChild(limitLabelElement)
        limitDivElement.appendChild(limitInputElement)
        formElement.appendChild(limitDivElement)

        var contextDivElement = document.createElement('div')
        setAttributes(
            contextDivElement,
            {'class': 'form-group'}
        )
        var contextLabelElement = document.createElement('label')
        setAttributes(
            contextLabelElement,
            {
                'for': 'context',
                'class': 'mt-3 text-break'
            }
        )
        contextLabelElement.innerHTML = 'Context of report. Was it for running an operation or fetching state?'
        var contextSelectElement = document.createElement('select')
        setAttributes(
            contextSelectElement,
            {
                'class': 'form-control',
                'name': 'context',
            }
        )
        var contextOptionEverythingElement = document.createElement('option')
        setAttributes(
            contextOptionEverythingElement,
            {
                'value': '',
                'selected': 'selected',
            }
        )
        contextOptionEverythingElement.innerHTML = 'Everything'
        contextSelectElement.appendChild(contextOptionEverythingElement)
        var contextOptionOperationElement = document.createElement('option')
        setAttributes(
            contextOptionOperationElement,
            {'value': 'run'}
        )
        contextOptionOperationElement.innerHTML = 'Run'
        contextSelectElement.appendChild(contextOptionOperationElement)
        var contextOptionFetchStateElement = document.createElement('option')
        setAttributes(
            contextOptionFetchStateElement,
            {'value': 'state'}
        )
        contextOptionFetchStateElement.innerHTML = 'State'
        contextSelectElement.appendChild(contextOptionFetchStateElement)
        contextDivElement.appendChild(contextLabelElement)
        contextDivElement.appendChild(contextSelectElement)

        formElement.appendChild(contextDivElement)

        var formButtonElement = document.createElement('button')
        setAttributes(
            formButtonElement,
            {
                'class': 'btn btn-sm btn-primary btn-block mt-3 justify-content-center fw-bold',
                'type': 'submit'
            }
        )
        formButtonElement.innerHTML = 'Search'
        formElement.appendChild(formButtonElement)
        commandElement.appendChild(formElement)
        async function submitHandler(event) {
            event.preventDefault()
            var inputs = new FormData(event.target);
            const before_time = inputs.get('before_time')
            const after_time = inputs.get('after_time')
            const context = inputs.get('context')
            const from = inputs.get('from')
            var limit = inputs.get('limit')
            if (limit !== null) {
                limit = parseInt(limit)
            }
            updateResultBeforeRequest()
            const reportsResult = await new Api(ApiOpts).report(before_time, after_time, context, from, limit)
            updateResultAfterRequest(reportsResult)
            if (reportsResult.status === 401) {
                changeLogoutToLogin()
            }
        }
        formElement.addEventListener('submit', submitHandler)
    }
    reportLiElement.appendChild(reportAElement)
    ulElement.appendChild(reportLiElement)

    element.appendChild(ulElement)
}

async function addCommandClickEventListener(commandName, command, element) {
    element.onclick = async function() {
        var commandElement = document.getElementById('command')
        commandElement.innerHTML = ''
        var commandResultElement = document.getElementById('command-result')
        commandResultElement.innerHTML = ''
        await drawCommand(commandName, command, commandElement)
    }
}

async function drawCommand(commandName, command, element) {
    await console.log('Drawing command', commandName)
    var commandInfo = command.info;

    var commandHeaderElement = document.createElement('h1');
    setAttributes(
        commandHeaderElement,
        {
            'id': 'command-header',
            'class': 'h1 my-4 text-capitalize'
        }
    )
    commandHeaderElement.innerHTML = commandName
    if ('version' in commandInfo) {
        var smallElement = document.createElement('small')
        setAttributes(
            smallElement,
            {'class': 'text-muted text-lowercase'}
        )
        smallElement.innerHTML = ' (v' + commandInfo.version + ')'
        commandHeaderElement.appendChild(smallElement)
    }
    element.appendChild(commandHeaderElement)

    if (commandInfo.description != commandName) {
        var commandDescriptionElement = makeOptionDescription(commandInfo.description)
        element.appendChild(commandDescriptionElement)
    }

    if (command.info.support_state) {
        var commandStateHeaderElement = document.createElement('h3')
        setAttributes(
            commandStateHeaderElement,
            {
                'id': 'command-state-header',
                'class': 'h3 my-4 text-capitalize text-start'
            }
        )
        commandStateHeaderElement.innerHTML = 'Current State'
        element.appendChild(commandStateHeaderElement)
        var commandStateDivElement = document.createElement('div')
        setAttributes(
            commandStateDivElement,
            {'class': '', 'id': 'command-state'}
        )
        var waitingElement = document.createElement('p')
        setAttributes(
            waitingElement,
            {'class': 'text-center', 'id': 'command-state-text'}
        )
        commandStateDivElement.appendChild(waitingElement)
        element.appendChild(commandStateDivElement)
        getAndDrawCommandState(command)
    }

    var optionDefinitions = {};
    if ('options' in commandInfo) {
        optionDefinitions = commandInfo.options
    };
    element.appendChild(await makeCommandOptionsInputs(optionDefinitions, command))

}

async function getAndDrawCommandState(command) {
    await beforeGetCommandState(command)
    const runResult = await new Api(ApiOpts).state(command.http_path.replace('run', 'state'))
    afterGetCommandState(command, runResult)
    if (runResult.status === 401) {
        changeLogoutToLogin()
    }
}

async function beforeGetCommandState(command) {
    var waitingElement = document.createElement('p')
    setAttributes(
        waitingElement,
        {'class': 'text-center', 'id': 'command-state-text'}
    )
    waitingElement.innerHTML = 'Waiting for state...'.italics()
    document.getElementById('command-state-text').replaceWith(waitingElement)
}

async function afterGetCommandState(command, runResult) {
    var result = runResult.result
    const resultText = prettifyResponse(result, 0)
    var resultTextElement = document.createElement('p')
    setAttributes(
        resultTextElement,
        {
            'class': 'p-1 text-start text-break',
            'id': 'command-state-text'
        }
    )
    if (runResult.status === 200) {
        resultTextElement.innerHTML = resultText
    } else {
        resultTextElement.innerHTML = 'Error: ' + resultText
    }
    document.getElementById('command-state-text').replaceWith(resultTextElement)
}

async function makeCommandOptionsInputs(options, command) {
    var commandOptionsElement = document.createElement('div')
    setAttributes(
        commandOptionsElement,
         {'id': 'command-options', 'class': 'pt-3'}
    )
    var commandOptionFormElement = document.createElement('form')
    setAttributes(
        commandOptionFormElement,
        {
            'action': command.http_path,
            'method': 'POST',
            'name': 'options-form',
            'id': 'options-form'
        }
    )
    for (var optionName in options) {
        var definition = options[optionName];
        var typeName = definition.value_type;
        if (typeof typeName !== 'string') {
            typeName = Object.keys(definition.value_type)[0];
        };
        var typeElementList = [];
        switch (typeName) {
            case 'enum':
                typeElementList = await makeInputEnum(optionName, definition);
                break;
            case 'string':
                typeElementList = await makeInputString(optionName, definition);
                break;
            case 'integer':
                typeElementList = await makeInputInteger(optionName, definition);
                break;
            case 'float':
                typeElementList = await makeInputFloat(optionName, definition);
                break;
            case 'boolean':
                typeElementList = await makeInputBoolean(optionName, definition);
                break;
            case 'any':
                typeElementList = await makeInputString(optionName, definition);
            default:
                await console.log('Unknown type name ', typeName, ' in definition ', definition);
        };
        if (typeElementList.length === 0) {
            continue;
        };
        for (var i = 0; i < typeElementList.length; i++) {
            commandOptionFormElement.appendChild(typeElementList[i]);
        };
    };
    var submitDivElement = document.createElement('div')
    setAttributes(
        submitDivElement,
        {'class': 'my-3 justify-content-center'}
    )
    var submitElement = document.createElement('button')
    setAttributes(
        submitElement,
        {
            'type': 'submit',
            'id': 'run-button',
            'class': 'btn btn-sm btn-primary btn-block px-3'
        }
    )
    submitElement.innerHTML = 'RUN'
    submitDivElement.appendChild(submitElement)
    commandOptionFormElement.appendChild(submitDivElement);
    commandOptionFormElement.addEventListener(
        'submit',
        async function(event) {
            event.preventDefault();
            var inputOptions = new FormData(event.target);
            var requestOptions = {};
            for (var pair of inputOptions.entries()) {
                if (pair[1] === '') {
                    await console.log('skip empty string of key', pair[0]);
                    continue;
                };
                var definition = options[pair[0]];
                var typeName = definition.value_type;
                await console.log('Got type name', typeName, 'for key', pair[0])
                var value = pair[1];
                switch (typeName) {
                    case 'integer':
                        value = parseInt(value);
                        break;
                    case 'float':
                        value = parseFloat(value);
                    case 'boolean':
                        value = JSON.parse(value);
                    default:
                        break;
                };
                if (value !== pair[1]) {
                    await console.log('value', pair[1], 'is changed to', value);
                };
                requestOptions[pair[0]] = value;
            };
            updateResultBeforeRequest()
            const runResult = await new Api(ApiOpts).run(command.http_path, requestOptions)
            if (runResult.status !== 401 && runResult.status !== 0 && command.info.support_state) {
                getAndDrawCommandState(command)
            }
            updateResultAfterRequest(runResult)
            if (runResult.status === 401) {
                changeLogoutToLogin()
            }
            document.location = '#command-result'
        }
    );
    commandOptionsElement.appendChild(commandOptionFormElement);
    return commandOptionsElement;
}

function updateResultBeforeRequest() {
    var waitingElement = document.createElement('p')
    setAttributes(
        waitingElement,
        {'class': 'text-center'}
    )
    waitingElement.innerHTML = 'Waiting for response...'.italics()
    var resultElement = document.getElementById('command-result')
    resultElement.innerHTML = ''
    resultElement.appendChild(waitingElement)
}

function updateResultAfterRequest(runResult) {
    var resultHeaderElement = document.createElement('h3')
    setAttributes(
        resultHeaderElement,
        {'class': 'h3 my-0 text-capitalize text-center'}
    )
    resultHeaderElement.innerHTML = 'Result'

    var statusCodeTextElement = document.createElement('p')
    setAttributes(
        statusCodeTextElement,
        {
            'class': 'fst-italic text-center alert text-secondary',
            'id': 'status-code-text'
        }
    )
    var statusCodeTextSmallElement = document.createElement('small')
    statusCodeTextElement.innerHTML = 'Status: '
    var statusCodeElement =  document.createElement('span')
    var statusCodeClass = 'text-success'
    if (runResult.ok == false) {
        statusCodeClass = 'text-danger'
    }
    setAttributes(
        statusCodeElement,
        {'class': statusCodeClass}
    )
    statusCodeElement.innerHTML = runResult.status.toString().bold()
    if (runResult.message !== undefined) {
        statusCodeElement.innerHTML += ' ' + runResult.message.bold()
    }
    statusCodeTextSmallElement.appendChild(statusCodeElement)
    statusCodeTextElement.appendChild(statusCodeTextSmallElement)
    var statusCodeDivElement = document.createElement('div')
    setAttributes(
            statusCodeDivElement,
            {'class': 'my-0 py-0'}
        )
    statusCodeDivElement.appendChild(statusCodeTextElement)

    var result = runResult.result
    const resultText = prettifyResponse(result, 0)
    var resultTextElement = document.createElement('p')
    setAttributes(
        resultTextElement,
        {
            'class': 'p-1 text-start text-break',
            'id': 'command-result-text'
        }
    )
    resultTextElement.innerHTML = resultText
    var resultDivElement = document.createElement('div')
    setAttributes(
                resultDivElement,
                {'class': 'mb-5'}
            )
    resultDivElement.appendChild(resultTextElement)

    var resultElement = document.getElementById('command-result')
    resultElement.innerHTML = ''
    resultElement.appendChild(resultHeaderElement)
    if (runResult.status !== 0) {
        resultElement.appendChild(statusCodeDivElement)
    }
    resultElement.appendChild(resultDivElement)

    var runButtonElement = document.getElementById('run-button')
    if (runButtonElement !== null) {
        runButtonElement.innerHTML = 'Run Again'
    }
}

async function makeInputEnum(optionName, definition) {
    var required = definition.required;
    var defaultValue = null;
    if ('default_value' in definition) {
        defaultValue = definition.default_value;
    };
    var selectElement = document.createElement('select');
    setAttributes(
        selectElement,
        {
            'name': optionName,
            'class': 'mb-4'
        }
    )
    var valueList = definition.value_type.enum;
    for (var i = 0; i < valueList.length; i++) {
        var value = valueList[i];
        var enumValue = document.createElement('option');
        enumValue.setAttribute('value', value);
        if (value == defaultValue) {
            enumValue.setAttribute('selected', 'selected');
        };
        enumValue.innerHTML = value;
        selectElement.appendChild(enumValue);
    };
    if (defaultValue == null && required) {
        var enumValue = document.createElement('option');
        setAttributes(
            enumValue,
            {
                'value': 'none',
                'selected': 'selected',
                'disabled': 'disabled',
                'hidden': 'hidden'
            }
        )
        enumValue.innerHTML = 'Select an Option';
        selectElement.appendChild(enumValue);
    }

    var header = makeOptionHeader(optionName)

    var description = makeOptionDescription(definition.description)
    var selectElementDiv = document.createElement('div')
    selectElementDiv.appendChild(selectElement)
    return [header, description, selectElementDiv];
}

async function makeInputString(optionName, definition) {
    var stringElement = document.createElement('div');
    setAttributes(
        stringElement,
        {}
    )
    var required = definition.required;
    var defaultValue = null;
    if ('default_value' in definition) {
        defaultValue = definition.default_value;
    };
    var min_size = 0;
    var max_size = null;
    if ('size' in definition) {
        if ('min' in definition.size) {
            if (definition.size.min !== null) {
                min_size = definition.size.min;
            };
        };
        if ('max' in definition.size) {
            if (definition.size.max !== null) {
                max_size = definition.size.max;
            };
        };
    }

    var header = makeOptionHeader(optionName)
    var description = makeOptionDescription(definition.description)
    var textArea = document.createElement('textarea');
    textArea.setAttribute('rows', '5')
    textArea.setAttribute('name', optionName);
    if (defaultValue != null) {
        textArea.innerHTML = defaultValue;
    };
    if (required) {
        textArea.setAttribute('required', 'required');
    };
    stringElement.appendChild(header);
    stringElement.appendChild(description);
    var textAreaDiv = document.createElement('div')
    textAreaDiv.appendChild(textArea)
    stringElement.appendChild(textAreaDiv);
    return [stringElement];
}

async function makeInputInteger(optionName, definition) {
    var StringElement = document.createElement('div');
    StringElement.setAttribute('class', 'command-option-integer');
    var required = definition.required;
    var defaultValue = null;
    if ('default_value' in definition) {
        defaultValue = definition.default_value;
    };

    var header = makeOptionHeader(optionName)
    var description = makeOptionDescription(definition.description)
    var textArea = document.createElement('input');
    textArea.setAttribute('name', optionName);
    textArea.setAttribute('type', 'number');
    if ('size' in definition) {
        if ('min' in definition.size) {
            if (definition.size.min !== null) {
                textArea.setAttribute('min', definition.size.min);
            };
        };
        if ('max' in definition.size) {
            if (definition.size.max !== null) {
                textArea.setAttribute('max', definition.size.max);
            };
        };
    }
    if (defaultValue != null) {
        textArea.setAttribute('value', defaultValue);
    };
    if (required) {
        textArea.setAttribute('required', 'required');
    };
    StringElement.appendChild(header);
    StringElement.appendChild(description);
    StringElement.appendChild(textArea);
    return [header, description, textArea];
}

async function makeInputFloat(optionName, definition) {
    var StringElement = document.createElement('div');
    StringElement.setAttribute('class', 'command-option-float');
    var required = definition.required;
    var defaultValue = null;
    if ('default_value' in definition) {
        defaultValue = definition.default_value;
    };

    var header = makeOptionHeader(optionName)
    var description = makeOptionDescription(definition.description)
    var textArea = document.createElement('input');
    textArea.setAttribute('name', optionName);
    textArea.setAttribute('type', 'number');
    textArea.setAttribute('step', '0.000000001');
    if ('size' in definition) {
        if ('min' in definition.size) {
            if (definition.size.min !== null) {
                textArea.setAttribute('min', definition.size.min);
            };
        };
        if ('max' in definition.size) {
            if (definition.size.max !== null) {
                textArea.setAttribute('max', definition.size.max);
            };
        };
    }
    if (defaultValue != null) {
        textArea.setAttribute('value', defaultValue);
    };
    if (required) {
        textArea.setAttribute('required', 'required');
    };
    StringElement.appendChild(header);
    StringElement.appendChild(description);
    StringElement.appendChild(textArea);
    return [header, description, textArea];
}

async function makeInputBoolean(optionName, definition) {
    var booleanElement = document.createElement('div');
    setAttributes(
        booleanElement,
        {}
    );
    var required = definition.required;
    var defaultValue = null;
    if ('default_value' in definition) {
        defaultValue = definition.default_value;
    };

    var header = makeOptionHeader(optionName)
    var description = makeOptionDescription(definition.description)
    var textArea = document.createElement('input');
    setAttributes(
        textArea,
        {
            'name': optionName,
            'type': 'checkbox',
            'value': 'true',
            'style': 'width:auto;'
        }
    )
    if (defaultValue != null) {
        textArea.checked = defaultValue;
    };
    var flagText = document.createElement('span');
    flagText.innerHTML = '  ' + optionName.bold();
    var spanDiv = document.createElement('div');
    setAttributes(
        spanDiv,
        {
            'class': 'mb-4 text-start'
        }
    )
    spanDiv.appendChild(textArea)
    spanDiv.appendChild(flagText)
    booleanElement.appendChild(header);
    booleanElement.appendChild(description);
    booleanElement.appendChild(spanDiv)

    return [booleanElement];
}

function makeOptionDescription(text) {
    var description = document.createElement('p')
    setAttributes(
        description,
        {'class': 'text-start text-break'}
    )
    description.innerHTML = text
    return description
}

function makeOptionHeader(name) {
    var header = document.createElement('h3')
    setAttributes(
        header,
        {'class': 'h3 mt-2 mb-1 text-capitalize text-start'}
    )
    header.innerHTML = name.replaceAll('-', ' ').replaceAll('_', ' ')
    return header
}

function prettifyResponse(x, indent) {
    var result = doPrettifyResponse(x, indent)
    return result.trim()
}

function doPrettifyResponse(x, indent) {
    var result = '';
    switch (typeof x) {
        case 'string':
            result = x;
            break;
        case 'number':
            result = x.toString();
            break;
        case 'object':
            if (Array.isArray(x)) {
                if (x.length === 0) {
                    result += '[]';
                } else {
                    result += '\r\n'
                    var listIndent = indent
                    if (listIndent !== 0) {
                        listIndent += 1
                    }
                    for (var i = 0; i < x.length; i++) {
                        result += doPrettifyResponse(x[i], listIndent);
                    };
                }
            } else if (x === null) {
                result = 'None';
            } else {
                for (var key in x) {
                    const value = x[key];
                    result += key + ':' + doPrettifyResponse(value, indent + 1);
                };
            };
            break;
        case 'boolean':
            if (x) {
                result = 'True';
            } else {
                result = 'False';
            };
            break;
        default:
            result += x;
    };
    if (indent > 0) {
        result = '    '.repeat(indent) + result;
    }
    result += '\r\n'
    return result;
}

function changeLogoutToLogin() {
    var logoutElement = document.getElementById('settings-logout')
    logoutElement.innerHTML = 'Login'
    logoutElement.onclick = async function() {
        document.location = 'login.html'
    }
    var commandResultElement = document.getElementById('command-result')
    if (commandResultElement !== null) {
        var helpElement = document.createElement('a')
        setAttributes(
            helpElement,
            {
                'class': 'mb-5 btn btn-sm btn-primary fw-bold',
                'href': 'login.html'
            }
        )
        helpElement.innerHTML = 'Login Again'
        commandResultElement.appendChild(helpElement)
    }
}

async function main() {
    const authResult = await new Api(ApiOpts).testAuth(true)
    if (authResult === false) {
        document.location = 'index.html'
        return
    }
    const configuration = await new Api(ApiOpts).configuration(true)
    if (configuration !== false) {
        if ('service_name' in configuration) {
            document.title = configuration.service_name
        } else {
            console.log('Could not found `service_name` in server configuration')
        }
        if ('footer' in configuration) {
            document.getElementById('footer').innerHTML = configuration.footer
        } else {
            console.log('Could not found `footer` in server configuration')
        }
    }
    drawNavbar()
}
window.main = main
