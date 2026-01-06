import {ApiOpts, Api} from './api.js'
import {setAttributes} from './utils.js'
import {setConfiguration} from './configuration.js'
import {initTheme, toggleTheme} from './theme.js'

// Convert flat MCP tools list to tree structure for navigation
function toolsToTree(tools) {
    const root = { children: {} }
    
    for (const tool of tools) {
        const parts = tool.name.split('/')
        let current = root
        
        // Create directory nodes for path segments
        for (let i = 0; i < parts.length - 1; i++) {
            const part = parts[i]
            if (!current.children[part]) {
                current.children[part] = {
                    name: part,
                    isDirectory: true,
                    children: {}
                }
            }
            current = current.children[part]
        }
        
        // Create leaf tool node
        const leafName = parts[parts.length - 1]
        current.children[leafName] = {
            name: leafName,
            isDirectory: false,
            tool: tool  // Store the full MCP tool object
        }
    }
    
    return root
}

// Get display name from tool (use last path segment, cleaned up)
function getDisplayName(tool) {
    const name = tool.name.split('/').pop()
    return name.replaceAll('-', ' ').replaceAll('_', ' ')
}

async function drawNavbar() {
    const loadingElement = document.getElementById('sidebar-loading')
    const navigationBarElement = document.getElementById('navigation-bar')
    
    // Show loading
    if (loadingElement) {
        loadingElement.style.display = 'block'
    }
    if (navigationBarElement) {
        navigationBarElement.innerHTML = ''
    }
    
    const result = await new Api(ApiOpts).mcpToolsList()
    
    // Hide loading
    if (loadingElement) {
        loadingElement.style.display = 'none'
    }
    
    if (!result.ok) {
        if (navigationBarElement) {
            navigationBarElement.innerHTML = '<div class="p-3 text-white text-center">Failed to load tools</div>'
        }
        appendSettings(navigationBarElement)
        return false
    }
    
    const tools = result.result.tools || []
    const toolCount = tools.length
    console.log('Got', toolCount, 'tool(s)')

    if (navigationBarElement) {
        navigationBarElement.innerHTML = ''
        
        if (toolCount === 0) {
            const guideElement = document.getElementById('guide')
            if (guideElement) {
                guideElement.textContent = 'No tools available. Please configure scripts in the server.'
            }
            appendSettings(navigationBarElement)
            return true
        }
        
        // Convert flat tools list to tree
        const tree = toolsToTree(tools)
        
        // Create tree structure for navigation
        const treeList = document.createElement('ul')
        setAttributes(treeList, {'class': 'sidebar-tree list-unstyled mb-0'})
        drawTreeNav(tree.children, treeList, 0)
        navigationBarElement.appendChild(treeList)
        
        // Add settings at the bottom
        appendSettings(navigationBarElement)
        
        // Update guide message
        const guideElement = document.getElementById('guide')
        if (guideElement) {
            guideElement.textContent = 'Select a tool to start'
        }
    }
    
    return true
}

