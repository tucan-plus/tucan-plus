console.log("tucan-plus login")

let passwordInput = /** @type { HTMLInputElement | null} */ (document.querySelector('#password'))

if (passwordInput) {
    passwordInput.addEventListener("change", (event) => {
        console.log("ONCHANGE", passwordInput?.value)
    })
}
