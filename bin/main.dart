import 'dart:io' show exitCode, stdout;

import 'package:alfred_workflow/alfred_workflow.dart'
    show AlfredItem, AlfredItemIcon, AlfredItemText, AlfredWorkflow;
import 'package:algolia/algolia.dart' show AlgoliaQuerySnapshot;
import 'package:args/args.dart' show ArgParser, ArgResults;

import 'src/models/search_result.dart';
import 'src/services/algolia_search.dart';

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
    workflow.addItems(
      snapshot.hits
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
          ),
    );
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

final AlfredWorkflow workflow = AlfredWorkflow();
bool verbose = false;

void main(List<String> arguments) async {
  try {
    exitCode = 0;

    workflow.clearItems();

    final ArgParser parser = ArgParser()
      ..addOption('query', abbr: 'q', mandatory: true)
      ..addFlag('verbose', abbr: 'v', defaultsTo: false);

    final ArgResults args = parser.parse(arguments);

    final String query = args['query'].replaceAll(RegExp(r'\s+'), ' ').trim();

    if (args['verbose']) {
      verbose = true;
    }

    if (verbose) {
      stdout.writeln('Query: "$query"');
    }

    if (query.isEmpty) {
      _showPlaceholder();
    } else {
      await _performSearch(query);
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
