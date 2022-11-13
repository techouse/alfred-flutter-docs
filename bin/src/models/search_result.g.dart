// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'search_result.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

SearchResult _$SearchResultFromJson(Map<String, dynamic> json) => SearchResult(
      objectID: json['objectID'] as String,
      name: json['name'] as String,
      qualifiedName: json['qualifiedName'] as String,
      href: json['href'] as String,
      type: json['type'] as String,
      enclosedBy: (json['enclosedBy'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, e as String?),
      ),
    );

Map<String, dynamic> _$SearchResultToJson(SearchResult instance) =>
    <String, dynamic>{
      'objectID': instance.objectID,
      'name': instance.name,
      'qualifiedName': instance.qualifiedName,
      'href': instance.href,
      'type': instance.type,
      'enclosedBy': instance.enclosedBy,
    };