function drawTreeNav(nodes, parentElement, depth) {
    for (const key in nodes) {
        const node = nodes[key]
        const displayKey = key.replaceAll('-', ' ').replaceAll('_', ' ')
        
        const listItem = document.createElement('li')
        setAttributes(listItem, {'class': 'sidebar-item'})
        
        if (node.isDirectory) {
            // It's a folder - create collapsible tree node
            const folderId = 'folder-' + depth + '-' + key.replace(/[^a-zA-Z0-9]/g, '-')
            
            const folderToggle = document.createElement('button')
            setAttributes(folderToggle, {
                'class': 'sidebar-folder-toggle w-100 text-start d-flex align-items-center',
                'type': 'button',
                'data-bs-toggle': 'collapse',
                'data-bs-target': '#' + folderId,
                'aria-expanded': 'false',
                'aria-controls': folderId
            })
            
            const folderIcon = document.createElement('span')
            setAttributes(folderIcon, {'class': 'sidebar-icon me-2'})
            folderIcon.innerHTML = '<svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16"><path d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z"/></svg>'
            
            const folderName = document.createElement('span')
            folderName.className = 'text-capitalize flex-grow-1'
            folderName.textContent = displayKey
            
            const folderChevron = document.createElement('span')
            setAttributes(folderChevron, {'class': 'sidebar-chevron ms-auto'})
            folderChevron.innerHTML = '<svg width="12" height="12" fill="currentColor" viewBox="0 0 16 16"><path fill-rule="evenodd" d="M4.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L10.293 8 4.646 2.354a.5.5 0 0 1 0-.708z"/></svg>'
            
            folderToggle.appendChild(folderIcon)
            folderToggle.appendChild(folderName)
            folderToggle.appendChild(folderChevron)
            
            const folderChildren = document.createElement('ul')
            setAttributes(folderChildren, {
                'class': 'sidebar-tree collapse list-unstyled',
                'id': folderId,
                'style': 'padding-left: calc(var(--sidebar-indent-base) + var(--sidebar-indent-unit) * ' + depth + ');'
            })
            
            drawTreeNav(node.children, folderChildren, depth + 1)
            
            listItem.appendChild(folderToggle)
            listItem.appendChild(folderChildren)
        } else {
            // It's a tool - create clickable link
            const toolLink = document.createElement('a')
            setAttributes(toolLink, {
                'class': 'sidebar-command w-100 text-start d-flex align-items-center text-capitalize',
                'href': '#'
            })
            
            const toolIcon = document.createElement('span')
            setAttributes(toolIcon, {'class': 'sidebar-icon me-2'})
            toolIcon.innerHTML = '<svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16"><path d="m11.596 8.697-6.363 3.692c-.54.313-1.233-.066-1.233-.697V4.308c0-.63.692-1.01 1.233-.696l6.363 3.692a.802.802 0 0 1 0 1.393z"/></svg>'
            
            const toolName = document.createElement('span')
            toolName.textContent = getDisplayName(node.tool)
            
            toolLink.appendChild(toolIcon)
            toolLink.appendChild(toolName)
            
            addToolClickEventListener(node.tool, toolLink)
            
            listItem.appendChild(toolLink)
        }
        
        parentElement.appendChild(listItem)
    }
}

