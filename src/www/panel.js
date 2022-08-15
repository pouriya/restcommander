var commands = null;

async function testAuth(count) {
    await console.log('Attempt to call /api/testAuth');
    var authError = null;
    const initOptions = {headers: {'Accept': 'application/json', 'Content-Type': 'application/json'}};
    var request = await fetch('api/testAuth', initOptions)
            .catch(
                async function (error) {
                    await console.log('Could not make HTTP request: ' + error.response.data.message);
                    authError = error.response.data.message;
                }
            )
            .then(
                async function (localResponse) {
                    var localResponseJSON = await localResponse.json();
                    if (!localResponseJSON['ok']) {
                        authError = localResponseJSON.reason;
                    }
                }
            );
    if (count === 0 || authError === null) {
        return authError;
    };
    await new Promise(resolve => setTimeout(resolve, 500));
    return await testAuth(count - 1);
}

async function getCommands() {
    await console.log('Attempt to call /api/commands');
    var error = null;
    const initOptions = {headers: {'Accept': 'application/json', 'Content-Type': 'application/json'}};
    await fetch('api/commands', initOptions)
            .catch(
                async function (error) {
                    await console.log('Could not fetch commands: ' + error.response.data.message);
                    error = error.response.data.message;
                }
            )
            .then(
                async function (localResponse) {
                    var localResponseJSON = await localResponse.json();
                    if (localResponseJSON['ok']) {
                        commands = localResponseJSON.result;
                        await console.log('Got commands:', commands);
                    } else {
                        error = localResponseJSON.reason;
                    }
                }
            );
    return error
}

async function reloadConfig() {
    await console.log('Attempt to call /api/reload/config');
    var error = null;
    const initOptions = {headers: {'Accept': 'application/json', 'Content-Type': 'application/json'}};
    await fetch('api/reload/config', initOptions)
            .catch(
                async function (error) {
                    await console.log('Could not reload config: ' + error.response.data.message);
                    error = error.response.data.message;
                }
            )
            .then(
                async function (localResponse) {
                    var localResponseJSON = await localResponse.json();
                    if (!localResponseJSON['ok']) {
                        error = localResponseJSON.reason;
                    }
                }
            );
    return error
}

async function reloadCommands() {
    await console.log('Attempt to call /api/reload/commands');
    var error = null;
    const initOptions = {headers: {'Accept': 'application/json', 'Content-Type': 'application/json'}};
    await fetch('api/reload/commands', initOptions)
            .catch(
                async function (error) {
                    await console.log('Could not reload commands: ' + error.response.data.message);
                    error = error.response.data.message;
                }
            )
            .then(
                async function (localResponse) {
                    var localResponseJSON = await localResponse.json();
                    if (!localResponseJSON['ok']) {
                        error = localResponseJSON.reason;
                    }
                }
            );
    return error
}

async function setPassword(newPassword) {
    await console.log('Attempt to call /api/setPassword');
    var error = null;
    const initOptions = {
        method: 'POST',
        headers: {'Accept': 'application/json', 'Content-Type': 'application/json'},
        body: JSON.stringify({password: newPassword})
    };
    await fetch('api/setPassword', initOptions)
            .catch(
                async function (error) {
                    await console.log('Could not set new password: ' + error.response.data.message);
                    error = error.response.data.message;
                }
            )
            .then(
                async function (localResponse) {
                    var localResponseJSON = await localResponse.json();
                    if (!localResponseJSON['ok']) {
                        error = localResponseJSON.reason;
                    }
                }
            );
    return error
}

