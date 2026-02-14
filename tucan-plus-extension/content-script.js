// if you activate the extension while having a tucan page open we want to find out the session id in any way possible

// vendored, because we can't use ES modules here
/**
 * 
 * @param {() => Promise<void>} closure 
 */
function asyncClosure(closure) {
    closure().catch(/** @param {unknown} error */ error => {
        console.error(error)
        chrome.notifications.create({
            type: "basic",
            iconUrl: chrome.runtime.getURL("/logo.png"),
            title: "TUCaN Plus extension error",
            message: String(error),
        });
    })
}

const imprintInFooter = /** @type {HTMLAnchorElement | null} */ (document.getElementById("pageFootControl_imp"))

if (document.body.classList.contains("access_denied")) {
    document.cookie = `id=; Secure; expires=Thu, 01 Jan 1970 00:00:00 UTC`;
} else if (document.body.classList.contains("redirect")) {
    const sessionId = /** @type {HTMLElement} */ (document.getElementById("sessionId"))
    if (sessionId.innerText === "000000000000001") {
        document.cookie = `id=; Secure; expires=Thu, 01 Jan 1970 00:00:00 UTC`;
    } else {
        document.cookie = `id=${sessionId.innerText}; Secure`;
    }
} else if (location.href === "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll") {
    // empty
} else if (imprintInFooter) {
    const args = /** @type {string} */ (new URL(imprintInFooter.href).searchParams.get("ARGUMENTS"))
    const sessionId = /** @type {string} */ (/^-N(?<id>\d+),/.exec(args)?.groups?.id)
    if (sessionId === "000000000000001") {
        document.cookie = `id=; Secure; expires=Thu, 01 Jan 1970 00:00:00 UTC`;
    } else {
        document.cookie = `id=${sessionId}; Secure`;
    }
} else {
    console.log("unknown part")
}

window.addEventListener("tucan-plus", event => {
    asyncClosure(async () => {
        console.log(event)
        await chrome.runtime.sendMessage(/** @type {CustomEvent} */(event).detail)
    })
})

console.log("content script")

const loginButton = /** @type {HTMLAnchorElement | null} */ (document.getElementById("logIn_btn"))

if (loginButton) {
    loginButton.href = "https://dsf.tucan.tu-darmstadt.de/IdentityServer/External/Challenge?provider=dfnshib&returnUrl=%2FIdentityServer%2Fconnect%2Fauthorize%2Fcallback%3Fclient_id%3DClassicWeb%26scope%3Dopenid%2520DSF%2520email%26response_mode%3Dquery%26response_type%3Dcode%26ui_locales%3Dde%26redirect_uri%3Dhttps%253A%252F%252Fwww.tucan.tu-darmstadt.de%252Fscripts%252Fmgrqispi.dll%253FAPPNAME%253DCampusNet%2526PRGNAME%253DLOGINCHECK%2526ARGUMENTS%253D-N000000000000001,ids_mode%2526ids_mode%253DY";
}