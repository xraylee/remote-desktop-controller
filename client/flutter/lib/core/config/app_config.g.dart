// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'app_config.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$AppConfigImpl _$$AppConfigImplFromJson(Map<String, dynamic> json) =>
    _$AppConfigImpl(
      signalingServerUrl:
          json['signalingServerUrl'] as String? ?? 'wss://localhost:8443',
      apiServerUrl: json['apiServerUrl'] as String? ?? 'http://localhost:8080',
      autoConnect: json['autoConnect'] as bool? ?? false,
      showNotifications: json['showNotifications'] as bool? ?? true,
      theme: json['theme'] as String? ?? 'system',
      language: json['language'] as String? ?? 'system',
      qualityMode: (json['qualityMode'] as num?)?.toInt() ?? 1,
      maxBitrate: (json['maxBitrate'] as num?)?.toInt() ?? 5000,
      enableHardwareAcceleration:
          json['enableHardwareAcceleration'] as bool? ?? true,
    );

Map<String, dynamic> _$$AppConfigImplToJson(_$AppConfigImpl instance) =>
    <String, dynamic>{
      'signalingServerUrl': instance.signalingServerUrl,
      'apiServerUrl': instance.apiServerUrl,
      'autoConnect': instance.autoConnect,
      'showNotifications': instance.showNotifications,
      'theme': instance.theme,
      'language': instance.language,
      'qualityMode': instance.qualityMode,
      'maxBitrate': instance.maxBitrate,
      'enableHardwareAcceleration': instance.enableHardwareAcceleration,
    };
