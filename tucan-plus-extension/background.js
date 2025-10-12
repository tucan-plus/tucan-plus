import "./fix-session-id-in-url.js"
import "./context-menu.js"
import { handleOpenInTucan, getCurrentTab } from "./open-in-tucan.js"
import { asyncClosure } from "./utils.js";
import { customUiRules } from "./custom-ui.js";
import { recoverTabs } from "./recover-tabs.js";

console.log("background script")

chrome.runtime.onMessage.addListener((message, sender) => {
    asyncClosure(async () => {
        console.log("onMessage", message, sender)

        await chrome.notifications.create({
            type: "basic",
            iconUrl: chrome.runtime.getURL("/icon-512.png"),
            title: "TUCaN Plus extension message",
            message: String(message),
        });

        if (!sender.tab?.id || !sender.tab.url) {
            console.log("no tab id or url")
            return;
        }

        if (message === "open-in-tucan-page") {
            const id = await chrome.cookies.get({
                url: "https://www.tucan.tu-darmstadt.de/scripts",
                name: "id",
            })

            await chrome.tabs.update(sender.tab.id, {
                url: await handleOpenInTucan(id?.value, sender.tab.id, sender.tab.url)
            })
            return;
        }
    })
})

if (chrome.commands) {
    chrome.commands.onCommand.addListener((command) => {
        asyncClosure(async () => {
            const id = await chrome.cookies.get({
                url: "https://www.tucan.tu-darmstadt.de/scripts",
                name: "id",
            })

            let tab = await getCurrentTab()

            if (!tab.id || !tab.url) {
                console.log("no tab id or url")
                return;
            }

            if (command === "open-in-tucan-page") {
                await chrome.tabs.update(tab.id, {
                    url: await handleOpenInTucan(id?.value, tab.id, tab.url)
                })
            }
        })
    });
}

chrome.runtime.onInstalled.addListener(() => {
    asyncClosure(async () => {
        console.log("oninstalled")
        let { mobileDesign, customUi } = await chrome.storage.sync.get(
            { mobileDesign: false, customUi: true },
        );

        if (customUi) {
            await enableCustomUi()
        } else {
            await disableCustomUi()
        }

        if (mobileDesign) {
            await enableMobileDesign()
        } else {
            await disableMobileDesign()
        }

        await recoverTabs();
    });
});

chrome.storage.sync.onChanged.addListener((changes) => {
    asyncClosure(async () => {
        for (let [key, { newValue }] of Object.entries(changes)) {
            if (key === "mobileDesign") {
                if (newValue) {
                    await enableMobileDesign()
                } else {
                    await disableMobileDesign()
                }
            }
            if (key === "customUi") {
                if (newValue) {
                    await enableCustomUi()
                } else {
                    await disableCustomUi()
                }
            }
        }
    });
});
/*
chrome.declarativeNetRequest.onRuleMatchedDebug.addListener(
    event => {
        console.log(event)
    }
)
*/

// https://groups.google.com/a/chromium.org/g/chromium-extensions/c/v3yrOjZIDJc

async function enableCustomUi() {
    await chrome.declarativeNetRequest.updateDynamicRules({
        removeRuleIds: customUiRules.map(r => r.id),
        addRules: customUiRules,
    })
}

async function disableCustomUi() {
    await chrome.declarativeNetRequest.updateDynamicRules({
        removeRuleIds: customUiRules.map(r => r.id)
    })
}

async function enableMobileDesign() {
    await chrome.scripting.registerContentScripts(
        [{
            id: "mobile",
            "matches": [
                "https://www.tucan.tu-darmstadt.de/*"
            ],
            "css": [
                "mobile.css"
            ],
            "js": [
                "mobile.js"
            ],
            "runAt": "document_end"
        }]
    )
}

async function disableMobileDesign() {
    const registeredContentScripts = await chrome.scripting.getRegisteredContentScripts()
    if (registeredContentScripts.find(s => s.id === "mobile")) {
        await chrome.scripting.unregisterContentScripts({
            ids: ["mobile"]
        })
    }
}

chrome.omnibox.onInputStarted.addListener(function () {
    chrome.omnibox.setDefaultSuggestion({
        description: "TUCaN Plus"
    });
});

chrome.omnibox.onInputChanged.addListener(() => {
    chrome.omnibox.setDefaultSuggestion({
        description: "TUCaN Plus"
    });

    /** @type {chrome.omnibox.SuggestResult[]} */
    let results = [{
        content: "https://www.tucan.tu-darmstadt.de",
        description: "TUCaN Plus"
    }]
    return results
})

chrome.omnibox.onInputEntered.addListener(() => {
    asyncClosure(async () => {
        await chrome.tabs.update({ url: "https://www.tucan.tu-darmstadt.de" })
    })
})

export { }