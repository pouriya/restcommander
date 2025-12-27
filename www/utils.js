
async function getUrlWithBasePath() {
    var basePath = ''
    const pathSegmentList = window.location.pathname.split('/')
    for (var offset = 0; offset < pathSegmentList.length; offset++) {
        const pathSegment = pathSegmentList[offset]
        await console.log(pathSegment)
        if (pathSegment.length === 0) {
            continue
        }
        if (pathSegment === 'static') {
            break
        }
        basePath = basePath + pathSegment + '/'
    }
    await console.log('Detected', basePath, 'as base path')
    return window.location.protocol + '//' + window.location.host + '/' + basePath
}

function setAttributes(element, attributes) {
    for(var key in attributes) {
        element.setAttribute(key, attributes[key]);
    }
}

export {getUrlWithBasePath, setAttributes}
