console.log("tucan-plus tucan")

if (document.querySelector('#logIn_btn')) {
    document.location.href = "https://dsf.tucan.tu-darmstadt.de/IdentityServer/External/Challenge?provider=dfnshib&returnUrl=%2FIdentityServer%2Fconnect%2Fauthorize%2Fcallback%3Fclient_id%3DClassicWeb%26scope%3Dopenid%2520DSF%2520email%26response_mode%3Dquery%26response_type%3Dcode%26ui_locales%3Dde%26redirect_uri%3Dhttps%253A%252F%252Fwww.tucan.tu-darmstadt.de%252Fscripts%252Fmgrqispi.dll%253FAPPNAME%253DCampusNet%2526PRGNAME%253DLOGINCHECK%2526ARGUMENTS%253D-N000000000000001,ids_mode%2526ids_mode%253DY";
}