async function makeSettings() {

    var SettingsLiElement = document.createElement('li');
    var Settings = document.createElement('a');
    Settings.setAttribute('href', 'javascript:void(0);');
    Settings.innerHTML = 'Settings'.concat(' ▼').bold();
    SettingsLiElement.appendChild(Settings);
    var SettingsUlElement = document.createElement('ul');

    var reloadConfigLiElement = document.createElement('li');
    var reloadConfigElement = document.createElement('a');
    reloadConfigElement.innerHTML = 'Reload Config'.bold();
    reloadConfigElement.setAttribute('href', 'javascript:void(0);');
    reloadConfigElement.onclick = async function(){
        const reloadError = await reloadConfig();
        var alertText = 'Configuration reloaded successfully';
        if (reloadError !== null) {
            alertText = 'Could not reload configuration: ' + reloadError;
        };
        window.alert(alertText);
    };
    reloadConfigLiElement.appendChild(reloadConfigElement);

    var reloadCommandsLiElement = document.createElement('li');
    var reloadCommandsElement = document.createElement('a');
    reloadCommandsElement.innerHTML = 'Reload Commands'.bold();
    reloadCommandsElement.setAttribute('href', 'javascript:void(0);');
    reloadCommandsElement.onclick = async function(){
        const reloadError = await reloadCommands();
        var alertText = 'Commands reloaded successfully';
        if (reloadError !== null) {
            alertText = 'Could not reload commands: ' + reloadError;
        };
        await window.alert(alertText);
        window.location.reload();
    };
    reloadCommandsLiElement.appendChild(reloadCommandsElement);


    var changePasswordLiElement = document.createElement('li');
    var changePassword = document.createElement('a');
    changePassword.innerHTML = 'Change Password'.bold();
    changePassword.setAttribute('href', 'javascript:void(0);');
    changePassword.onclick = async function(){await drawChangePassword();};
    changePasswordLiElement.appendChild(changePassword);

    SettingsUlElement.appendChild(changePasswordLiElement);
    SettingsUlElement.appendChild(reloadCommandsLiElement);
    SettingsUlElement.appendChild(reloadConfigLiElement);
    SettingsLiElement.appendChild(SettingsUlElement);

    return SettingsLiElement;
}

async function drawChangePassword() {
    await console.log("Drawing change password page");
    var topDiv = document.createElement('div');
    topDiv.setAttribute('class', 'change-password');
    topDiv.setAttribute('id', 'change-password');

    var changePasswordHeader = document.createElement('h1');
    changePasswordHeader.setAttribute('class', 'change-password-header');
    changePasswordHeader.innerHTML = 'Change Admin password';

    var description = document.createElement('p');
    description.setAttribute('class', 'change-password-description');
    description.innerHTML = 'Set new password for administration.'

    var changePasswordForm = document.createElement('form');
    changePasswordForm.setAttribute('action', 'api/setPassword');
    changePasswordForm.setAttribute('method', 'post');
    changePasswordForm.setAttribute('name', 'change-password-form');
    changePasswordForm.setAttribute('id', 'change-password-form');
    var textArea = document.createElement('textarea');
    textArea.setAttribute('required', 'required');
    textArea.setAttribute('name', 'password');
    textArea.setAttribute('rows', 1);
    textArea.setAttribute('cols', 40);
    var submit = document.createElement('input');
    submit.setAttribute('class', 'input-change-password-button');
    submit.setAttribute('value', 'Change Password');
    submit.setAttribute('type', 'submit');
    changePasswordForm.appendChild(textArea);
    changePasswordForm.appendChild(submit);
    changePasswordForm.addEventListener(
        'submit',
        async function(event) {
            event.preventDefault();
            var inputOptions = new FormData(event.target);
            const setPasswordError = await setPassword(inputOptions.get('password'));
            var alertText = 'Administration Password has been changed';
            if (setPasswordError !== null) {
                alertText = 'Could not change password\n' + setPasswordError;
            };
            window.alert(alertText);
            if (setPasswordError === null) {
                window.location.reload();
            }
        }
    );


    topDiv.appendChild(changePasswordHeader);
    topDiv.appendChild(description);
    topDiv.appendChild(changePasswordForm);

    var section = document.createElement('section');
    section.setAttribute('id', 'current-command');
    section.appendChild(topDiv);
    document.getElementById('current-command').replaceWith(section);
}

async function runCommandAndDrawResponse(httpPath, requestBody) {
    await drawWaitingForResponse();
    await drawResponse(await runCommand(httpPath, requestBody));
}

