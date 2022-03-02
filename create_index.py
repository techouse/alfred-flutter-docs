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

    full_data = [
        {
            **el,
            **{
                "weight": filters[el["type"]] - 1
                if el["packageName"] == "flutter"
                else filters[el["type"]]
            },
        }
        for el in data
        if el["type"] in filters.keys()
    ]

    with open("full_index.json", "w") as out_fh:
        json.dump(full_data, out_fh)
