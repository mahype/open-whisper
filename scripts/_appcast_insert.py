#!/usr/bin/env python3
"""
Prepend a <item> entry to a Sparkle appcast.xml. Everything comes in via
env vars so the caller (a shell script) doesn't have to juggle quoting.

Required env:
  APPCAST_PATH, VERSION, RELEASE_NOTES_URL, DMG_URL, DMG_LENGTH,
  DMG_ED_SIGNATURE, MIN_SYSTEM_VERSION, PUB_DATE
"""
import os
import sys
from xml.etree import ElementTree as ET

NS_SPARKLE = "http://www.andymatuschak.org/xml-namespaces/sparkle"
ET.register_namespace("sparkle", NS_SPARKLE)


def require(name):
    value = os.environ.get(name)
    if not value:
        sys.exit(f"error: {name} is required")
    return value


def main():
    path = require("APPCAST_PATH")
    version = require("VERSION")
    notes_url = require("RELEASE_NOTES_URL")
    dmg_url = require("DMG_URL")
    dmg_len = require("DMG_LENGTH")
    ed_sig = require("DMG_ED_SIGNATURE")
    min_sys = require("MIN_SYSTEM_VERSION")
    pub_date = require("PUB_DATE")

    tree = ET.parse(path)
    channel = tree.getroot().find("channel")
    if channel is None:
        sys.exit("error: <channel> not found in appcast")

    if any(
        elem.findtext(f"{{{NS_SPARKLE}}}shortVersionString") == version
        for elem in channel.findall("item")
    ):
        print(f"appcast already contains version {version}; skipping", file=sys.stderr)
        return

    item = ET.Element("item")
    ET.SubElement(item, "title").text = f"Version {version}"
    ET.SubElement(item, "pubDate").text = pub_date
    ET.SubElement(item, f"{{{NS_SPARKLE}}}shortVersionString").text = version
    ET.SubElement(item, f"{{{NS_SPARKLE}}}version").text = version
    ET.SubElement(item, f"{{{NS_SPARKLE}}}releaseNotesLink").text = notes_url
    ET.SubElement(item, f"{{{NS_SPARKLE}}}minimumSystemVersion").text = min_sys
    ET.SubElement(
        item,
        "enclosure",
        {
            "url": dmg_url,
            f"{{{NS_SPARKLE}}}edSignature": ed_sig,
            "length": dmg_len,
            "type": "application/octet-stream",
        },
    )

    last_meta_idx = 0
    for idx, elem in enumerate(list(channel)):
        if elem.tag != "item":
            last_meta_idx = idx
    channel.insert(last_meta_idx + 1, item)

    ET.indent(tree, space="  ")
    tree.write(path, encoding="UTF-8", xml_declaration=True)


if __name__ == "__main__":
    main()
