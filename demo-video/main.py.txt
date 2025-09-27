#!/usr/bin/python3
import os
import subprocess
import tempfile

import gi
import obsws_python as obs

gi.require_version('Atspi', '2.0')
from gi.repository import Atspi

from dogtail.tree import root, Node
from dogtail.config import config
from dotenv import load_dotenv
from time import sleep
from pathlib import PurePath, Path
from contextlib import contextmanager

from pyatspi import SCROLL_ANYWHERE

import torchaudio as ta
from chatterbox.tts import ChatterboxTTS

load_dotenv()

config.searchShowingOnly = True

# python3 -i main.py

record_state = "OBS_WEBSOCKET_OUTPUT_STOPPED"

# python3.11 -m venv env
# source env/bin/activate
# pip install "kokoro>=0.9.4" soundfile
# python tts.py

model = ChatterboxTTS.from_pretrained(device="cpu")

def on_record_state_changed(data):
    global record_state
    print(data.output_state)
    print(type(data.output_state))
    record_state = data.output_state

@contextmanager
def recording(filename):
    while record_state != "OBS_WEBSOCKET_OUTPUT_STOPPED":
        print(f"1 waiting {record_state} to become OBS_WEBSOCKET_OUTPUT_STOPPED")
        sleep(0.1)
        continue
    req_client.start_record()
    while record_state != "OBS_WEBSOCKET_OUTPUT_STARTED":
        print(f"2 waiting {record_state} to become OBS_WEBSOCKET_OUTPUT_STARTED")
        sleep(0.1)
        continue
    try:
        yield ()
    finally:
        while record_state != "OBS_WEBSOCKET_OUTPUT_STARTED":
            print(f"3 waiting {record_state} to become OBS_WEBSOCKET_OUTPUT_STARTED")
            sleep(0.1)
            continue
        output_path = PurePath(req_client.stop_record().output_path)
        while record_state != "OBS_WEBSOCKET_OUTPUT_STOPPED":
            print(f"4 waiting {record_state} to become OBS_WEBSOCKET_OUTPUT_STOPPED")
            sleep(0.1)
            continue
        os.rename(output_path, output_path.with_name(filename+".mkv"))

def toggle_navigation():
    # TODO maybe we can check whether it is expanded in accessibility info
    return
    # on mobile
    #firefox.child("Toggle navigation", "button").click()

def step1_open_tucan_plus_installation_page():
    urlbar_input: Node = firefox.child(identifier="urlbar-input")
    urlbar_input.click()
    urlbar_input.keyCombo("<ctrl><a>")
    urlbar_input.typeText("https://tucan-plus.github.io/tucan-plus/")
    urlbar_input.keyCombo("<enter>")

def step2_install_extension():
    sleep(2)
    download_button = firefox.child("Download extension for Firefox", "link")
    try:
        download_button.click()
    except ValueError:
        print(f"clicking failed")
    download_button = firefox.child("Download extension for Firefox", "link")
    download_button.click()

    sleep(2)
    firefox.child("Continue to Installation", "button").click()
    firefox.child("Add", "button").click()
    firefox.child("OK", "button").click()

def step2_5_extension_settings():
    firefox.child("Extensions", "button").click()
    sleep(1)
    firefox.child("TUCaN Plus", "button").click()
    firefox.child("Go to options", "button").click()
    firefox.child("Anonymize grades (for demoing).", "check box").click()

def step3_open_tucan_plus():
    urlbar_input: Node = firefox.child(identifier="urlbar-input")
    urlbar_input.click()
    urlbar_input.keyCombo("<ctrl><a>")
    urlbar_input.typeText("https://www.tucan.tu-darmstadt.de/")
    urlbar_input.keyCombo("<enter>")

def step4_login():
    toggle_navigation()

    username_input: Node = firefox.child(identifier="login-username")
    username_input.click()
    username_input.keyCombo("<ctrl><a>")
    username_input.typeText(os.getenv("TUCAN_USERNAME"))

    password_input: Node = firefox.child(identifier="login-password")
    password_input.click()
    password_input.keyCombo("<ctrl><a>")
    password_input.typeText(os.getenv("TUCAN_PASSWORD"))

    login_button: Node = firefox.child(identifier="login-button")
    login_button.click()

