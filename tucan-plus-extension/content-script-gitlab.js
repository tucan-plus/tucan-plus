console.log("tucan-plus gitlab")

let gitlabLoginButton = /** @type {HTMLButtonElement | null} */(document.querySelector('[data-testid="saml-login-button"]'))

if (gitlabLoginButton) {
    let rememberMe = /** @type {HTMLInputElement} */(document.querySelector("#js-remember-me-omniauth"))
    rememberMe.checked = true;
    rememberMe.dispatchEvent(new Event('input', { bubbles: true }));
    gitlabLoginButton.click();
}

let selectElement = /** @type {HTMLSelectElement | null} */(document.querySelector("#idpSelectSelector"))
if (selectElement) {
    selectElement.value = "https://idp.hrz.tu-darmstadt.de/idp/shibboleth";
    selectElement.dispatchEvent(new Event('input', { bubbles: true }));
    selectElement.dispatchEvent(new Event('change', { bubbles: true }));

    let rememberRadio = /** @type {HTMLInputElement} */(document.querySelector('[aria-label="9 months"]'));
    rememberRadio.checked = true;
    rememberRadio.dispatchEvent(new Event('input', { bubbles: true }));

    /** @type {HTMLInputElement} */(document.querySelector("#idpSelectListButton")).click();
}