function appendSettings(element) {
    const settingsDiv = document.createElement('div')
    setAttributes(settingsDiv, {'class': 'sidebar-settings border-top border-secondary mt-auto'})
    
    const settingsId = 'settings-folder'
    const settingsToggle = document.createElement('button')
    setAttributes(settingsToggle, {
        'class': 'sidebar-folder-toggle w-100 text-start d-flex align-items-center',
        'type': 'button',
        'data-bs-toggle': 'collapse',
        'data-bs-target': '#' + settingsId,
        'aria-expanded': 'false',
        'aria-controls': settingsId
    })
    
    const settingsIcon = document.createElement('span')
    setAttributes(settingsIcon, {'class': 'sidebar-icon me-2'})
    settingsIcon.innerHTML = '<svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16"><path d="M9.405 1.05c-.413-1.4-2.397-1.4-2.81 0l-.1.34a1.464 1.464 0 0 1-2.105.872l-.31-.17c-1.283-.698-2.686.705-1.987 1.987l.169.311c.446.82.023 1.841-.872 2.105l-.34.1c-1.4.413-1.4 2.397 0 2.81l.34.1a1.464 1.464 0 0 1 .872 2.105l-.17.31c-.698 1.283.705 2.686 1.987 1.987l.311-.169a1.464 1.464 0 0 1 2.105.872l.1.34c.413 1.4 2.397 1.4 2.81 0l.1-.34a1.464 1.464 0 0 1 2.105-.872l.31.17c1.283.698 2.686-.705 1.987-1.987l-.169-.311a1.464 1.464 0 0 1 .872-2.105l.34-.1c1.4-.413 1.4-2.397 0-2.81l-.34-.1a1.464 1.464 0 0 1-.872-2.105l.17-.31c.698-1.283-.705-2.686-1.987-1.987l-.311.169a1.464 1.464 0 0 1-2.105-.872l-.1-.34zM8 10.93a2.929 2.929 0 1 1 0-5.86 2.929 2.929 0 0 1 0 5.86z"/></svg>'
    
    const settingsName = document.createElement('span')
    settingsName.className = 'text-capitalize flex-grow-1'
    settingsName.textContent = 'Settings'
    
    const settingsChevron = document.createElement('span')
    setAttributes(settingsChevron, {'class': 'sidebar-chevron ms-auto'})
    settingsChevron.innerHTML = '<svg width="12" height="12" fill="currentColor" viewBox="0 0 16 16"><path fill-rule="evenodd" d="M4.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L10.293 8 4.646 2.354a.5.5 0 0 1 0-.708z"/></svg>'
    
    settingsToggle.appendChild(settingsIcon)
    settingsToggle.appendChild(settingsName)
    settingsToggle.appendChild(settingsChevron)
    
    const settingsList = document.createElement('ul')
    setAttributes(settingsList, {
        'class': 'sidebar-tree collapse list-unstyled',
        'id': settingsId,
        'style': 'padding-left: var(--sidebar-indent-base);'
    })
    
    // Logout
    const logoutItem = document.createElement('li')
    const logoutLink = document.createElement('a')
    setAttributes(logoutLink, {
        'class': 'sidebar-command w-100 text-start d-flex align-items-center text-capitalize',
        'href': '#',
        'id': 'settings-logout'
    })
    const logoutIcon = document.createElement('span')
    setAttributes(logoutIcon, {'class': 'sidebar-icon me-2'})
    logoutIcon.innerHTML = '<svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16"><path fill-rule="evenodd" d="M10 12.5a.5.5 0 0 1-.5.5h-8a.5.5 0 0 1-.5-.5v-9a.5.5 0 0 1 .5-.5h8a.5.5 0 0 1 .5.5v2a.5.5 0 0 0 1 0v-2A1.5 1.5 0 0 0 9.5 2h-8A1.5 1.5 0 0 0 0 3.5v9A1.5 1.5 0 0 0 1.5 14h8a1.5 1.5 0 0 0 1.5-1.5v-2a.5.5 0 0 0-1 0z"/><path fill-rule="evenodd" d="M15.854 8.354a.5.5 0 0 0 0-.708l-3-3a.5.5 0 0 0-.708.708L14.293 7.5H5.5a.5.5 0 0 0 0 1h8.793l-2.147 2.146a.5.5 0 0 0 .708.708l3-3z"/></svg>'
    const logoutName = document.createElement('span')
    logoutName.textContent = 'Logout'
    logoutLink.appendChild(logoutIcon)
    logoutLink.appendChild(logoutName)
    logoutLink.onclick = function() {
        closeSidebar()
        document.cookie = 'mcpd_token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:01 GMT;'
        document.location = 'index.html'
    }
    logoutItem.appendChild(logoutLink)
    settingsList.appendChild(logoutItem)

    // Set New Password
    const setPasswordItem = document.createElement('li')
    const setPasswordLink = document.createElement('a')
    setAttributes(setPasswordLink, {
        'class': 'sidebar-command w-100 text-start d-flex align-items-center text-capitalize',
        'href': '#'
    })
    const setPasswordIcon = document.createElement('span')
    setAttributes(setPasswordIcon, {'class': 'sidebar-icon me-2'})
    setPasswordIcon.innerHTML = '<svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16"><path d="M8 1a2 2 0 0 1 2 2v4H6V3a2 2 0 0 1 2-2zm3 6V3a3 3 0 0 0-6 0v4a2 2 0 0 0-2 2v5a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2zM5 8h6a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V9a1 1 0 0 1 1-1z"/></svg>'
    const setPasswordName = document.createElement('span')
    setPasswordName.textContent = 'Set New Password'
    setPasswordLink.appendChild(setPasswordIcon)
    setPasswordLink.appendChild(setPasswordName)
    setPasswordLink.onclick = function() {
        closeSidebar()
        drawSetPasswordForm()
    }
    setPasswordItem.appendChild(setPasswordLink)
    settingsList.appendChild(setPasswordItem)
    
    settingsDiv.appendChild(settingsToggle)
    settingsDiv.appendChild(settingsList)
    element.appendChild(settingsDiv)
}