def step5_aktuelles():
    toggle_navigation()

    aktuelles_button = firefox.child("Aktuelles", "button")
    aktuelles_button.scroll_to(SCROLL_ANYWHERE)
    aktuelles_button.click()
    firefox.child("Aktuelles", "link").click()

def step6_vv():
    toggle_navigation()

    firefox.child("VV", "button").click()
    firefox.child("Vorlesungsverzeichnis", "link").click()

    informatik_link: Atspi.Component | Node = firefox.child("FB20 - Informatik", "link", showingOnly=False)
    informatik_link.scroll_to(SCROLL_ANYWHERE)
    sleep(0.5) # scrolling seems to have a delay
    informatik_link.click()

    pflichtveranstaltungen: Atspi.Component | Node = firefox.child("Pflichtveranstaltungen", "link")
    pflichtveranstaltungen.click()

    aud: Atspi.Component | Node = firefox.child("Kurs 20-00-0005-iv Algorithmen und Datenstrukturen", "link")
    aud.click()

    aud: Atspi.Component | Node = firefox.child("Algorithmen und Datenstrukturen 01", "link")
    aud.click()
    aud.doActionNamed("jump") # click does not work on mobile mode as it clicks at the wrong place
    sleep(3)

    # https://gitlab.gnome.org/GNOME/gtk/-/blob/main/gdk/gdkkeysyms.h
    # https://github.com/vhumpa/dogtail/blob/3600ef901bcd7b4f8d64dce17a600219dcc1abf9/dogtail/rawinput.py#L477
    # TODO maybe we can improve this scrolling
    firefox.keyCombo("<pagedown>")
    sleep(1)
    firefox.keyCombo("<pagedown>")
    sleep(1)
    firefox.keyCombo("<pagedown>")
    sleep(1)
    firefox.keyCombo("<pagedown>")
    sleep(1)
    firefox.keyCombo("<pagedown>")
    sleep(1)
    firefox.keyCombo("<pagedown>")
    sleep(1)
    firefox.keyCombo("<pagedown>")
    sleep(3)
    firefox.keyCombo("<Home>")

def step7_semestermodule():
    toggle_navigation()
    firefox.child("Veranstaltungen", "button").click()
    firefox.child("Meine Semestermodule", "link").click()
    sleep(3)
    firefox.child("Select semester", "combo box").click()
    firefox.child("WiSe 2024/25", "menu item").click()
    sleep(3)

def step8_veranstaltungen():
    toggle_navigation()
    firefox.child("Veranstaltungen", "button").click()
    firefox.child("Meine Veranstaltungen", "link").click()
    sleep(3)
    firefox.child("Select semester", "combo box").click()
    firefox.child("WiSe 2024/25", "menu item").click()
    sleep(3)

def step9_anmeldung_und_pruefungen():
    toggle_navigation()
    firefox.child("Veranstaltungen", "button").click()
    firefox.child("Anmeldung", "link").click()

    informatik_link: Atspi.Component | Node = firefox.child("Vertiefungen, Wahlbereiche und Studium Generale", "link")
    informatik_link.scroll_to(SCROLL_ANYWHERE)
    sleep(0.5) # scrolling seems to have a delay
    informatik_link.click()

    informatik_link: Atspi.Component | Node = firefox.child("Vertiefungen", "link")
    informatik_link.scroll_to(SCROLL_ANYWHERE)
    sleep(0.5) # scrolling seems to have a delay
    informatik_link.click()

    informatik_link: Atspi.Component | Node = firefox.child("Fachprüfungen aus den Basis Wahlbereichen und Wahlbereichen der Individuellen Vertiefung", "link")
    informatik_link.scroll_to(SCROLL_ANYWHERE)
    sleep(0.5) # scrolling seems to have a delay
    informatik_link.click()

    informatik_link: Atspi.Component | Node = firefox.child("Basis Wahlbereiche", "link")
    informatik_link.scroll_to(SCROLL_ANYWHERE)
    sleep(0.5) # scrolling seems to have a delay
    informatik_link.click()

    informatik_link: Atspi.Component | Node = firefox.child("Theorie (Theoretische Informatik)", "link")
    informatik_link.scroll_to(SCROLL_ANYWHERE)
    sleep(0.5) # scrolling seems to have a delay
    informatik_link.click()

    # seamless interop between tucan plus and tucan
    informatik_link: Atspi.Component | Node = firefox.child("Zum Modul anmelden", "button")
    informatik_link.scroll_to(SCROLL_ANYWHERE)
    sleep(0.5) # scrolling seems to have a delay
    informatik_link.click()

    sleep(2)

    # seamless interop between tucan and tucan plus
    firefox.child("Prüfungen", "link").click()
    firefox.child("Meine Prüfungen", "link").click()

    # if you want you can still open in tucan
    firefox.child("TUCaN Plus", "document web").click(3) # right click somewhere on page
    firefox.child("TUCaN Plus", "menu").click()
    firefox.child("Open page in TUCaN", "menu item").click()
    # Open page in TUCaN in new tab

    sleep(2)

    firefox.child("Technische Universität Darmstadt", "document web").click(3) # right click somewhere on page
    firefox.child("TUCaN Plus", "menu").click()
    firefox.child("Open page in TUCaN Plus in new tab", "menu item").click()

    sleep(1)

