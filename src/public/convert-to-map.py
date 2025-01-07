import os
import subprocess

contents = {

}

def add_to_clipboard(text: str):
    subprocess.run('clip', universal_newlines=True, input=text)

def get_camel_case_name(filename: str) -> str:
    filename = filename.removesuffix(".svg")
    split = filename.split("-")

    for i in range(0, len(split)):
        split[i] = split[i].capitalize()

    return str.join("", split)


for file in os.scandir('./icons'):
    if not file.is_file():
        continue

    filename = file.name

    if not filename.endswith('.svg'):
        continue

    camel_case_name = get_camel_case_name(filename)

    with open(file.path, 'r') as f:
        if f.readline().startswith("<?xml"): pass
        else: f.seek(0)

        contents[f"[IconName.{camel_case_name}]"] = f.read()

add_to_clipboard(str(contents).replace("'", "").replace("\\n", ""))
