#!/usr/bin/env python3
import json
import requests

res = requests.get(
    "https://api.flutter.dev/flutter/index.json"
)  # official Flutter docs index; currently contains about 62k indices

if res.ok:
    data = res.json()

    # filters with weights
    filters = {
        "library": 2,
        "class": 2,
        "mixin": 3,
        "extension": 3,
        "typedef": 3,
        "method": 4,
        "accessor": 4,
        "operator": 4,
        "constant": 4,
        "property": 4,
        "constructor": 4,
        "top-level property": 5,
        "function": 5,
        "enum": 5,
        "top-level constant": 5,
    }

    # index of kinds
    kinds = {
        0: "accessor",
        1: "constant",
        2: "constructor",
        3: "class",
        4: "dynamic",
        5: "enum",
        6: "extension",
        7: "function",
        8: "library",
        9: "method",
        10: "mixin",
        11: "Never",
        12: "package",
        13: "parameter",
        14: "prefix",
        15: "property",
        16: "SDK",
        17: "topic",
        18: "top-level constant",
        19: "top-level property",
        20: "typedef",
        21: "type parameter",
    }

    full_data = []

    for el in data:
        if "kind" in el and el["kind"] in kinds:
            el["type"] = kinds[el["kind"]]
            el["weight"] = el["kind"]
            del el["kind"]
            if "enclosedBy" in el:
                if "kind" in el["enclosedBy"]:
                    el["enclosedBy"]["type"] = kinds[el["enclosedBy"]["kind"]]
                    del el["enclosedBy"]["kind"]
        full_data.append(el)

    with open("full_index.json", "w") as out_fh:
        json.dump(full_data, out_fh)
