// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'config_model.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$ServerConfigImpl _$$ServerConfigImplFromJson(Map<String, dynamic> json) =>
    _$ServerConfigImpl(
      rendezvousUrl: json['rendezvousUrl'] as String? ?? '',
      relayUrl: json['relayUrl'] as String? ?? '',
      apiUrl: json['apiUrl'] as String? ?? '',
      requireTls: json['requireTls'] as bool? ?? true,
    );

Map<String, dynamic> _$$ServerConfigImplToJson(_$ServerConfigImpl instance) =>
    <String, dynamic>{
      'rendezvousUrl': instance.rendezvousUrl,
      'relayUrl': instance.relayUrl,
      'apiUrl': instance.apiUrl,
      'requireTls': instance.requireTls,
    };

_$QualityConfigImpl _$$QualityConfigImplFromJson(Map<String, dynamic> json) =>
    _$QualityConfigImpl(
      codec: json['codec'] as String? ?? 'h264',
      maxFps: (json['maxFps'] as num?)?.toInt() ?? 60,
      maxWidth: (json['maxWidth'] as num?)?.toInt() ?? 0,
      maxHeight: (json['maxHeight'] as num?)?.toInt() ?? 0,
      bitrateKbps: (json['bitrateKbps'] as num?)?.toInt() ?? 0,
      hardwareAccel: json['hardwareAccel'] as bool? ?? true,
    );

Map<String, dynamic> _$$QualityConfigImplToJson(_$QualityConfigImpl instance) =>
    <String, dynamic>{
      'codec': instance.codec,
      'maxFps': instance.maxFps,
      'maxWidth': instance.maxWidth,
      'maxHeight': instance.maxHeight,
      'bitrateKbps': instance.bitrateKbps,
      'hardwareAccel': instance.hardwareAccel,
    };

_$GeneralConfigImpl _$$GeneralConfigImplFromJson(Map<String, dynamic> json) =>
    _$GeneralConfigImpl(
      locale: json['locale'] as String? ?? 'zh-CN',
      startMinimized: json['startMinimized'] as bool? ?? false,
      autoReconnect: json['autoReconnect'] as bool? ?? false,
      syncClipboard: json['syncClipboard'] as bool? ?? true,
      enableRemoteAudio: json['enableRemoteAudio'] as bool? ?? true,
    );

Map<String, dynamic> _$$GeneralConfigImplToJson(_$GeneralConfigImpl instance) =>
    <String, dynamic>{
      'locale': instance.locale,
      'startMinimized': instance.startMinimized,
      'autoReconnect': instance.autoReconnect,
      'syncClipboard': instance.syncClipboard,
      'enableRemoteAudio': instance.enableRemoteAudio,
    };

_$RdcsConfigImpl _$$RdcsConfigImplFromJson(Map<String, dynamic> json) =>
    _$RdcsConfigImpl(
      server: json['server'] == null
          ? const ServerConfig()
          : ServerConfig.fromJson(json['server'] as Map<String, dynamic>),
      quality: json['quality'] == null
          ? const QualityConfig()
          : QualityConfig.fromJson(json['quality'] as Map<String, dynamic>),
      general: json['general'] == null
          ? const GeneralConfig()
          : GeneralConfig.fromJson(json['general'] as Map<String, dynamic>),
      deviceCode: json['deviceCode'] as String? ?? '',
      deviceName: json['deviceName'] as String? ?? '',
    );

Map<String, dynamic> _$$RdcsConfigImplToJson(_$RdcsConfigImpl instance) =>
    <String, dynamic>{
      'server': instance.server,
      'quality': instance.quality,
      'general': instance.general,
      'deviceCode': instance.deviceCode,
      'deviceName': instance.deviceName,
    };
