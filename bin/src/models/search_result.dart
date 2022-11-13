import 'package:json_annotation/json_annotation.dart';

part 'search_result.g.dart';

@JsonSerializable()
class SearchResult {
  const SearchResult({
    required this.objectID,
    required this.name,
    required this.qualifiedName,
    required String href,
    required this.type,
    required this.enclosedBy,
  }) : href = 'https://api.flutter.dev/flutter/$href';

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

  factory SearchResult.fromJson(Map<String, dynamic> json) =>
      _$SearchResultFromJson(json);

  Map<String, dynamic> toJson() => _$SearchResultToJson(this);
}
