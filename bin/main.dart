import 'dart:io' show Platform, exitCode, stdout;

import 'package:alfred_workflow/alfred_workflow.dart'
    show
        AlfredItem,
        AlfredItemIcon,
        AlfredItemText,
        AlfredItems,
        AlfredWorkflow;
import 'package:algolia/algolia.dart' show AlgoliaQuerySnapshot;
import 'package:args/args.dart' show ArgParser, ArgResults;
import 'package:path/path.dart' show dirname;
import 'package:stash/stash_api.dart'
    show
        Cache,
        CacheEntryCreatedEvent,
        CacheEntryEvictedEvent,
        CacheEntryExpiredEvent,
        CacheEntryRemovedEvent,
        CacheEntryUpdatedEvent,
        CreatedExpiryPolicy,
        EventListenerMode,
        LruEvictionPolicy,
        CacheExtension;
import 'package:stash_file/stash_file.dart'
    show FileCacheStore, newFileLocalCacheStore;

import 'src/extensions/string_helpers.dart' show StringHelpers;
import 'src/models/search_result.dart' show SearchResult;
import 'src/services/algolia_search.dart' show AlgoliaSearch;

final AlfredWorkflow workflow = AlfredWorkflow();

final FileCacheStore store = newFileLocalCacheStore(
  path: dirname(Platform.script.toFilePath()),
  fromEncodable: (Map<String, dynamic> json) => AlfredItems.fromJson(json),
);

final Cache cache = store.cache<AlfredItems>(
  name: 'query_cache',
  maxEntries: 10,
  eventListenerMode: EventListenerMode.synchronous,
  evictionPolicy: const LruEvictionPolicy(),
  expiryPolicy: const CreatedExpiryPolicy(Duration(minutes: 1)),
);

bool verbose = false;

void main(List<String> arguments) async {
  try {
    exitCode = 0;

    workflow.clearItems();

    final ArgParser parser = ArgParser()
      ..addOption('query', abbr: 'q', mandatory: true)
      ..addFlag('verbose', abbr: 'v', defaultsTo: false);
    final ArgResults args = parser.parse(arguments);

    verbose = args['verbose'];

    final String query = args['query'].replaceAll(RegExp(r'\s+'), ' ').trim();

    if (verbose) {
      stdout.writeln('Query: "$query"');
      _cacheVerbosity();
    }

    if (query.isEmpty) {
      _showPlaceholder();
    } else {
      final AlfredItems? cachedItem = await cache.get(query.md5hex);
      if (cachedItem != null) {
        workflow.addItems(cachedItem.items);
      } else {
        await _performSearch(query);
      }
    }
  } on FormatException catch (err) {
    exitCode = 2;
    workflow.addItem(AlfredItem(title: err.toString()));
  } catch (err) {
    exitCode = 1;
    workflow.addItem(AlfredItem(title: err.toString()));
    if (verbose) {
      rethrow;
    }
  } finally {
    workflow.run();
  }
}

void _cacheVerbosity() {
  cache
    ..on<CacheEntryCreatedEvent<AlfredItems>>()
        .listen((event) => print('Key "${event.entry.key}" added'))
    ..on<CacheEntryUpdatedEvent<AlfredItems>>()
        .listen((event) => print('Key "${event.newEntry.key}" updated'))
    ..on<CacheEntryRemovedEvent<AlfredItems>>()
        .listen((event) => print('Key "${event.entry.key}" removed'))
    ..on<CacheEntryExpiredEvent<AlfredItems>>()
        .listen((event) => print('Key "${event.entry.key}" expired'))
    ..on<CacheEntryEvictedEvent<AlfredItems>>()
        .listen((event) => print('Key "${event.entry.key}" evicted'));
}

void _showPlaceholder() {
  workflow.addItem(
    const AlfredItem(
      title: 'Search the Flutter docs...',
      icon: AlfredItemIcon(path: 'icon.png'),
    ),
  );
}

Future<void> _performSearch(String query) async {
  final AlgoliaQuerySnapshot snapshot = await AlgoliaSearch.query(query);

  if (snapshot.nbHits > 0) {
    final AlfredItems items = AlfredItems(
      items: snapshot.hits
          .map(
            (snapshot) => SearchResult.fromJson(snapshot.data),
          )
          .map(
            (result) => AlfredItem(
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
    cache.putIfAbsent(query.md5hex, items);
    workflow.addItems(items.items);
  } else {
    final Uri url = Uri.https(
      'www.google.com',
      '/search',
      {'q': 'flutter $query'},
    );

    workflow.addItem(
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
}
