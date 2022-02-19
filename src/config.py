# encoding: utf-8


class Config(object):
    # Number of results to fetch from API
    RESULT_COUNT = 20
    # How long to cache results for
    CACHE_MAX_AGE = 20  # seconds
    # Icon
    FLUTTER_ICON = "icon.png"
    GOOGLE_ICON = "google.png"
    # Algolia credentials
    ALGOLIA_APP_ID = "WODHKE4WZG"
    ALGOLIA_SEARCH_ONLY_API_KEY = "7456cdd91ba8d4f87846549697397759"
    ALGOLIA_SEARCH_INDEX = "flutter"