function drawSetPasswordForm() {
    const contentElement = document.getElementById('tool-content')
    contentElement.innerHTML = ''
    document.getElementById('tool-result').innerHTML = ''

    const headerElement = document.createElement('h1')
    setAttributes(headerElement, {'class': 'h3 my-4'})
    headerElement.textContent = 'Change Password'
    contentElement.appendChild(headerElement)

    const textElement = document.createElement('p')
    setAttributes(textElement, {'class': 'my-3 text-start text-break'})
    textElement.textContent = 'Set your new dashboard password.'
    contentElement.appendChild(textElement)

    const formElement = document.createElement('form')
    
    const oldPasswordDiv = document.createElement('div')
    setAttributes(oldPasswordDiv, {'class': 'mb-3'})
    const oldPasswordInput = document.createElement('input')
    setAttributes(oldPasswordInput, {
        'class': 'form-control',
        'type': 'password',
        'id': 'old-password',
        'name': 'old-password',
        'placeholder': 'Current Password*',
        'required': 'required'
    })
    oldPasswordDiv.appendChild(oldPasswordInput)
    formElement.appendChild(oldPasswordDiv)
    
    const passwordDiv = document.createElement('div')
    setAttributes(passwordDiv, {'class': 'mb-3'})
    const passwordInput = document.createElement('input')
    setAttributes(passwordInput, {
        'class': 'form-control',
        'type': 'password',
        'id': 'password',
        'name': 'password',
        'placeholder': 'New Password*',
        'required': 'required'
    })
    passwordDiv.appendChild(passwordInput)
    formElement.appendChild(passwordDiv)
    
    const submitButton = document.createElement('button')
    setAttributes(submitButton, {
        'class': 'btn btn-sm btn-primary btn-block mt-3 justify-content-center fw-bold',
        'type': 'submit'
    })
    submitButton.textContent = 'Apply'
    formElement.appendChild(submitButton)
    contentElement.appendChild(formElement)
    
    formElement.addEventListener('submit', async function(event) {
        event.preventDefault()
        const inputs = new FormData(event.target)
        const oldPassword = inputs.get('old-password')
        const password = inputs.get('password')
        updateResultBeforeRequest()
        const result = await new Api(ApiOpts).setPassword(password, oldPassword)
        if (result.ok === true) {
            result.result = 'Password Changed Successfully.'
        }
        updateResultAfterRequest(result)
        if (result.status === 401) {
            changeLogoutToLogin()
        }
    })
}

function addToolClickEventListener(tool, element) {
    element.onclick = function() {
        closeSidebar()
        
        const contentElement = document.getElementById('tool-content')
        contentElement.innerHTML = ''
        const resultElement = document.getElementById('tool-result')
        resultElement.innerHTML = ''
        drawTool(tool, contentElement)
    }
}

function drawTool(tool, element) {
    console.log('Drawing tool', tool.name)
    
    const displayName = getDisplayName(tool)
    
    // Header
    const headerElement = document.createElement('h1')
    setAttributes(headerElement, {'class': 'h1 my-4 text-capitalize'})
    headerElement.textContent = displayName
    element.appendChild(headerElement)

    // Description
    if (tool.description && tool.description !== displayName) {
        const descriptionElement = document.createElement('div')
        setAttributes(descriptionElement, {'class': 'form-text text-muted mb-3'})
        descriptionElement.textContent = tool.description
        element.appendChild(descriptionElement)
    }

    // Check for state support via resources
    checkAndDrawState(tool)

    // Build form from inputSchema
    element.appendChild(buildToolForm(tool))
}

async function checkAndDrawState(tool) {
    // Check if there's a resource for this tool's state
    const resourceUri = 'mcpd://' + tool.name + '/state'
    const result = await new Api(ApiOpts).mcpResourcesRead(resourceUri)
    
    if (result.ok) {
        drawToolState(tool, result)
    }
}