async function drawWaitingForResponse() {
    var waiting = document.createElement('code');
    waiting.innerHTML = 'Waiting for response...'.italics();
    var header = document.createElement('h3');
    header.innerHTML = 'Response';
    var responseDiv = document.createElement('div');
    responseDiv.setAttribute('id', 'response');
    responseDiv.appendChild(header);
    responseDiv.appendChild(waiting);
    document.getElementById('response').replaceWith(responseDiv);
}

async function drawResponse(response) {
    var header = document.createElement('h3');
    header.innerHTML = 'Response';
    var responseDiv = document.createElement('div');
    responseDiv.setAttribute('id', 'response');
    var statusCode = document.createElement('code');
    if (response.code !== 200) {
        statusCode.style.color = "red";
    } else {
        statusCode.style.color = "green";
    };
    statusCode.innerHTML = ("Status Code: " + response.code.toString()).bold()
    responseDiv.appendChild(header);
    responseDiv.appendChild(statusCode);
    responseDiv.appendChild(document.createElement("br"));
    responseDiv.appendChild(document.createElement("br"));
    responseDiv.appendChild(response.element);
    document.getElementById('response').replaceWith(responseDiv);
}

async function runCommand(httpPath, requestBody) {
    const initOptions = {
        method: 'POST',
        headers: {'Accept': 'application/json', 'Content-Type': 'application/json'},
        body: JSON.stringify(requestBody)
    };
    var statusCode = 0;
    var response = null;
    var isError = false;
    var request = await fetch(httpPath, initOptions)
        .catch(
            async function (error) {
                response = 'Could not make HTTP request: ' + toString(error);
                isError = true;
            }
        )
        .then(
            async function (localResponse) {
                await console.log(httpPath, '->', localResponse);
                statusCode = localResponse.status;
                var localResponseJSON = await localResponse.json();
                if (localResponseJSON['ok']) {
                    if ('result' in localResponseJSON) {
                        response = localResponseJSON['result'];
                    } else {
                        response = 'Done';
                    };
                } else {
                    isError = true;
                    response = localResponseJSON['reason'];
                };
            }
        );
    await console.log('Response is set to', response);
    var prettyResponse = await prettifyResponse(response);
    await console.log('Prettified response is set to ', prettyResponse);
    var responseDiv = document.createElement('div');
    responseDiv.setAttribute('class', 'response-code');
    var responseElement = document.createElement('code');
    responseElement.innerHTML = prettyResponse.bold();
    if (isError) {
        responseElement.style.color = '#b15f1f'; // Shit color for error texts
    };
    responseDiv.appendChild(responseElement);
    return {'element': responseDiv, 'code': statusCode};
}

async function prettifyResponse(x, indent) {
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
                for (var i = 0; i < x.length; i++) {
                    result = result + await prettifyResponse(x[i], indent + 1) + '\r\n';
                };
            } else if (x === null) {
                result = 'Null';
            } else {
                for (var key in x) {
                    const value = x[key];
                    result = result + key + ': ' + await prettifyResponse(value, indent + 1) + '\r\n';
                };
            };
            break;
        case 'boolean':
            if (x) {
                result = 'True';
            } else {
                result = 'False'
            };
            break;
        default:
            result = result + x;
    };
    return result;
}

async function showGuide() {
    var guide = document.createElement('p');
    guide.style.marginTop = '100px';
    guide.innerHTML = 'Select a menu item to run a command.';
    var section = document.createElement('section');
    section.setAttribute('id', 'current-command');
    section.appendChild(guide);
    document.getElementById('current-command').replaceWith(section);
}

