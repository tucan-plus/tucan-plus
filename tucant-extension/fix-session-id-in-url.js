import { asyncClosure } from "./utils.js";

chrome.webRequest.onBeforeRequest.addListener((details) => {
    asyncClosure(async () => {
        if (details.url === "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll" && details.method === "POST") {
            console.log("login attempt")
            await chrome.cookies.remove({
                url: "https://www.tucan.tu-darmstadt.de/scripts",
                name: "id",
            })
        }
    });
}, { urls: ["https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll"] })

chrome.webRequest.onHeadersReceived.addListener((details) => {
    asyncClosure(async () => {
        if (details.url === "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll" && details.method === "POST") {
            const refreshHeader = details.responseHeaders?.find(v => v.name === "REFRESH")?.value ?? "";
            const match = new RegExp("^0; URL=/scripts/mgrqispi\\.dll\\?APPNAME=CampusNet&PRGNAME=STARTPAGE_DISPATCH&ARGUMENTS=-N(\\d+),-N\\d+,-N000000000000000$", "g").exec(refreshHeader);
            if (match !== null) {
                const sessionId = match[1]

                await chrome.cookies.set({
                    url: "https://www.tucan.tu-darmstadt.de/scripts",
                    name: "id",
                    value: sessionId,
                    secure: true
                })
            }
        }

        const logoutMatch = new RegExp("^https://www\\.tucan\\.tu-darmstadt\\.de/scripts/mgrqispi\\.dll\\?APPNAME=CampusNet&PRGNAME=LOGOUT&.*$", "g").exec(details.url);
        if (logoutMatch !== null) {
            await chrome.cookies.remove({
                url: "https://www.tucan.tu-darmstadt.de/scripts",
                name: "id",
            })
        }
    });
}, { urls: ["https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll", "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=LOGOUT&*"] }, ["responseHeaders"]);

chrome.cookies.onChanged.addListener((changeInfo) => {
    asyncClosure(async () => {
        if (changeInfo.cookie.name === "cnsc" && changeInfo.removed) {
            await chrome.cookies.remove({
                url: "https://www.tucan.tu-darmstadt.de/scripts",
                name: "id",
            })
        } else if (changeInfo.cookie.name === "id") {
            if (changeInfo.removed && changeInfo.cause !== "overwrite") {
                await chrome.declarativeNetRequest.updateDynamicRules({
                    removeRuleIds: fixupSessionIdInUrl("").map(r => r.id),
                    addRules: [],
                });
                await chrome.action.setBadgeText({ text: "" })
            } else {
                await chrome.action.setBadgeText({ text: "L" })
                await chrome.action.setBadgeBackgroundColor(
                    { color: 'green' }
                )
                await chrome.action.setBadgeTextColor({ color: "white" });

                if ((await chrome.storage.sync.get(
                    { fixSessionIdInUrl: true },
                )).fixSessionIdInUrl) {
                    await chrome.declarativeNetRequest.updateDynamicRules({
                        removeRuleIds: fixupSessionIdInUrl(changeInfo.cookie.value).map(r => r.id),
                        addRules: fixupSessionIdInUrl(changeInfo.cookie.value),
                    });
                }
            }
        }
    });
});

