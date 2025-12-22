import {ApiOpts, Api} from './api.js'
import {setAttributes} from './utils.js'
import {setConfiguration} from './configuration.js'

async function drawNavbar() {
    var loadingElement = document.getElementById('sidebar-loading')
    var navigationBarElement = document.getElementById('navigation-bar')
    
    // Show loading
    if (loadingElement) {
        loadingElement.style.display = 'block'
    }
    if (navigationBarElement) {
        navigationBarElement.innerHTML = ''
    }
    
    const commandsResult = await new Api(ApiOpts).commands(true)
    
    // Hide loading
    if (loadingElement) {
        loadingElement.style.display = 'none'
    }
    
    if (commandsResult === false) {
        if (navigationBarElement) {
            navigationBarElement.innerHTML = '<div class="p-3 text-white text-center">Failed to load commands</div>'
        }
        // Still show settings even if commands failed
        appendSettings(navigationBarElement)
        return false
    }
    
    const commands = commandsResult.commands
    const commandCount = Object.keys(commands).length
    await console.log('Got', commandCount, 'command(s)')

    if (navigationBarElement) {
        navigationBarElement.innerHTML = ''
        
        if (commandCount === 0) {
            // No commands - only show settings
            var guideElement = document.getElementById('guide')
            if (guideElement) {
                guideElement.textContent = 'No commands available. Please configure commands in the server.'
            }
            // Only show settings
            appendSettings(navigationBarElement)
            return true
        }
        
        // Create tree structure for commands
        var treeList = document.createElement('ul')
        setAttributes(treeList, {'class': 'sidebar-tree list-unstyled mb-0'})
        await drawTreeNav(commands, treeList, 0)
        navigationBarElement.appendChild(treeList)
        
        // Add settings at the bottom
        appendSettings(navigationBarElement)
        
        // Update guide message
        var guideElement = document.getElementById('guide')
        if (guideElement) {
            guideElement.textContent = 'Select a menu item to start'
        }
    }
    
    return true
}

async function drawTreeNav(commands, parentElement, depth) {
    for (const key in commands) {
        const command = commands[key];
        const keyName = key.replaceAll('-', ' ').replaceAll('_', ' ')
        
        var listItem = document.createElement('li')
        setAttributes(listItem, {'class': 'sidebar-item'})
        
        if (command.is_directory) {
            // It's a folder - create collapsible tree node
            var folderId = 'folder-' + depth + '-' + key.replace(/[^a-zA-Z0-9]/g, '-')
            
            var folderToggle = document.createElement('button')
            setAttributes(folderToggle, {
                'class': 'sidebar-folder-toggle w-100 text-start d-flex align-items-center',
                'type': 'button',
                'data-bs-toggle': 'collapse',
                'data-bs-target': '#' + folderId,
                'aria-expanded': 'false',
                'aria-controls': folderId
            })
            
            var folderIcon = document.createElement('span')
            setAttributes(folderIcon, {'class': 'sidebar-icon me-2'})
            // Folder icon (plus icon for folders)
            folderIcon.innerHTML = '<svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16"><path d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z"/></svg>'
            
            var folderName = document.createElement('span')
            folderName.className = 'text-capitalize flex-grow-1'
            folderName.textContent = keyName
            
            var folderChevron = document.createElement('span')
            setAttributes(folderChevron, {'class': 'sidebar-chevron ms-auto'})
            folderChevron.innerHTML = '<svg width="12" height="12" fill="currentColor" viewBox="0 0 16 16"><path fill-rule="evenodd" d="M4.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L10.293 8 4.646 2.354a.5.5 0 0 1 0-.708z"/></svg>'
            
            folderToggle.appendChild(folderIcon)
            folderToggle.appendChild(folderName)
            folderToggle.appendChild(folderChevron)
            
            var folderChildren = document.createElement('ul')
            setAttributes(folderChildren, {
                'class': 'sidebar-tree collapse list-unstyled',
                'id': folderId,
                'style': 'padding-left: calc(var(--sidebar-indent-base) + var(--sidebar-indent-unit) * ' + depth + ');'
            })
            
            await drawTreeNav(command.commands, folderChildren, depth + 1)
            
            listItem.appendChild(folderToggle)
            listItem.appendChild(folderChildren)
        } else {
            // It's a command - create clickable link
            var commandLink = document.createElement('a')
            setAttributes(commandLink, {
                'class': 'sidebar-command w-100 text-start d-flex align-items-center text-capitalize',
                'href': '#'
            })
            
            var commandIcon = document.createElement('span')
            setAttributes(commandIcon, {'class': 'sidebar-icon me-2'})
            // Command icon (play button for commands)
            commandIcon.innerHTML = '<svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16"><path d="m11.596 8.697-6.363 3.692c-.54.313-1.233-.066-1.233-.697V4.308c0-.63.692-1.01 1.233-.696l6.363 3.692a.802.802 0 0 1 0 1.393z"/></svg>'
            
            var commandName = document.createElement('span')
            commandName.textContent = keyName
            
            commandLink.appendChild(commandIcon)
            commandLink.appendChild(commandName)
            
            await addCommandClickEventListener(keyName, command, commandLink)
            
            listItem.appendChild(commandLink)
        }
        
        parentElement.appendChild(listItem)
    }
}