async function makeNavigationBar(commands, topElement, depth) {
    for (var key in commands) {
        const value = commands[key];
        const nameWordList = key.replaceAll('-', ' ').replaceAll('_', ' ').split(' ');
        var name = '';
        for (var i = 0; i < nameWordList.length; i++) {
            const capitalName = nameWordList[i][0].toUpperCase() + nameWordList[i].slice(1);
            name = name + capitalName + ' ';
        };
        const commandName = name.slice(0, -1);
        var liElement = document.createElement('li');
        if (value.is_directory) {
            var liElement = document.createElement('li');
            var directory = document.createElement('a');
            directory.setAttribute('href', 'javascript:void(0);');
            if (depth === 1) {
                directory.innerHTML = commandName.concat(' ▼').bold();
            } else {
                directory.innerHTML = commandName.concat(' ►').bold();
            };
            liElement.appendChild(directory);
            var ulElement = document.createElement('ul');
            liElement.appendChild(await makeNavigationBar(value.commands, ulElement, depth + 1));
        } else {
            var liElement = document.createElement('li');
            var command = document.createElement('a');
            command.innerHTML = commandName.bold();
            command.setAttribute('href', 'javascript:void(0);');
            command.onclick = async function(){await drawCommand(commandName, value);};
            liElement.appendChild(command);
        };
        topElement.appendChild(liElement);
    };
    if (depth === 1) {
        topElement.appendChild(await makeSettings());
    };
    return topElement;
}

async function makeTopNavigationBar() {
    var ulElement = document.createElement('ul');
    ulElement.setAttribute('id', 'navigation-bar');
    return await makeNavigationBar(commands.commands, ulElement, 1);
}

async function drawCommand(commandName, command) {
    await console.log("Drawing command", commandName, ": ", command);
    var topDiv = document.createElement('div');
    topDiv.setAttribute('class', 'command');
    topDiv.setAttribute('id', 'command');

    var commandHeader = document.createElement('h1');
    commandHeader.setAttribute('class', 'command-header');
    commandHeader.innerHTML = commandName;

    var commandInfo = command.info;
    var commandDescription = document.createElement('p');
    commandDescription.setAttribute('class', 'command-info-description');
    if (commandInfo.description != commandName) {
        commandDescription.innerHTML = commandInfo.description;
    };

    topDiv.appendChild(commandHeader);
    topDiv.appendChild(commandDescription);

    if ('version' in commandInfo) {
        commandHeader.innerHTML += ' (v' + commandInfo.version + ')';
    };

    if ('current_state' in commandInfo) {
        var commandCurrentStateHeader = document.createElement('h2');
        commandCurrentStateHeader.setAttribute('class', 'command-info-current-state-header');
        commandCurrentStateHeader.innerHTML = 'Current State';
        topDiv.appendChild(commandCurrentStateHeader);
        var commandCurrentState = document.createElement('p');
        commandCurrentState.setAttribute('class', 'command-info-current-state');
        commandCurrentState.innerHTML = commandInfo.current_state;
        topDiv.appendChild(commandCurrentState);
    };
    var optionDefinitionList = {};
    if ('options' in commandInfo) {
        optionDefinitionList = commandInfo.options;
    };
    if (Object.keys(optionDefinitionList).length > 0) {
        var optionsHeader = document.createElement('h2');
        optionsHeader.setAttribute('class', 'command-info-options-header');
        optionsHeader.innerHTML = 'Options';
        topDiv.appendChild(optionsHeader);
    };
    const optionInputList = await makeCommandOptionsInputs(optionDefinitionList, command.http_path);
    topDiv.appendChild(optionInputList);
    var response = document.createElement('div');
    response.setAttribute('id', 'response');
    topDiv.appendChild(response);
    var section = document.createElement('section');
    section.setAttribute('id', 'current-command');
    section.appendChild(topDiv);
    document.getElementById('current-command').replaceWith(section);
}