function drawToolState(tool, stateResult) {
    const contentElement = document.getElementById('tool-content')
    if (!contentElement) return
    
    // Insert state section after description, before form
    const existingState = document.getElementById('tool-state-container')
    if (existingState) {
        existingState.remove()
    }
    
    const stateContainer = document.createElement('div')
    stateContainer.id = 'tool-state-container'
    
    const stateHeader = document.createElement('h3')
    setAttributes(stateHeader, {'class': 'h3 my-4 text-capitalize text-start'})
    stateHeader.textContent = 'Current State'
    stateContainer.appendChild(stateHeader)
    
    const stateCard = document.createElement('div')
    setAttributes(stateCard, {'class': 'card border-info mb-4'})
    
    const cardHeader = document.createElement('div')
    setAttributes(cardHeader, {'class': 'card-header bg-info text-white'})
    cardHeader.innerHTML = '<strong>Current State</strong>'
    stateCard.appendChild(cardHeader)
    
    const cardBody = document.createElement('div')
    setAttributes(cardBody, {'class': 'card-body'})
    
    const resultText = document.createElement('pre')
    setAttributes(resultText, {
        'class': 'card-text text-start text-break mb-0',
        'style': 'white-space: pre-wrap; font-family: \'Inconsolata\', monospace; font-size: 0.875rem;'
    })
    resultText.innerHTML = prettifyResponse(stateResult.result, 0, !stateResult.ok)
    cardBody.appendChild(resultText)
    stateCard.appendChild(cardBody)
    stateContainer.appendChild(stateCard)
    
    // Insert before the form
    const form = document.getElementById('tool-form')
    if (form) {
        contentElement.insertBefore(stateContainer, form.parentElement)
    } else {
        contentElement.appendChild(stateContainer)
    }
}

async function refreshToolState(tool) {
    const resourceUri = 'mcpd://' + tool.name + '/state'
    const result = await new Api(ApiOpts).mcpResourcesRead(resourceUri)
    if (result.ok) {
        drawToolState(tool, result)
    }
}

function buildToolForm(tool) {
    const formContainer = document.createElement('div')
    setAttributes(formContainer, {'class': 'pt-3'})
    
    const formElement = document.createElement('form')
    setAttributes(formElement, {'id': 'tool-form', 'name': 'tool-form'})
    
    const schema = tool.inputSchema || {}
    const properties = schema.properties || {}
    const required = schema.required || []
    
    // Create inputs for each property
    for (const propName in properties) {
        const prop = properties[propName]
        const isRequired = required.includes(propName)
        const inputElements = buildSchemaInput(propName, prop, isRequired)
        for (const el of inputElements) {
            formElement.appendChild(el)
        }
    }
    
    // Submit button
    const submitDiv = document.createElement('div')
    setAttributes(submitDiv, {'class': 'my-3 justify-content-center'})
    const submitButton = document.createElement('button')
    setAttributes(submitButton, {
        'type': 'submit',
        'id': 'call-button',
        'class': 'btn btn-primary btn-lg w-100 px-4 fw-bold'
    })
    submitButton.textContent = 'CALL'
    submitDiv.appendChild(submitButton)
    formElement.appendChild(submitDiv)
    
    // Handle form submission
    formElement.addEventListener('submit', async function(event) {
        event.preventDefault()
        const formData = new FormData(event.target)
        const args = {}
        
        for (const [key, value] of formData.entries()) {
            if (value === '') continue
            
            const prop = properties[key]
            let typedValue = value
            
            // Convert value based on schema type
            if (prop) {
                switch (prop.type) {
                    case 'integer':
                        typedValue = parseInt(value)
                        break
                    case 'number':
                        typedValue = parseFloat(value)
                        break
                    case 'boolean':
                        typedValue = JSON.parse(value)
                        break
                }
            }
            args[key] = typedValue
        }
        
        updateResultBeforeRequest()
        const result = await new Api(ApiOpts).mcpToolsCall(tool.name, args)
        
        // Refresh state if successful
        if (result.status !== 401 && result.status !== 0) {
            await refreshToolState(tool)
        }
        
        updateResultAfterRequest(result)
        if (result.status === 401) {
            changeLogoutToLogin()
        }
        document.location = '#tool-result'
    })
    
    formContainer.appendChild(formElement)
    return formContainer
}