function appendSettings(element) {
    var settingsDiv = document.createElement('div')
    setAttributes(settingsDiv, {'class': 'sidebar-settings border-top border-secondary mt-auto'})
    
    var settingsId = 'settings-folder'
    var settingsToggle = document.createElement('button')
    setAttributes(settingsToggle, {
        'class': 'sidebar-folder-toggle w-100 text-start d-flex align-items-center',
        'type': 'button',
        'data-bs-toggle': 'collapse',
        'data-bs-target': '#' + settingsId,
        'aria-expanded': 'false',
        'aria-controls': settingsId
    })
    
    var settingsIcon = document.createElement('span')
    setAttributes(settingsIcon, {'class': 'sidebar-icon me-2'})
    settingsIcon.innerHTML = '<svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16"><path d="M9.405 1.05c-.413-1.4-2.397-1.4-2.81 0l-.1.34a1.464 1.464 0 0 1-2.105.872l-.31-.17c-1.283-.698-2.686.705-1.987 1.987l.169.311c.446.82.023 1.841-.872 2.105l-.34.1c-1.4.413-1.4 2.397 0 2.81l.34.1a1.464 1.464 0 0 1 .872 2.105l-.17.31c-.698 1.283.705 2.686 1.987 1.987l.311-.169a1.464 1.464 0 0 1 2.105.872l.1.34c.413 1.4 2.397 1.4 2.81 0l.1-.34a1.464 1.464 0 0 1 2.105-.872l.31.17c1.283.698 2.686-.705 1.987-1.987l-.169-.311a1.464 1.464 0 0 1 .872-2.105l.34-.1c1.4-.413 1.4-2.397 0-2.81l-.34-.1a1.464 1.464 0 0 1-.872-2.105l.17-.31c.698-1.283-.705-2.686-1.987-1.987l-.311.169a1.464 1.464 0 0 1-2.105-.872l-.1-.34zM8 10.93a2.929 2.929 0 1 1 0-5.86 2.929 2.929 0 0 1 0 5.86z"/></svg>'
    
    var settingsName = document.createElement('span')
    settingsName.className = 'text-capitalize flex-grow-1'
    settingsName.textContent = 'Settings'
    
    var settingsChevron = document.createElement('span')
    setAttributes(settingsChevron, {'class': 'sidebar-chevron ms-auto'})
    settingsChevron.innerHTML = '<svg width="12" height="12" fill="currentColor" viewBox="0 0 16 16"><path fill-rule="evenodd" d="M4.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L10.293 8 4.646 2.354a.5.5 0 0 1 0-.708z"/></svg>'
    
    settingsToggle.appendChild(settingsIcon)
    settingsToggle.appendChild(settingsName)
    settingsToggle.appendChild(settingsChevron)
    
    var settingsList = document.createElement('ul')
    setAttributes(settingsList, {
        'class': 'sidebar-tree collapse list-unstyled',
        'id': settingsId,
        'style': 'padding-left: var(--sidebar-indent-base);'
    })
    // Logout
    var logoutItem = document.createElement('li')
    var logoutLink = document.createElement('a')
    setAttributes(logoutLink, {
        'class': 'sidebar-command w-100 text-start d-flex align-items-center text-capitalize',
        'href': '#',
        'id': 'settings-logout'
    })
    var logoutIcon = document.createElement('span')
    setAttributes(logoutIcon, {'class': 'sidebar-icon me-2'})
    logoutIcon.innerHTML = '<svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16"><path fill-rule="evenodd" d="M10 12.5a.5.5 0 0 1-.5.5h-8a.5.5 0 0 1-.5-.5v-9a.5.5 0 0 1 .5-.5h8a.5.5 0 0 1 .5.5v2a.5.5 0 0 0 1 0v-2A1.5 1.5 0 0 0 9.5 2h-8A1.5 1.5 0 0 0 0 3.5v9A1.5 1.5 0 0 0 1.5 14h8a1.5 1.5 0 0 0 1.5-1.5v-2a.5.5 0 0 0-1 0z"/><path fill-rule="evenodd" d="M15.854 8.354a.5.5 0 0 0 0-.708l-3-3a.5.5 0 0 0-.708.708L14.293 7.5H5.5a.5.5 0 0 0 0 1h8.793l-2.147 2.146a.5.5 0 0 0 .708.708l3-3z"/></svg>'
    var logoutName = document.createElement('span')
    logoutName.textContent = 'Logout'
    logoutLink.appendChild(logoutIcon)
    logoutLink.appendChild(logoutName)
    logoutLink.onclick = async function() {
        closeSidebar()
        document.cookie = 'token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:01 GMT;'
        document.location = 'index.html'
    }
    logoutItem.appendChild(logoutLink)
    settingsList.appendChild(logoutItem)

    // Set New Password
    var setPasswordItem = document.createElement('li')
    var setPasswordLink = document.createElement('a')
    setAttributes(setPasswordLink, {
        'class': 'sidebar-command w-100 text-start d-flex align-items-center text-capitalize',
        'href': '#'
    })
    var setPasswordIcon = document.createElement('span')
    setAttributes(setPasswordIcon, {'class': 'sidebar-icon me-2'})
    setPasswordIcon.innerHTML = '<svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16"><path d="M8 1a2 2 0 0 1 2 2v4H6V3a2 2 0 0 1 2-2zm3 6V3a3 3 0 0 0-6 0v4a2 2 0 0 0-2 2v5a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2zM5 8h6a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V9a1 1 0 0 1 1-1z"/></svg>'
    var setPasswordName = document.createElement('span')
    setPasswordName.textContent = 'Set New Password'
    setPasswordLink.appendChild(setPasswordIcon)
    setPasswordLink.appendChild(setPasswordName)
    setPasswordLink.onclick = async function() {
        closeSidebar()
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
    setPasswordItem.appendChild(setPasswordLink)
    settingsList.appendChild(setPasswordItem)
    
    settingsDiv.appendChild(settingsToggle)
    settingsDiv.appendChild(settingsList)
    element.appendChild(settingsDiv)
}

async function addCommandClickEventListener(commandName, command, element) {
    element.onclick = async function() {
        // Close sidebar after selecting a command
        closeSidebar()
        
        var commandElement = document.getElementById('command')
        commandElement.innerHTML = ''
        var commandResultElement = document.getElementById('command-result')
        commandResultElement.innerHTML = ''
        await drawCommand(commandName, command, commandElement)
    }
}

async function toggleSidebar() {
    var sidebar = document.getElementById('sidebar')
    var backdrop = document.getElementById('sidebar-backdrop')
    var toggleButton = document.getElementById('sidebar-toggle')
    
    if (!sidebar) return
    
    if (sidebar.classList.contains('show')) {
        // Close sidebar
        closeSidebar()
    } else {
        // Open sidebar - always fetch commands fresh
        sidebar.classList.add('show')
        if (backdrop && window.innerWidth < 768) {
            backdrop.classList.add('show')
        }
        if (window.innerWidth < 768) {
            document.body.style.overflow = 'hidden'
        }
        
        // Hide hamburger button when sidebar opens
        if (toggleButton) {
            toggleButton.style.display = 'none'
        }
        
        // Always fetch commands when opening
        await drawNavbar()
    }
}

function closeSidebar() {
    var sidebar = document.getElementById('sidebar')
    var backdrop = document.getElementById('sidebar-backdrop')
    var toggleButton = document.getElementById('sidebar-toggle')
    
    if (sidebar) {
        sidebar.classList.remove('show')
        if (backdrop) {
            backdrop.classList.remove('show')
        }
        document.body.style.overflow = ''
    }
    
    // Show hamburger button when sidebar closes
    if (toggleButton) {
        toggleButton.style.display = 'block'
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
    // Show waiting message for state
    var waitingElement = document.createElement('p')
    setAttributes(
        waitingElement,
        {'class': 'text-center', 'id': 'command-state-text'}
    )
    waitingElement.innerHTML = 'Waiting for state...'.italics()
    document.getElementById('command-state-text').replaceWith(waitingElement)
    
    const runResult = await new Api(ApiOpts).state(command.http_path.replace('run', 'state'))
    afterGetCommandState(command, runResult)
    if (runResult.status === 401) {
        changeLogoutToLogin()
    }
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
                await getAndDrawCommandState(command)
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
    
    // Create form group container
    var formGroup = document.createElement('div')
    setAttributes(formGroup, {'class': 'mb-4'})
    
    var header = makeOptionHeader(optionName)
    var description = makeOptionDescription(definition.description)
    
    var selectElement = document.createElement('select');
    setAttributes(
        selectElement,
        {
            'name': optionName,
            'class': 'form-select form-select-lg'
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

    formGroup.appendChild(header)
    formGroup.appendChild(description)
    formGroup.appendChild(selectElement)
    return [formGroup];
}

async function makeInputString(optionName, definition) {
    // Create form group container
    var formGroup = document.createElement('div');
    setAttributes(formGroup, {'class': 'mb-4'})
    
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
    setAttributes(textArea, {
        'rows': '5',
        'name': optionName,
        'class': 'form-control'
    })
    if (defaultValue != null) {
        textArea.innerHTML = defaultValue;
    };
    if (required) {
        textArea.setAttribute('required', 'required');
    };
    if (min_size > 0) {
        textArea.setAttribute('minlength', min_size);
    }
    if (max_size !== null) {
        textArea.setAttribute('maxlength', max_size);
    }
    
    formGroup.appendChild(header);
    formGroup.appendChild(description);
    formGroup.appendChild(textArea);
    return [formGroup];
}

async function makeInputInteger(optionName, definition) {
    // Create form group container
    var formGroup = document.createElement('div');
    setAttributes(formGroup, {'class': 'mb-4'})
    
    var required = definition.required;
    var defaultValue = null;
    if ('default_value' in definition) {
        defaultValue = definition.default_value;
    };

    var header = makeOptionHeader(optionName)
    var description = makeOptionDescription(definition.description)
    
    var inputElement = document.createElement('input');
    setAttributes(inputElement, {
        'name': optionName,
        'type': 'number',
        'class': 'form-control form-control-lg'
    })
    if ('size' in definition) {
        if ('min' in definition.size) {
            if (definition.size.min !== null) {
                inputElement.setAttribute('min', definition.size.min);
            };
        };
        if ('max' in definition.size) {
            if (definition.size.max !== null) {
                inputElement.setAttribute('max', definition.size.max);
            };
        };
    }
    if (defaultValue != null) {
        inputElement.setAttribute('value', defaultValue);
    };
    if (required) {
        inputElement.setAttribute('required', 'required');
    };
    
    formGroup.appendChild(header);
    formGroup.appendChild(description);
    formGroup.appendChild(inputElement);
    return [formGroup];
}

async function makeInputFloat(optionName, definition) {
    // Create form group container
    var formGroup = document.createElement('div');
    setAttributes(formGroup, {'class': 'mb-4'})
    
    var required = definition.required;
    var defaultValue = null;
    if ('default_value' in definition) {
        defaultValue = definition.default_value;
    };

    var header = makeOptionHeader(optionName)
    var description = makeOptionDescription(definition.description)
    
    var inputElement = document.createElement('input');
    setAttributes(inputElement, {
        'name': optionName,
        'type': 'number',
        'step': '0.000000001',
        'class': 'form-control form-control-lg'
    })
    if ('size' in definition) {
        if ('min' in definition.size) {
            if (definition.size.min !== null) {
                inputElement.setAttribute('min', definition.size.min);
            };
        };
        if ('max' in definition.size) {
            if (definition.size.max !== null) {
                inputElement.setAttribute('max', definition.size.max);
            };
        };
    }
    if (defaultValue != null) {
        inputElement.setAttribute('value', defaultValue);
    };
    if (required) {
        inputElement.setAttribute('required', 'required');
    };
    
    formGroup.appendChild(header);
    formGroup.appendChild(description);
    formGroup.appendChild(inputElement);
    return [formGroup];
}

async function makeInputBoolean(optionName, definition) {
    // Create form group container
    var formGroup = document.createElement('div');
    setAttributes(formGroup, {'class': 'mb-4'})
    
    var required = definition.required;
    var defaultValue = null;
    if ('default_value' in definition) {
        defaultValue = definition.default_value;
    };

    var header = makeOptionHeader(optionName)
    var description = makeOptionDescription(definition.description)
    
    var checkboxDiv = document.createElement('div');
    setAttributes(checkboxDiv, {'class': 'form-check'})
    
    var checkboxInput = document.createElement('input');
    setAttributes(
        checkboxInput,
        {
            'name': optionName,
            'type': 'checkbox',
            'value': 'true',
            'class': 'form-check-input',
            'id': 'checkbox-' + optionName
        }
    )
    if (defaultValue != null) {
        checkboxInput.checked = defaultValue;
    };
    
    var checkboxLabel = document.createElement('label');
    setAttributes(checkboxLabel, {
        'class': 'form-check-label',
        'for': 'checkbox-' + optionName
    })
    checkboxLabel.innerHTML = optionName.replaceAll('-', ' ').replaceAll('_', ' ')
    
    checkboxDiv.appendChild(checkboxInput)
    checkboxDiv.appendChild(checkboxLabel)
    
    formGroup.appendChild(header);
    formGroup.appendChild(description);
    formGroup.appendChild(checkboxDiv);

    return [formGroup];
}

function makeOptionDescription(text) {
    var description = document.createElement('div')
    setAttributes(
        description,
        {'class': 'form-text text-muted mb-3'}
    )
    description.innerHTML = text
    return description
}

function makeOptionHeader(name) {
    var header = document.createElement('label')
    setAttributes(
        header,
        {'class': 'form-label fw-semibold mb-2 text-capitalize'}
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
    if (logoutElement) {
        var nameSpan = logoutElement.querySelector('span:not(.sidebar-icon)')
        if (nameSpan) {
            nameSpan.textContent = 'Login'
        }
        logoutElement.onclick = async function() {
            closeSidebar()
            document.location = 'login.html'
        }
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
    setConfiguration({'footer': null})
    
    // Setup sidebar toggle button
    var toggleButton = document.getElementById('sidebar-toggle')
    var closeButton = document.getElementById('sidebar-close')
    var backdrop = document.getElementById('sidebar-backdrop')
    
    if (toggleButton) {
        toggleButton.addEventListener('click', toggleSidebar)
    }
    
    if (closeButton) {
        closeButton.addEventListener('click', closeSidebar)
    }
    
    if (backdrop) {
        backdrop.addEventListener('click', closeSidebar)
    }
    
    // Close sidebar when clicking on main content area
    var mainContent = document.querySelector('.main-content')
    if (mainContent) {
        mainContent.addEventListener('click', function(event) {
            // Only close if sidebar is open and we're not clicking inside sidebar
            var sidebar = document.getElementById('sidebar')
            if (sidebar && sidebar.classList.contains('show')) {
                var clickedInsideSidebar = sidebar.contains(event.target)
                if (!clickedInsideSidebar) {
                    closeSidebar()
                }
            }
        })
    }
    
    // Sidebar starts closed on both desktop and mobile
    // Commands will be loaded only when user clicks hamburger button
    closeSidebar()
    // Update guide message
    var guideElement = document.getElementById('guide')
    if (guideElement) {
        guideElement.textContent = 'Click the menu button to view available commands'
    }
}
window.main = main
