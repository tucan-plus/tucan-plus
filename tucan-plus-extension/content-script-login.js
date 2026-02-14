console.log("tucan-plus login")

const usernameInput = /** @type { HTMLInputElement | null} */ (document.querySelector('#username'))
const passwordInput = /** @type { HTMLInputElement | null} */ (document.querySelector('#password'))

/**
 * @param {HTMLInputElement} usernameInput
 * @param {HTMLInputElement} passwordInput
 */
function handle(usernameInput, passwordInput) {
    if (usernameInput.value !== "" && passwordInput.value !== "") {
        /** @type { HTMLButtonElement } */ (document.querySelector('[name="_eventId_proceed"]')).click()
    }
}

if (usernameInput && passwordInput) {
    passwordInput.addEventListener("change", (event) => {
        handle(usernameInput, passwordInput)
    })
    usernameInput.addEventListener("change", (event) => {
        handle(usernameInput, passwordInput)
    })
}

if (document.querySelector("#fudis_selected_token_ids_input")) {
    /** @type { HTMLButtonElement } */ (document.querySelector('[name="_eventId_proceed"]')).click()
}