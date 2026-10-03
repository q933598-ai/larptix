#!/usr/bin/env python3
from pathlib import Path

path = Path("src-tauri/gen/android/app/build.gradle.kts")
text = path.read_text(encoding="utf-8")

imports = [
    "import java.io.FileInputStream",
    "import java.util.Properties",
]
for imp in reversed(imports):
    if imp not in text:
        text = imp + "\n" + text

signing_block = r'''    signingConfigs {
        create("release") {
            val keystorePropertiesFile = rootProject.file("keystore.properties")
            val keystoreProperties = Properties()
            if (keystorePropertiesFile.exists()) {
                keystoreProperties.load(FileInputStream(keystorePropertiesFile))
            }
            keyAlias = keystoreProperties["keyAlias"] as String
            keyPassword = keystoreProperties["keyPassword"] as String
            storeFile = file(keystoreProperties["storeFile"] as String)
            storePassword = keystoreProperties["storePassword"] as String
        }
    }

'''

if 'create("release")' not in text:
    marker = "    buildTypes {"
    if marker not in text:
        raise SystemExit("Could not find buildTypes block in Android app/build.gradle.kts")
    text = text.replace(marker, signing_block + marker, 1)

def find_block_end(source: str, start: int) -> int:
    open_brace = source.find("{", start)
    if open_brace < 0:
        raise ValueError("Missing opening brace")
    depth = 0
    in_string = None
    escape = False
    i = open_brace
    while i < len(source):
        ch = source[i]
        if in_string:
            if escape:
                escape = False
            elif ch == "\":
                escape = True
            elif ch == in_string:
                in_string = None
        else:
            if ch in ('"', "'"):
                in_string = ch
            elif ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    return i
        i += 1
    raise ValueError("Unbalanced braces")

release = 'getByName("release")'
release_index = text.find(release)
if release_index < 0:
    marker = "    buildTypes {"
    insertion = '''        getByName("release") {
            signingConfig = signingConfigs.getByName("release")
        }
'''
    text = text.replace(marker, marker + "\n" + insertion, 1)
else:
    open_brace = text.find("{", release_index)
    close_brace = find_block_end(text, release_index)
    block = text[release_index:close_brace]
    if "signingConfig = signingConfigs.getByName(" not in block:
        insertion = '\n        signingConfig = signingConfigs.getByName("release")'
        text = text[:open_brace + 1] + insertion + text[open_brace + 1:]

path.write_text(text, encoding="utf-8")
print(f"Configured Android release signing in {path}")
