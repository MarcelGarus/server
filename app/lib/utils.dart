import 'package:dartx/dartx.dart';

extension NextOnEndlessIterator<T> on Iterator<T> {
  T next() {
    assert(this.moveNext());
    return current;
  }
}

enum UserAgentType { uptimeMonitoring, bot, debugging, human }

class UserAgentInfo {
  final String userAgent;
  final UserAgentType type;
  final String? simpleName;

  UserAgentInfo({
    required this.userAgent,
    required this.type,
    required this.simpleName,
  });

  factory UserAgentInfo.from(String userAgent) {
    return UserAgentInfo(
      userAgent: userAgent,
      type: _typeFor(userAgent),
      simpleName: _simpleNameFor(userAgent),
    );
  }

  static UserAgentType _typeFor(String userAgent) {
    userAgent = userAgent.toLowerCase();
    final keywords = {
      'statuscake': UserAgentType.uptimeMonitoring,
      'bot': UserAgentType.bot,
      'research': UserAgentType.bot,
      'inspect': UserAgentType.bot,
      'python': UserAgentType.bot,
      'dart': UserAgentType.bot,
      'google-adstxt': UserAgentType.bot,
      'companionapp': UserAgentType.debugging,
      'postmanruntime': UserAgentType.debugging,
    };

    for (final entry in keywords.entries) {
      if (userAgent.contains(entry.key)) return entry.value;
    }
    return UserAgentType.human;
  }

  static String? _simpleNameFor(String userAgent) {
    userAgent = userAgent.toLowerCase();
    final botKeywords = {
      ['statuscake']: 'StatusCake',
      ['postmanruntime']: 'Postman',
      ['companionapp']: '(this companion app)',
      ['dotbot']: 'DotBot',
      ['petalbot']: 'PetalBot',
      ['bingbot']: 'BingBot',
      ['googlebot', 'nexus 5x']: 'GoogleBot (mobile)',
      ['googlebot']: 'GoogleBot',
      ['google-adstxt']: 'Google AdsTxt',
      ['netsystemsresearch']: 'NetSystemsResearch',
      ['censys']: 'Censys',
    };

    for (final entry in botKeywords.entries) {
      if (entry.key.every(userAgent.contains)) return entry.value;
    }

    final deviceKeywords = {
      'hm note 1w': 'Xiaomi HM Note 1W',
      'oneplus a6': 'OnePlus 6',
      'ipad': 'iPad',
    };
    final device = deviceKeywords.entries
        .firstOrNullWhere((it) => userAgent.contains(it.key))
        ?.value;

    final osKeywords = {
      'windows nt 6.1': 'Windows 7',
      'windows nt 6.2': 'Windows 8',
      'windows nt 10.0': 'Windows 10',
      'android 4': 'Android 4',
      'android 5': 'Android 5',
      'android 6': 'Android 6',
      'android 7': 'Android 7',
      'android 8': 'Android 8',
      'android 9': 'Android 9',
      'android 10': 'Android 10',
      'android 11': 'Android 11',
      'android 12': 'Android 12',
      'ubuntu': 'Ubuntu',
      'linux': 'Linux',
      'intel mac os x': 'MacOS or iPadOS',
    };
    final os = osKeywords.entries
        .firstOrNullWhere((it) => userAgent.contains(it.key))
        ?.value;

    final browserKeywords = {
      'ucbrowser': 'UC Browser',
      'chrome': 'Chrome',
      'firefox': 'Firefox',
      'applewebkit': 'WebKit',
    };
    final browser = browserKeywords.entries
        .firstOrNullWhere((it) => userAgent.contains(it.key))
        ?.value;

    final extraKeywords = {
      'page-preview-tool': 'page preview tool',
      'x11': 'X11',
    };
    final extra = extraKeywords.entries
        .where((it) => userAgent.contains(it.key))
        .map((it) => it.value)
        .join(', ');

    if (browser == null && os == null && device == null && extra.isBlank)
      return null;

    final buffer = StringBuffer(browser ?? 'Some browser');
    if (os != null) buffer.write(' on $os');
    if (device != null) buffer.write(' on $device');
    if (extra.isNotBlank) buffer.write(' ($extra)');
    return buffer.toString();
  }
}
