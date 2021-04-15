# GB READER

CUBIC STYLEさんが制作されている、[ラズパイアドバンス拡張ボード](https://cubic-style.jp/rpa_exp/)を使用して、GBのROMを読み出すツールです。  

## 使用方法

Raspberry PIの設定から、SPIを有効にしてください。  
その後、以下のコマンドを打つことでROMの読み出しが開始します。  

```sh
$ gb-reader read --output ファイル名.gb
```

## TODO

- [x] RomOnly
- [-] MBC1
- [ ] MBC2
- [ ] MBC3
- [ ] MBC5
