import { asyncClosure } from "./utils.js";
import { handleOpenInTucan } from "./open-in-tucan.js"
import { customUiRules } from "./custom-ui.js";

if (chrome.contextMenus) {

    chrome.contextMenus.onClicked.addListener((info, tab) => {
        asyncClosure(async () => {
            const id = await chrome.cookies.get({
                url: "https://www.tucan.tu-darmstadt.de/scripts",
                name: "id",
            })

            let url = info.linkUrl ?? info.pageUrl
            let tabId = tab?.id

            if (!tabId || !url) {
                return;
            }

            if (info.menuItemId === "open-in-tucan" || info.menuItemId === "open-in-tucan-plus" || info.menuItemId === "open-in-tucan-page" || info.menuItemId === "open-in-tucan-plus-page") {
                let result = await handleOpenInTucan(id?.value, tabId, url);
                if (result !== undefined) {
                    await chrome.tabs.update(tabId, {
                        url: result
                    })
                }
                return;
            }

            if (info.menuItemId === "open-in-tucan-new-tab" || info.menuItemId === "open-in-tucan-plus-new-tab" || info.menuItemId === "open-in-tucan-page-new-tab" || info.menuItemId === "open-in-tucan-plus-page-new-tab") {
                let result = await handleOpenInTucan(id?.value, tabId, url);
                if (result !== undefined) {
                    let newTab = await chrome.tabs.create({
                        url: result
                    })

                    const newTabId = newTab.id;
                    if (newTabId === undefined) {
                        return;
                    }

                    /** @type {{ excludedTabIds: number[];  }} */
                    const settings = (await chrome.storage.session.get(
                        { excludedTabIds: [] },
                    ))
                    const updatedExludedTabIds = [newTabId, ...settings.excludedTabIds]
                    await chrome.storage.session.set({
                        excludedTabIds: updatedExludedTabIds
                    })
                    console.log(updatedExludedTabIds)
                    await chrome.declarativeNetRequest.updateSessionRules({
                        removeRuleIds: customUiRules.map(r => r.id),
                        addRules: customUiRules.map(rule => {
                            return {
                                ...rule,
                                condition: {
                                    excludedTabIds: updatedExludedTabIds,
                                    ...rule.condition
                                },
                            }
                        }),
                    })
                    console.log(chrome.runtime.lastError)
                    await chrome.declarativeNetRequest.updateDynamicRules({
                        removeRuleIds: customUiRules.map(r => r.id),
                    })
                    console.log(chrome.runtime.lastError)
                }
                return;
            }

            if (info.menuItemId === "shareable-link-page" || info.menuItemId === "shareable-link") {
                await chrome.notifications.create({
                    type: "basic",
                    iconUrl: chrome.runtime.getURL("/icon-512.png"),
                    title: "Sharing this URL is not supported",
                    message: "Unfortunately sharing this URL is not supported (yet). We welcome any contribution",
                });
                return;
            }

            await chrome.notifications.create({
                type: "basic",
                iconUrl: chrome.runtime.getURL("/icon-512.png"),
                title: "Context menu action not supported",
                message: "Unfortunately this context menu action is not supported yet. We welcome any contribution",
            });
        })
    })

    const EXTENSION_PAGE = chrome.runtime.getURL('/');

    chrome.runtime.onInstalled.addListener(() => {
        asyncClosure(async () => {

            await chrome.contextMenus.removeAll();

            chrome.contextMenus.create({
                id: "open-in-tucan",
                title: "Open link in TUCaN",
                contexts: ["link"],
                targetUrlPatterns: [`${EXTENSION_PAGE}*`]
            }, () => {
                console.log(chrome.runtime.lastError)
            })

            chrome.contextMenus.create({
                id: "open-in-tucan-new-tab",
                title: "Open link in TUCaN in new tab",
                contexts: ["link"],
                targetUrlPatterns: [`${EXTENSION_PAGE}*`]
            }, () => {
                console.log(chrome.runtime.lastError)
            })

            chrome.contextMenus.create({
                id: "open-in-tucan-plus",
                title: "Open link in TUCaN Plus",
                contexts: ["link"],
                targetUrlPatterns: ["https://www.tucan.tu-darmstadt.de/*"]
            }, () => {
                console.log(chrome.runtime.lastError)
            })

            chrome.contextMenus.create({
                id: "open-in-tucan-plus-new-tab",
                title: "Open link in TUCaN Plus in new tab",
                contexts: ["link"],
                targetUrlPatterns: ["https://www.tucan.tu-darmstadt.de/*"]
            }, () => {
                console.log(chrome.runtime.lastError)
            })

            chrome.contextMenus.create({
                id: "open-in-tucan-page",
                title: "Open page in TUCaN",
                contexts: ["page"],
                documentUrlPatterns: [`${EXTENSION_PAGE}*`]
            }, () => {
                console.log(chrome.runtime.lastError)
            })

            chrome.contextMenus.create({
                id: "open-in-tucan-page-new-tab",
                title: "Open page in TUCaN in new tab",
                contexts: ["page"],
                documentUrlPatterns: [`${EXTENSION_PAGE}*`]
            }, () => {
                console.log(chrome.runtime.lastError)
            })

            chrome.contextMenus.create({
                id: "open-in-tucan-plus-page",
                title: "Open page in TUCaN Plus",
                contexts: ["page"],
                documentUrlPatterns: ["https://www.tucan.tu-darmstadt.de/*"]
            }, () => {
                console.log(chrome.runtime.lastError)
            })

            chrome.contextMenus.create({
                id: "open-in-tucan-plus-page-new-tab",
                title: "Open page in TUCaN Plus in new tab",
                contexts: ["page"],
                documentUrlPatterns: ["https://www.tucan.tu-darmstadt.de/*"]
            }, () => {
                console.log(chrome.runtime.lastError)
            })

            chrome.contextMenus.create({
                id: "shareable-link-page",
                title: "Share link to page (without session id)",
                contexts: ["page"],
                documentUrlPatterns: ["https://www.tucan.tu-darmstadt.de/*", `${EXTENSION_PAGE}*`]
            }, () => {
                console.log(chrome.runtime.lastError)
            })

            chrome.contextMenus.create({
                id: "shareable-link",
                title: "Share link (without session id)",
                contexts: ["link"],
                documentUrlPatterns: ["https://www.tucan.tu-darmstadt.de/*", `${EXTENSION_PAGE}*`]
            }, () => {
                console.log(chrome.runtime.lastError)
            })
        });
    });
}