function buildSchemaInput(name, prop, isRequired) {
    const formGroup = document.createElement('div')
    setAttributes(formGroup, {'class': 'mb-4'})
    
    // Label
    const label = document.createElement('label')
    setAttributes(label, {'class': 'form-label fw-semibold mb-2 text-capitalize'})
    label.textContent = name.replaceAll('-', ' ').replaceAll('_', ' ')
    formGroup.appendChild(label)
    
    // Description
    if (prop.description) {
        const desc = document.createElement('div')
        setAttributes(desc, {'class': 'form-text text-muted mb-3'})
        desc.textContent = prop.description
        formGroup.appendChild(desc)
    }
    
    // Input based on type
    if (prop.enum) {
        // Enum -> select
        const select = document.createElement('select')
        setAttributes(select, {'name': name, 'class': 'form-select form-select-lg'})
        
        // Add placeholder option if required and no default
        if (isRequired && prop.default === undefined) {
            const placeholder = document.createElement('option')
            setAttributes(placeholder, {
                'value': '',
                'selected': 'selected',
                'disabled': 'disabled',
                'hidden': 'hidden'
            })
            placeholder.textContent = 'Select an Option'
            select.appendChild(placeholder)
        }
        
        for (const val of prop.enum) {
            const option = document.createElement('option')
            option.value = val
            option.textContent = val
            if (val === prop.default) {
                option.selected = true
            }
            select.appendChild(option)
        }
        formGroup.appendChild(select)
    } else if (prop.type === 'boolean') {
        // Boolean -> checkbox
        const checkDiv = document.createElement('div')
        setAttributes(checkDiv, {'class': 'form-check'})
        
        const checkbox = document.createElement('input')
        setAttributes(checkbox, {
            'name': name,
            'type': 'checkbox',
            'value': 'true',
            'class': 'form-check-input',
            'id': 'checkbox-' + name
        })
        if (prop.default === true) {
            checkbox.checked = true
        }
        
        const checkLabel = document.createElement('label')
        setAttributes(checkLabel, {'class': 'form-check-label', 'for': 'checkbox-' + name})
        checkLabel.textContent = name.replaceAll('-', ' ').replaceAll('_', ' ')
        
        checkDiv.appendChild(checkbox)
        checkDiv.appendChild(checkLabel)
        formGroup.appendChild(checkDiv)
    } else if (prop.type === 'integer' || prop.type === 'number') {
        // Number -> number input
        const input = document.createElement('input')
        const attrs = {
            'name': name,
            'type': 'number',
            'class': 'form-control form-control-lg'
        }
        if (prop.type === 'number') {
            attrs.step = '0.000000001'
        }
        if (prop.minimum !== undefined) {
            attrs.min = prop.minimum
        }
        if (prop.maximum !== undefined) {
            attrs.max = prop.maximum
        }
        if (prop.default !== undefined) {
            attrs.value = prop.default
        }
        if (isRequired) {
            attrs.required = 'required'
        }
        setAttributes(input, attrs)
        formGroup.appendChild(input)
    } else {
        // String (default) -> textarea
        const textarea = document.createElement('textarea')
        const attrs = {
            'name': name,
            'rows': '5',
            'class': 'form-control'
        }
        if (prop.minLength !== undefined) {
            attrs.minlength = prop.minLength
        }
        if (prop.maxLength !== undefined) {
            attrs.maxlength = prop.maxLength
        }
        if (isRequired) {
            attrs.required = 'required'
        }
        setAttributes(textarea, attrs)
        if (prop.default !== undefined) {
            textarea.textContent = prop.default
        }
        formGroup.appendChild(textarea)
    }
    
    return [formGroup]
}

function updateResultBeforeRequest() {
    const resultElement = document.getElementById('tool-result')
    if (!resultElement) return
    
    const waitingCard = document.createElement('div')
    setAttributes(waitingCard, {'class': 'card border-secondary mb-4'})
    
    const cardBody = document.createElement('div')
    setAttributes(cardBody, {'class': 'card-body'})
    
    const waitingText = document.createElement('p')
    setAttributes(waitingText, {'class': 'card-text text-center mb-0'})
    waitingText.innerHTML = 'Waiting for response...'.italics()
    
    cardBody.appendChild(waitingText)
    waitingCard.appendChild(cardBody)
    resultElement.innerHTML = ''
    resultElement.appendChild(waitingCard)
}

