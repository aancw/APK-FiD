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

Detection now uses weighted multi-signal matching and reports confidence percentage.

## License

MIT License
