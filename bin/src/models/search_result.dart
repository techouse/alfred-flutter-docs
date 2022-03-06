class SearchResult {
  const SearchResult({
    required this.objectID,
    required this.name,
    required this.qualifiedName,
    required this.href,
    required this.type,
    required this.enclosedBy,
  });

  final String objectID;
  final String name;
  final String qualifiedName;
  final String href;
  final String type;
  final Map<String, String?>? enclosedBy;

  static const List<String> attributesToRetrieve = [
    'name',
    'qualifiedName',
    'href',
    'type',
    'enclosedBy',
  ];

  SearchResult.fromJson(Map<String, dynamic> json)
      : objectID = json['objectID'] as String,
        name = json['name'] as String,
        qualifiedName = json['qualifiedName'] as String,
        href = 'https://api.flutter.dev/flutter/${json['href']}',
        type = json['type'] as String,
        enclosedBy = json['enclosedBy'] != null
            ? (json['enclosedBy'] as Map<String, dynamic>).map(
                (key, value) => MapEntry(
                  key,
                  value?.toString(),
                ),
              )
            : null;

  Map<String, dynamic> toJson() => {
        'objectID': objectID,
        'name': name,
        'qualifiedName': qualifiedName,
        'href': href,
        'type': type,
        'enclosedBy': enclosedBy,
      };
}
