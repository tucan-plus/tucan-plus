const EXTENSION_DOMAIN = chrome.runtime.getURL('');

export async function recoverTabs() {
    console.log("enable selfmade4u rule")
    const rules = {
        removeRuleIds: [4100], // TODO check that rules have no dupes
        addRules: [{
            id: 4100,
            priority: 10,
            condition: {
                isUrlFilterCaseSensitive: true,
                resourceTypes: [
    /** @type {chrome.declarativeNetRequest.ResourceType} */ ("main_frame")
                ],
                urlFilter: `|https://www.tucan.tu-darmstadt.de/public/index.html*`
            },
            action: {
                type: /** @type {chrome.declarativeNetRequest.RuleActionType} */ ('redirect'),
                redirect: {
                    // I think this needs to statically be an allowed url
                    transform: {
                        scheme: EXTENSION_DOMAIN.split("://")[0],
                        host: EXTENSION_DOMAIN.slice(0, -1).replace("moz-extension://", "").replace("chrome-extension://", "")
                    }
                },
            },
        }],
    }
    await chrome.declarativeNetRequest.updateDynamicRules(rules);
    console.log(chrome.runtime.lastError)

    let tabs = await chrome.tabs.query({
        url: `https://www.tucan.tu-darmstadt.de/public/index.html*`
    })

    await Promise.all(tabs.map(async tab => {
        if (tab.id) {
            await chrome.tabs.reload(tab.id)
        }
    }))
}