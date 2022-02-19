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
        # 'constant' : 4,                       # skip constants to keep the number of indices below the index cap
        "property": 4,
        "constructor": 4,
        "top-level property": 5,
        "function": 5,
        "enum": 5,
        "top-level constant": 5,
    }

    filtered_data = [
        {
            **el,
            **{"weight": filters[el["type"]]},  # add weights to make the result sort relevant
        }
        for el in data
        if el["type"] in filters.keys()         # filter by the filters listed above
        and el["packageName"] == "flutter"      # additionally, index only items with the "flutter" packageName
    ]

    with open("filtered_index.json", "w") as out_fh:
        json.dump(filtered_data, out_fh)
