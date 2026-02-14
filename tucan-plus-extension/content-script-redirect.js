let url = window.location.href;
url = url.replaceAll(new RegExp("^https://www\\.tucan\\.tu-darmstadt\\.de/scripts/mgrqispi\\.dll\\?APPNAME=CampusNet&PRGNAME=CHANGELANGUAGE&ARGUMENTS=-N(\\d+),-N001$", "g"),
    "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=MLSSTART&ARGUMENTS=-N$1,-N000019,");
url = url.replaceAll(new RegExp("^https://www\\.tucan\\.tu-darmstadt\\.de/scripts/mgrqispi\\.dll\\?APPNAME=CampusNet&PRGNAME=CHANGELANGUAGE&ARGUMENTS=-N(\\d+),-N002$", "g"),
    "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=MLSSTART&ARGUMENTS=-N$1,-N000350,");
url = url.replaceAll(new RegExp("^https://www\\.tucan\\.tu-darmstadt\\.de/scripts/mgrqispi\\.dll\\?APPNAME=CampusNet&PRGNAME=STARTPAGE_DISPATCH&ARGUMENTS=-N(\\d+),-N(\\d+),-N000000000000000$", "g"),
    "https://www.tucan.tu-darmstadt.de/scripts/mgrqispi.dll?APPNAME=CampusNet&PRGNAME=MLSSTART&ARGUMENTS=-N$1,-N$2,");
if (url !== window.location.href) {
    console.log("REDIRECTING")
    window.location.href = url;
} else {
    console.log("NOT REDIRECTING")
}