async function makeCommandOptionsInputs(options, httpPath) {
    var commandOptions = document.createElement('div');
    commandOptions.setAttribute('class', 'command-info-options');
    var commandOptionForm = document.createElement('form');
    commandOptionForm.setAttribute('action', httpPath);
    commandOptionForm.setAttribute('method', 'post');
    commandOptionForm.setAttribute('name', 'options-form');
    commandOptionForm.setAttribute('id', 'options-form');
    for (var optionName in options) {
        var definition = options[optionName];
        var typeName = definition.value_type;
        if (typeof typeName !== 'string') {
            typeName = Object.keys(definition.value_type)[0];
        };
        var typeElementList = [];
        switch (typeName) {
            case 'accepted_value_list':
                typeElementList = await makeInputAcceptedValueList(optionName, definition);
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
            case 'bool':
                typeElementList = await makeInputBool(optionName, definition);
                break;
            case 'any':
                typeElementList = await makeInputString(optionName, definition);
            default:
                await console.log('Unknown type name ', typeName, ' in definition ', definition);
        };
        await console.log(typeElementList);
        if (typeElementList.length === 0) {
            continue;
        };
        for (var i = 0; i < typeElementList.length; i++) {
            commandOptionForm.appendChild(typeElementList[i]);
        };
    };
    var submit = document.createElement('input');
    submit.setAttribute('class', 'input-run-button');
    submit.setAttribute('value', 'RUN');
    submit.setAttribute('type', 'submit');
    commandOptionForm.appendChild(submit);
    commandOptionForm.addEventListener(
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
                if (typeof typeName !== 'string') {
                    typeName = Object.keys(definition.value_type)[0];
                };
                var value = pair[1];
                switch (typeName) {
                    case 'integer':
                        value = parseInt(value);
                        break;
                    case 'float':
                        value = parseFloat(value);
                    case 'bool':
                        value = JSON.parse(value);
                    default:
                        break;
                };
                if (value !== pair[1]) {
                    await console.log('value', pair[1], 'is changed to', value);
                };
                requestOptions[pair[0]] = value;
            };
            var requestBody = {'options': requestOptions};
            await runCommandAndDrawResponse(httpPath, requestBody);
        }
    );
    commandOptions.appendChild(commandOptionForm);
    return commandOptions;
}

async function makeInputAcceptedValueList(optionName, definition) {
    var required = definition.required;
    var defaultValue = null;
    if ('default_value' in definition) {
        defaultValue = definition.default_value;
    };
    var selectElement = document.createElement('select');
    selectElement.setAttribute('name', optionName);
    var valueList = definition.value_type.accepted_value_list;
    for (var i = 0; i < valueList.length; i++) {
        var value = valueList[i];
        var acceptedValue = document.createElement('option');
        acceptedValue.setAttribute('value', value);
        if (value == defaultValue) {
            acceptedValue.setAttribute('selected', 'selected');
        };
        acceptedValue.innerHTML = value;
        selectElement.appendChild(acceptedValue);
    };
    if (defaultValue == null && required) {
        var acceptedValue = document.createElement('option');
        acceptedValue.setAttribute('value', 'none');
        acceptedValue.setAttribute('selected', 'selected');
        acceptedValue.setAttribute('disabled', 'disabled');
        acceptedValue.setAttribute('hidden', 'hidden');
        acceptedValue.innerHTML = 'Select an Option';
        selectElement.appendChild(acceptedValue);
    }

    var header = document.createElement('h3');
    header.innerHTML = optionName;

    var description = document.createElement('p');
    description.setAttribute('class', 'command-input-description');
    description.innerHTML = definition.description;

    return [header, description, selectElement];
}

async function makeInputString(optionName, definition) {
    var StringElement = document.createElement('div');
    StringElement.setAttribute('class', 'command-option-string');
    var required = definition.required;
    var defaultValue = null;
    if ('default_value' in definition) {
        defaultValue = definition.default_value;
    };
    var min_size = 0;
    if ('min_size' in definition.value_type.string) {
        if (definition.value_type.string.min_size != null) {
            min_size = definition.value_type.string.min_size;
        };
    };
    var max_size = null;
    if ('max_size' in definition.value_type.string) {
            max_size = definition.value_type.string.max_size;
    };

    var header = document.createElement('h3');
    header.innerHTML = optionName;
    var description = document.createElement('p');
    description.setAttribute('class', 'command-input-description');
    description.innerHTML = definition.description;
    var textArea = document.createElement('textarea');
    textArea.setAttribute('name', optionName);
    var rowCount = 1;
    var columnCount = 40;
    if (max_size != null && max_size > 100) {
        rowCount = 10;
        columnCount = 60;
    };
    textArea.setAttribute('rows', rowCount);
    textArea.setAttribute('cols', columnCount);
    if (defaultValue != null) {
        textArea.innerHTML = defaultValue;
    };
    if (required) {
        textArea.setAttribute('required', 'required');
    };
    StringElement.appendChild(header);
    StringElement.appendChild(description);
    StringElement.appendChild(textArea);
    return [header, description, textArea];
}

