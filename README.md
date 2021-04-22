# GB READER

![image](https://user-images.githubusercontent.com/6854255/115110761-fb5deb80-9fb7-11eb-87bf-c9b518f388b8.png)

CUBIC STYLEさんが制作されている、[ラズパイアドバンス拡張ボード](https://cubic-style.jp/rpa_exp/)を使用して、GBのROMを読み出すツールです。  

## インストール

https://github.com/mj-hd/gb-reader/releases リリースタブから、自分のRaspberry Piに適したバイナリファイルをダウンロードしてください。

## ビルド

### Raspberry Pi上でビルド

```sh
$ git clone https://github.com/mj-hd/gb-reader.git
$ cd gb-reader
$ cargo build --release
```

### クロスコンパイル

[cross](https://github.com/rust-embedded/cross) をインストールします。
その後、

```sh
$ git clone https://github.com/mj-hd/gb-reader.git
$ cd gb-reader
$ cross build --release --target=<トリプル>
```

を実行してください。  
  
トリプルの例)  
Raspberry Pi Zero W: `arm-unknown-linux-musleabi`  

## 使用方法

Raspberry Piの設定から、SPIを有効にしてください。  
その後、以下のコマンドを打つことでROMの読み出しが開始します。  

```sh
$ gb-reader read --output ファイル名.gb
```

## 対応MBC

- RomOnly
- MBC1
- MBC2(動作未検証)
- MBC3(動作未検証)
- MBC5(動作未検証)

MBC2, 3, 5は検証できておらず、動作しない可能性が高いです。PR大歓迎です。

## トラブルシューティング

- ROMの検証で失敗する => カードリッジの接触不良です。差し込み直してください
