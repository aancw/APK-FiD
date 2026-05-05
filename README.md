# APK-FiD
Give me your APK, I will give you framework name

## Usage

```bash
apk_fid --file app.apk
apk_fid --file app.apk --output text
apk_fid --file app.apk --output json
```

```text
Usage: apk_fid --file <FILE> [--output <OUTPUT>]

Options:
  -f, --file <FILE>      Android APK file location
      --output <OUTPUT>  Output format [possible values: text, json] [default: text]
  -h, --help             Print help
  -V, --version          Print version
```

## Detection Support

- [x] [React Native Framework](https://reactnative.dev)
- [x] [Flutter Framework](https://flutter.dev)
- [x] [Ionic](http://ionicframework.com)
- [x] [Cordova](http://cordova.apache.org)
- [x] [Capacitorjs](https://capacitorjs.com)
- [x] [Framework7](http://framework7.io)
- [x] [NativeScript Framework](https://nativescript.org)
- [x] [Unity](https://unity.com)
- [x] [Unreal Engine](https://www.unrealengine.com)
- [x] [Xamarin / .NET for Android](https://dotnet.microsoft.com/apps/xamarin)
- [x] [Cocos2d-x](https://www.cocos.com/en/cocos2dx)
- [x] Apache Weex
- [x] Qt for Android
- [x] [Godot](https://godotengine.org)
- [x] [Solar2D](https://solar2d.com)
- [x] Adobe AIR
- [x] [Appcelerator Titanium](https://titaniumsdk.com)
- [x] [Kivy](https://kivy.org)
- [x] [Defold](https://defold.com)

Detection now uses weighted multi-signal matching and reports confidence percentage.

## License

MIT License