def step10_ergebnisse():
    sleep(2)

    toggle_navigation()
    firefox.child("Prüfungen", "button").click()
    firefox.child("Modulergebnisse", "link").click()
    sleep(5)

    toggle_navigation()
    firefox.child("Prüfungen", "button").click()
    firefox.child("Prüfungsergebnisse", "link").click()
    sleep(5)

    toggle_navigation()
    firefox.child("Prüfungen", "button").click()
    firefox.child("Leistungsspiegel", "link").click()
    sleep(5)
    firefox.child("Select course of study", "combo box").click()
    firefox.child("B.Sc. Informatik (2015)", "menu item").click()
    sleep(5)

ta.save("/home/moritz/Videos/tucan-plus/1.wav", model.generate("""
Why does TUCaN load so slowly?

Why can't I share URLs with other students?

Why is it so bad on mobile?

Why is the registration menu so slow?

If you have been asking yourself the same, we have a solution for you.

We introduce TUCaN Plus the best extension to make TUCaN can again.

It will remove questionable half a second waits in the code of TUCaN and skip unecessary navigations.

Also it works nicely on mobile with a completely new user interface.

It also caches pages you already viewed before so TUCaN is not always so slow. Unfortunately we can't fix it being slow the first time.

But let's look at the features in detail:





How do I install this cool extension?

Go to https://tucan-plus.github.io/tucan-plus/.

Then, click on download extension for Firefox.

Now, confirm the installation prompts.

How do I configure TUCaN Plus?

Click on the extension icon in the top right and select TUCaN Plus.

Now click on Go to options.
"""), model.sr)

with tempfile.TemporaryDirectory() as tmpdirname:
    with open(Path(tmpdirname, "user.js"), "w") as text_file:
        # https://github.com/mozilla-firefox/firefox/blob/e93030b39fb3f3e8f9279bbb57107a8315d2c40a/browser/locales/en-US/browser/featureCallout.ftl#L103
        print("""
                user_pref('browser.newtabpage.activity-stream.asrouter.userprefs.cfr.features', false);
                user_pref('datareporting.policy.dataSubmissionEnabled', false);
                user_pref('signon.rememberSignons', false);
                user_pref('browser.translations.enable', false);
            """, file=text_file)
    print("test")
    firefox_process = subprocess.Popen(["/usr/bin/firefox", "--profile", tmpdirname, "-width", "1920", "-height", "1080", "about:blank"])
    print(firefox_process)
    sleep(1)
    firefox: Node = root.application("Firefox")
    input("Select window to record in OBS")

    # OBS -> Tools -> WebSocket Server Settings

    # https://github.com/obsproject/obs-websocket/blob/master/docs/generated/protocol.md
    req_client = obs.ReqClient(password='PZtbUAIwD8DPxzUT')
    event_client = obs.EventClient(password='PZtbUAIwD8DPxzUT')
    event_client.callback.register([on_record_state_changed])

    with recording("installation"):
        step1_open_tucan_plus_installation_page()
        step2_install_extension()
    with recording("settings"):
        step2_5_extension_settings()
    with recording("login"):
        step3_open_tucan_plus()
        step4_login()
    with recording("aktuelles"):
        step5_aktuelles()
    with recording("vv"):
        step6_vv()
    with recording("semestermodule"):
        step7_semestermodule()
    with recording("veranstaltungen"):
        step8_veranstaltungen()
    with recording("anmeldung_und_pruefungen"):
        step9_anmeldung_und_pruefungen()
    with recording("ergebnisse"):
        step10_ergebnisse()