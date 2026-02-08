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
            iconUrl: chrome.runtime.getURL("/logo.svg"),
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
} else if (imprintInFooter && document.cookie.includes("cnsc=")) {
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


/*
window.dispatchEvent(new CustomEvent('tucan-plus', { detail: "open-in-tucan-page" }));

let loginForm = /** @type {HTMLFormElement} (document.querySelector("#cn_loginForm"))

loginForm.addEventListener("submit", async event => {
    event.preventDefault()
    const formData = new FormData(loginForm);

    try {
        const response = await fetch("/scripts/mgrqispi.dll", {
            method: "POST",
            // Set the FormData instance as the request body
            body: formData,
        });
        const refreshHeader = response.headers.get("refresh")
        const match = new RegExp("^0; URL=/scripts/mgrqispi\\.dll\\?APPNAME=CampusNet&PRGNAME=STARTPAGE_DISPATCH&ARGUMENTS=-N(\\d+),-N\\d+,-N000000000000000$", "g").exec(refreshHeader);
        if (match !== null) {
            const sessionId = match[1]

            //chrome.cookies.set({
            //    url: "https://www.tucan.tu-darmstadt.de/scripts",
            //    name: "id",
            //    value: sessionId,
            //    secure: true
            //})
        }
        // TODO check if it's a not logged in url and then change to a logged in one
        // some urls like the vv urls are different when logged in, this is going to be fun.
        location.reload()
    } catch (e) {
        console.error(e);
    }
})
    */