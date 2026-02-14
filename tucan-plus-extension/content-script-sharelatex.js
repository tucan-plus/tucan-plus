if (document.querySelector('a[href="/login"]')) {
    document.location.href = "https://sharelatex.tu-darmstadt.de/saml/login/go";
}

// https://sharelatex-01.ca.hrz.tu-darmstadt.de/Saml2/disco?entityID=https%3A%2F%2Fidp.hrz.tu-darmstadt.de%2Fidp%2Fshibboleth

let chooseIdp = document.querySelector('div.IdPSelectPreferredIdPButton[title="Technical University of Darmstadt"]')

if (chooseIdp) {
    chooseIdp.querySelector("a")?.click()
}