# Builds the on-screen popup's fonts (src/win/osd.rs) from Google Sans Code, the design system's
# text face (design/). GDI can't read WOFF2 or pick weights from a variable font, so this cuts
# static TrueType instances at the two weights the popup uses. Rerun it to update the font:
#
#   pip install fonttools
#   python3 scripts/osd-fonts.py
#
# Writes src/win/fonts/GoogleSansCode-{Regular,SemiBold}.ttf and the font's license (OFL.txt).
import urllib.request
from io import BytesIO

from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

SRC = "https://raw.githubusercontent.com/google/fonts/main/ofl/googlesanscode/"
OUT = "src/win/fonts/"


def fetch(name: str) -> bytes:
    with urllib.request.urlopen(SRC + urllib.request.quote(name)) as r:
        return r.read()


variable = fetch("GoogleSansCode[wght].ttf")
for weight, style in [(400, "Regular"), (600, "SemiBold")]:
    font = TTFont(BytesIO(variable))
    # Names the instance after its fvar entry: "Google Sans Code" / "Google Sans Code SemiBold".
    static = instantiateVariableFont(font, {"wght": weight}, updateFontNames=True)
    static.save(f"{OUT}GoogleSansCode-{style}.ttf")
open(f"{OUT}OFL.txt", "wb").write(fetch("OFL.txt"))
