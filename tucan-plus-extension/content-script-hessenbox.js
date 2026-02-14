console.log("tucan-plus hessenbox")

let dropdownToggle = /** @type {HTMLAnchorElement | null} */(document.querySelector(".IdPSelectDropDownToggle"))

if (dropdownToggle) {
    dropdownToggle.click();
    let selectElement = /** @type {HTMLSelectElement} */(document.querySelector("#idpSelectSelector"))
    selectElement.value = "https://idp.hrz.tu-darmstadt.de/idp/shibboleth";
    selectElement.dispatchEvent(new Event('input', { bubbles: true }));
    selectElement.dispatchEvent(new Event('change', { bubbles: true }));
    /** @type {HTMLInputElement} */ (document.querySelector("#idpSelectListButton")).click()
}

// TODO tucan automatically click login button