function updateResultAfterRequest(result) {
    const resultElement = document.getElementById('tool-result')
    if (!resultElement) return
    
    let cardVariant = 'border-secondary'
    let cardHeaderClass = 'bg-secondary text-white'
    let cardTitle = 'Result'
    
    if (result.status === 401) {
        cardVariant = 'border-warning'
        cardHeaderClass = 'bg-warning text-dark'
        cardTitle = 'Authentication Required'
    } else if (result.status === 404) {
        cardVariant = 'border-secondary'
        cardHeaderClass = 'bg-secondary text-white'
        cardTitle = 'Not Found'
    } else if (result.ok === true) {
        cardVariant = 'border-success'
        cardHeaderClass = 'bg-success text-white'
        cardTitle = 'Tool Executed Successfully'
    } else if (result.ok === false) {
        cardVariant = 'border-danger'
        cardHeaderClass = 'bg-danger text-white'
        cardTitle = 'Tool Execution Failed'
    }
    
    const resultCard = document.createElement('div')
    setAttributes(resultCard, {'class': 'card ' + cardVariant + ' mb-4'})
    
    const cardHeader = document.createElement('div')
    setAttributes(cardHeader, {'class': 'card-header ' + cardHeaderClass})
    const cardTitleElement = document.createElement('strong')
    cardTitleElement.textContent = cardTitle
    cardHeader.appendChild(cardTitleElement)
    resultCard.appendChild(cardHeader)
    
    const cardBody = document.createElement('div')
    setAttributes(cardBody, {'class': 'card-body'})
    
    const resultText = document.createElement('pre')
    setAttributes(resultText, {
        'class': 'card-text text-start text-break mb-0',
        'style': 'white-space: pre-wrap; font-family: \'Inconsolata\', monospace; font-size: 0.875rem;'
    })
    resultText.innerHTML = prettifyResponse(result.result, 0, !result.ok)
    cardBody.appendChild(resultText)
    
    // Login button for 401 errors
    if (result.status === 401) {
        const loginDiv = document.createElement('div')
        setAttributes(loginDiv, {'class': 'mt-3 d-grid'})
        const loginButton = document.createElement('a')
        setAttributes(loginButton, {
            'class': 'btn btn-warning btn-lg fw-bold w-100',
            'href': 'login.html'
        })
        loginButton.textContent = 'Login Again'
        loginDiv.appendChild(loginButton)
        cardBody.appendChild(loginDiv)
    }
    
    // Status code for non-standard responses
    if (result.status !== 0 && result.status !== 200 && result.status !== 401 && result.status !== 404) {
        const statusDiv = document.createElement('div')
        setAttributes(statusDiv, {'class': 'mt-3 pt-3 border-top'})
        const statusText = document.createElement('small')
        setAttributes(statusText, {'class': 'text-muted', 'style': 'font-size: 0.6rem;'})
        statusText.textContent = 'HTTP Status: ' + result.status
        if (result.message !== undefined) {
            statusText.textContent += ' ' + result.message
        }
        statusDiv.appendChild(statusText)
        cardBody.appendChild(statusDiv)
    }
    
    resultCard.appendChild(cardBody)
    resultElement.innerHTML = ''
    resultElement.appendChild(resultCard)
    
    const callButton = document.getElementById('call-button')
    if (callButton) {
        callButton.textContent = 'CALL AGAIN'
    }
}

function escapeHtml(text) {
    const div = document.createElement('div')
    div.textContent = text
    return div.innerHTML
}

function prettifyResponse(x, indent, error = false) {
    return doPrettifyResponse(x, indent, error).trim()
}

