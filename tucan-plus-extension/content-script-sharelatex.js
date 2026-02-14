console.log("tucan-plus sharelatex")

if (document.querySelector('a[href="/login"]')) {
    document.location.href = "https://sharelatex.tu-darmstadt.de/saml/login/go";
}

let chooseIdp = document.querySelector('div.IdPSelectPreferredIdPButton[title="Technical University of Darmstadt"]')

if (chooseIdp) {
    chooseIdp.querySelector("a")?.click()
}
