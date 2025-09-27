from dogtail.tree import root, Node

def tucan_plus_cli():
    firefox: Node = root.application("Firefox")
    urlbar_input: Node = firefox.child(identifier="urlbar-input")
    urlbar_input.click()
    urlbar_input.keyCombo("<ctrl><a>")
    urlbar_input.typeText("https://tucan-plus.github.io/tucan-plus/")
    urlbar_input.keyCombo("<enter>")

if __name__ == "__main__":
    tucan_plus_cli()