function doPrettifyResponse(x, indent, error = false) {
    let result = ''
    const strClass = error ? 'json-error' : 'json-string'
    
    switch (typeof x) {
        case 'string':
            result = '<span class="' + strClass + '">' + escapeHtml(x) + '</span>'
            break
        case 'number':
            if (Number.isInteger(x)) {
                result = '<span class="json-number json-int">' + escapeHtml(x.toString()) + '</span>'
            } else {
                result = '<span class="json-number json-float">' + escapeHtml(x.toString()) + '</span>'
            }
            break
        case 'object':
            if (Array.isArray(x)) {
                if (x.length === 0) {
                    result += '[]'
                } else {
                    result += '\r\n'
                    const listIndent = indent !== 0 ? indent + 1 : indent
                    for (const item of x) {
                        result += doPrettifyResponse(item, listIndent, error)
                    }
                }
            } else if (x === null) {
                result = '<span class="json-null">None</span>'
            } else {
                for (const key in x) {
                    result += '<span class="json-key">' + escapeHtml(key) + '</span>:'
                    result += doPrettifyResponse(x[key], indent + 1, error)
                }
            }
            break
        case 'boolean':
            result = '<span class="json-boolean">' + (x ? 'True' : 'False') + '</span>'
            break
        default:
            result += escapeHtml(String(x))
    }
    
    if (indent > 0) {
        result = '    '.repeat(indent) + result
    }
    result += '\r\n'
    return result
}

function changeLogoutToLogin() {
    const logoutElement = document.getElementById('settings-logout')
    if (logoutElement) {
        const nameSpan = logoutElement.querySelector('span:not(.sidebar-icon)')
        if (nameSpan) {
            nameSpan.textContent = 'Login'
        }
        logoutElement.onclick = function() {
            closeSidebar()
            document.location = 'login.html'
        }
    }
}

async function toggleSidebar() {
    const sidebar = document.getElementById('sidebar')
    const backdrop = document.getElementById('sidebar-backdrop')
    const toggleButton = document.getElementById('sidebar-toggle')
    
    if (!sidebar) return
    
    if (sidebar.classList.contains('show')) {
        closeSidebar()
    } else {
        sidebar.classList.add('show')
        if (backdrop && window.innerWidth < 768) {
            backdrop.classList.add('show')
        }
        if (window.innerWidth < 768) {
            document.body.style.overflow = 'hidden'
        }
        
        if (toggleButton) {
            toggleButton.style.display = 'none'
        }
        
        await drawNavbar()
    }
}

function closeSidebar() {
    const sidebar = document.getElementById('sidebar')
    const backdrop = document.getElementById('sidebar-backdrop')
    const toggleButton = document.getElementById('sidebar-toggle')
    
    if (sidebar) {
        sidebar.classList.remove('show')
        if (backdrop) {
            backdrop.classList.remove('show')
        }
        document.body.style.overflow = ''
    }
    
    if (toggleButton) {
        toggleButton.style.display = 'block'
    }
}

async function main() {
    initTheme()
    
    const themeToggle = document.getElementById('theme-toggle')
    if (themeToggle) {
        themeToggle.addEventListener('click', toggleTheme)
    }
    
    const authResult = await new Api(ApiOpts).testAuth(true)
    if (authResult === false) {
        document.location = 'index.html'
        return
    }
    setConfiguration({'footer': null})
    
    const toggleButton = document.getElementById('sidebar-toggle')
    const closeButton = document.getElementById('sidebar-close')
    const backdrop = document.getElementById('sidebar-backdrop')
    
    if (toggleButton) {
        toggleButton.addEventListener('click', toggleSidebar)
    }
    
    if (closeButton) {
        closeButton.addEventListener('click', closeSidebar)
    }
    
    if (backdrop) {
        backdrop.addEventListener('click', closeSidebar)
    }
    
    const mainContent = document.querySelector('.main-content')
    if (mainContent) {
        mainContent.addEventListener('click', function(event) {
            const sidebar = document.getElementById('sidebar')
            if (sidebar && sidebar.classList.contains('show')) {
                if (!sidebar.contains(event.target)) {
                    closeSidebar()
                }
            }
        })
    }
    
    closeSidebar()
    const guideElement = document.getElementById('guide')
    if (guideElement) {
        guideElement.textContent = 'Click the menu button to view available tools'
    }
}

window.main = main

