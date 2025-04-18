part of 'main.dart';

final AlfredWorkflow _workflow = AlfredWorkflow()
  ..disableAlfredSmartResultOrdering = true;

final AlfredUpdater _updater = AlfredUpdater(
  githubRepositoryUrl: Env.githubRepositoryUrl,
  currentVersion: Env.appVersion,
  updateInterval: Duration(days: 7),
);

const updateItem = AlfredItem(
  title: 'Auto-Update available!',
  subtitle: 'Press <enter> to auto-update to a new version of this workflow.',
  arg: 'update:workflow',
  match:
      'Auto-Update available! Press <enter> to auto-update to a new version of this workflow.',
  icon: AlfredItemIcon(path: 'alfredhatcog.png'),
  valid: true,
);

void _showPlaceholder() {
  _workflow.addItem(
    const AlfredItem(
      title: 'Search the Flutter docs...',
      icon: AlfredItemIcon(path: 'icon.png'),
    ),
  );
}

Future<void> _performSearch(String query) async {
  try {
    final SearchResponse searchResponse = await AlgoliaSearch.query(query);

    if ((searchResponse.nbHits ?? 0) > 0) {
      final AlfredItems items = AlfredItems(
        searchResponse.hits
            .map((Hit hit) => SearchResult.fromJson(
                <String, dynamic>{...hit, 'objectID': hit.objectID}))
            .map(
              (SearchResult result) => AlfredItem(
                uid: result.objectID,
                title: '${result.name} ${result.type}',
                subtitle: result.enclosedBy != null
                    ? 'from ${result.enclosedBy!["name"]}'
                    : '',
                arg: result.href,
                text: AlfredItemText(
                  copy: result.href,
                  largeType: result.qualifiedName,
                ),
                quickLookUrl: result.href,
                icon: AlfredItemIcon(path: 'icon.png'),
                valid: true,
              ),
            )
            .toList(),
      );
      _workflow.addItems(items.items);
    } else {
      final Uri url = Uri.https(
        'www.google.com',
        '/search',
        {'q': 'flutter $query'},
      );

      _workflow.addItem(
        AlfredItem(
          title: 'No matching answers found',
          subtitle: 'Shall I try and search Google?',
          arg: url.toString(),
          text: AlfredItemText(
            copy: url.toString(),
          ),
          quickLookUrl: url.toString(),
          icon: AlfredItemIcon(path: 'google.png'),
          valid: true,
        ),
      );
    }
  } finally {
    AlgoliaSearch.dispose();
  }
}
