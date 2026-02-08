/**
 * 
 * @param {() => Promise<void>} closure 
 */
export function asyncClosure(closure) {
    closure().catch(/** @param error {unknown} */ error => {
        console.error(error)
        chrome.notifications.create({
            type: "basic",
            iconUrl: chrome.runtime.getURL("/logo.svg"),
            title: "TUCaN Plus extension error",
            message: String(error),
        });
    })
}