async function makeInputInteger(optionName, definition) {
    var StringElement = document.createElement('div');
    StringElement.setAttribute('class', 'command-option-integer');
    var required = definition.required;
    var defaultValue = null;
    if ('default_value' in definition) {
        defaultValue = definition.default_value;
    };

    var header = document.createElement('h3');
    header.innerHTML = optionName;
    var description = document.createElement('p');
    description.setAttribute('class', 'command-input-description');
    description.innerHTML = definition.description;
    var textArea = document.createElement('input');
    textArea.setAttribute('name', optionName);
    textArea.setAttribute('type', 'number');
    if ('min_size' in definition.value_type.integer) {
        if (definition.value_type.integer.min_size != null) {
            textArea.setAttribute('min', definition.value_type.integer.min_size);
        };
    };
    if ('max_size' in definition.value_type.integer) {
        if (definition.value_type.integer.max_size != null) {
            textArea.setAttribute('max', definition.value_type.integer.max_size);
        };
    };
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

    var header = document.createElement('h3');
    header.innerHTML = optionName;
    var description = document.createElement('p');
    description.setAttribute('class', 'command-input-description');
    description.innerHTML = definition.description;
    var textArea = document.createElement('input');
    textArea.setAttribute('name', optionName);
    textArea.setAttribute('type', 'number');
    if ('min_size' in definition.value_type.float) {
        if (definition.value_type.float.min_size != null) {
            textArea.setAttribute('min', definition.value_type.float.min_size);
        };
    };
    if ('max_size' in definition.value_type.float) {
        if (definition.value_type.float.max_size != null) {
            textArea.setAttribute('max', definition.value_type.float.max_size);
        };
    };
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

async function makeInputBool(optionName, definition) {
    var StringElement = document.createElement('div');
    StringElement.setAttribute('class', 'command-option-bool');
    var required = definition.required;
    var defaultValue = null;
    if ('default_value' in definition) {
        defaultValue = definition.default_value;
    };

    var header = document.createElement('h3');
    header.innerHTML = optionName;
    var description = document.createElement('p');
    description.setAttribute('class', 'command-input-description');
    description.innerHTML = definition.description;
    var textArea = document.createElement('input');
    textArea.setAttribute('name', optionName);
    textArea.setAttribute('type', 'checkbox');
    textArea.setAttribute('value', 'true');
    if (defaultValue != null) {
        textArea.checked = defaultValue;
    };
    var flagText = document.createElement('span');
    flagText.innerHTML = optionName.bold();
    StringElement.appendChild(header);
    StringElement.appendChild(description);
    StringElement.appendChild(textArea);

    return [header, description, textArea, flagText];
}

async function drawError(header, reason, footer) {
    var topDiv = document.createElement('div');
    topDiv.style.marginTop = '100px';
    footer = typeof footer !== 'undefined' ? footer : 'Please contact your administrator for more information.';
    var errorHeader = document.createElement('p');
    errorHeader.innerHTML = header + ': '
    var error = document.createElement('p');
    error.innerHTML = reason.bold();
    error.style.color = 'red';
    var errorFooter = document.createElement('p');
    errorFooter.innerHTML = footer.italics();
    topDiv.appendChild(errorHeader);
    topDiv.appendChild(error);
    topDiv.appendChild(errorFooter);
    document.body.appendChild(topDiv);
}

async function startPanel() {
    var authError = await testAuth(1);
    if (authError === null) {
        await console.log('Authenticated or Authentication is not required');
    } else {
        await drawError('Could not authenticate', authError, 'Remove credentials and reload the page.');
        return;
    };
    var getCommandsError = await getCommands();
    if (getCommandsError !== null) {
        await drawError('Could not fetch commands', getCommandsError);
        return;
    };
    document.body.insertBefore(await makeTopNavigationBar(), document.body.childNodes[0]);
    await showGuide();
}
