import { recoverTabs } from "./recover-tabs.js";
import { asyncClosure } from "./utils.js";

/** @type {HTMLElement} */ (document.querySelector('#go-to-options')).addEventListener('click', function () {
    asyncClosure(async () => {
        await chrome.runtime.openOptionsPage();
    })
});

const EXT_PAGE_INDEX_HTML = chrome.runtime.getURL('/public/index.html');

/** @type {HTMLElement} */ (document.querySelector('#open-custom-ui')).addEventListener('click', function () {
    asyncClosure(async () => {
        await chrome.tabs.create({
            url: EXT_PAGE_INDEX_HTML
        })
    });
})

// TODO maybe chrome.runtime.onUpdateAvailable

/**
 * 
 * @param {number} tabId 
 * @param {string} url 
 * @returns {Promise<void>}
 */
function updateTab(tabId, url) {
    return new Promise((resolve, reject) => {
        chrome.tabs.onUpdated.addListener(function listener(updatedTabId, info) {
            if (info.status === 'complete' && updatedTabId === tabId) {
                chrome.tabs.onUpdated.removeListener(listener);
                resolve();
            }
        });
        chrome.tabs.update(tabId, { url }).catch(reject)
    })
}

document.querySelector("#update-extension")?.addEventListener('click', function () {
    asyncClosure(async () => {
        console.log("update extension")

        // Chrome will close all extension tabs including blob urls, see https://issues.chromium.org/issues/41189391
        // The following is a hack and should mostly be used for development

        // https://stackoverflow.com/questions/68422688/chrome-extension-declarativenetrequest-isnt-matching-rulecondition

        console.log("remove selfmade4u rule")
        await chrome.declarativeNetRequest.updateDynamicRules({
            // TODO centrally reference this id
            removeRuleIds: [4100], // TODO check that rules have no dupes
        });

        // https://issues.chromium.org/issues/40670457
        let tabs = await chrome.runtime.getContexts({
            contextTypes: [/** @type {chrome.runtime.ContextType.TAB} */("TAB")],
        });

        await Promise.all(tabs.map(async tab => {
            if (!tab.documentUrl) {
                return;
            }
            let url = new URL(tab.documentUrl);
            await updateTab(tab.tabId, "https://www.tucan.tu-darmstadt.de" + url.pathname + url.hash)
        }));

        await new Promise(r => setTimeout(r, 2000));

        console.log("calling reload")
        chrome.runtime.reload();
        console.log("reload called")
    })
});

/** @type {HTMLElement} */(document.querySelector('#grant-permission')).addEventListener('click', () => {
    asyncClosure(async () => {
        if (await chrome.permissions.request({
            origins: ['https://www.tucan.tu-darmstadt.de/']
        })) {
        /** @type {HTMLElement} */ (document.querySelector("#grant-permission-area")).style.display = "none";
        }
    })
});

/** @type {HTMLElement} */(document.querySelector("#extract-tucan-session")).addEventListener("click", () => {
    asyncClosure(async () => {
        console.log("copying");
        const idCookie = await chrome.cookies.get({
            url: "https://www.tucan.tu-darmstadt.de/scripts",
            name: "id",
        })
        const cnscCookie = await chrome.cookies.get({
            url: "https://www.tucan.tu-darmstadt.de/scripts",
            name: "cnsc",
        })
        await navigator.clipboard.writeText(`SESSION_ID=${idCookie?.value}
SESSION_KEY=${cnscCookie?.value}
`)
        await chrome.notifications.create({
            type: "basic",
            iconUrl: chrome.runtime.getURL("/logo.png"),
            title: "Copied to clipboard",
            message: "Credentials copied to clipboard. Take care.",
        });
    })
});

if (!await chrome.permissions.contains({
    origins: ['https://www.tucan.tu-darmstadt.de/']
})) {
    console.log("no host permissions");
    /** @type {HTMLElement} */(document.querySelector("#grant-permission-area")).style.display = "block";
} else {
    console.log("have host permission");
}

export { }