const fixupSessionIdInUrl = (/** @type {string} */ sessionId) => {
    console.log("fixup session id in url")
    return [{
        // redirect any session id to the currently valid session id
        id: 100,
        condition: {
            isUrlFilterCaseSensitive: true,
            resourceTypes: [
            /** @type {chrome.declarativeNetRequest.ResourceType} */ ("main_frame")
            ],
            regexFilter: `^https://www\\.tucan\\.tu-darmstadt\\.de/scripts/mgrqispi\\.dll\\?APPNAME=CampusNet&PRGNAME=([A-Z_]+)&ARGUMENTS=-N\\d+,(.+)$`
        },
        action: {
            type: /** @type {chrome.declarativeNetRequest.RuleActionType} */ ('redirect'),
            redirect: {
                regexSubstitution: `https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=\\1&ARGUMENTS=-N${sessionId},\\2`,
            },
        },
    }, {
        // but don't create an infinite loop
        id: 101,
        priority: 2,
        condition: {
            isUrlFilterCaseSensitive: true,
            resourceTypes: [
            /** @type {chrome.declarativeNetRequest.ResourceType} */ ("main_frame")
            ],
            regexFilter: `^https://www\\.tucan\\.tu-darmstadt\\.de/scripts/mgrqispi\\.dll\\?APPNAME=CampusNet&PRGNAME=([A-Z_]+)&ARGUMENTS=-N${sessionId},(.+)$`
        },
        action: {
            type: /** @type {chrome.declarativeNetRequest.RuleActionType} */ ('allow')
        },
    }, {
        // and don't redirect explicitly unauthenticated urls
        id: 102,
        priority: 2,
        condition: {
            isUrlFilterCaseSensitive: true,
            resourceTypes: [
            /** @type {chrome.declarativeNetRequest.ResourceType} */ ("main_frame")
            ],
            regexFilter: `^https://www\\.tucan\\.tu-darmstadt\\.de/scripts/mgrqispi\\.dll\\?APPNAME=CampusNet&PRGNAME=([A-Z_]+)&ARGUMENTS=-N000000000000001,(.+)$`
        },
        action: {
            type: /** @type {chrome.declarativeNetRequest.RuleActionType} */ ('allow')
        },
    }, {
        id: 103,
        priority: 2,
        condition: {
            isUrlFilterCaseSensitive: true,
            resourceTypes: [
            /** @type {chrome.declarativeNetRequest.ResourceType} */ ("main_frame")
            ],
            regexFilter: `^https://www\\.tucan\\.tu-darmstadt\\.de/$`
        },
        action: {
            type: /** @type {chrome.declarativeNetRequest.RuleActionType} */ ('redirect'),
            redirect: {
                regexSubstitution: `https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=MLSSTART&ARGUMENTS=-N${sessionId},-N000019,`,
            },
        },
    }]
};

chrome.storage.sync.onChanged.addListener((changes) => {
    asyncClosure(async () => {
        for (let [key, { newValue }] of Object.entries(changes)) {
            if (key === "fixSessionIdInUrl") {
                if (newValue) {
                    const id = await chrome.cookies.get({
                        url: "https://www.tucan.tu-darmstadt.de/scripts",
                        name: "id",
                    })
                    if (id) {
                        await chrome.declarativeNetRequest.updateDynamicRules({
                            removeRuleIds: fixupSessionIdInUrl(id.value).map(r => r.id),
                            addRules: fixupSessionIdInUrl(id.value),
                        });
                    } else {
                        await chrome.declarativeNetRequest.updateDynamicRules({
                            removeRuleIds: fixupSessionIdInUrl("").map(r => r.id),
                            addRules: [],
                        });
                    }
                } else {
                    await chrome.declarativeNetRequest.updateDynamicRules({
                        removeRuleIds: fixupSessionIdInUrl("").map(r => r.id),
                        addRules: [],
                    });
                }
            }
        }
    });
});

// ensure state is set on extension enable
asyncClosure(async () => {
    const idCookie = await chrome.cookies.get({
        url: "https://www.tucan.tu-darmstadt.de/scripts",
        name: "id",
    })
    if (idCookie) {
        await chrome.action.setBadgeText({ text: "L" })
        await chrome.action.setBadgeBackgroundColor(
            { color: 'green' }
        )
        await chrome.action.setBadgeTextColor({ color: "white" });
        await chrome.declarativeNetRequest.updateDynamicRules({
            removeRuleIds: fixupSessionIdInUrl(idCookie.value).map(r => r.id),
            addRules: fixupSessionIdInUrl(idCookie.value),
        });
    } else {
        await chrome.declarativeNetRequest.updateDynamicRules({
            removeRuleIds: fixupSessionIdInUrl("").map(r => r.id),
            addRules: [],
        });
        await chrome.action.setBadgeText({ text: "" })
    }
})

export { }