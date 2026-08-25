# roland-rs

[![CI](https://github.com/FlowingSPDG/roland-rs/workflows/CI/badge.svg)](https://github.com/FlowingSPDG/roland-rs/actions)
[![crates.io](https://img.shields.io/crates/v/roland-rs.svg)](https://crates.io/crates/roland-rs)
[![docs.rs](https://docs.rs/roland-rs/badge.svg)](https://docs.rs/roland-rs)

Roland VR-6HD / V-160HD リモートコントロール用のRustライブラリ

## 概要

このプロジェクトは、Roland VR-6HD および V-160HD のリモートコントロール機能をRustで実装したものです。
組み込み環境での使用を想定し、コア部分を`roland-core`として独立したライブラリとして提供しています。

V-160HD のアドレスマップは公式リモート・コントロール・ガイドと [companion-module-roland-v160hd](https://github.com/bitfocus/companion-module-roland-v160hd) を参照しています。

## リンク

- [crates.io](https://crates.io/crates/roland-rs)
- [docs.rs](https://docs.rs/roland-rs)

## 公式ドキュメント

プロトコルの詳細については、以下の公式ドキュメントを参照してください：

- [VR-6HD リモート・コントロール・ガイド](https://static.roland.com/assets/media/pdf/VR-6HD_Control_jpn03_W.pdf)
- [V-160HD リモート・コントロール・ガイド](https://static.roland.com/assets/media/pdf/V-160HD_Control_jpn04_W.pdf)

V-160HD の LAN 制御は TCP ポート **8023**、4桁のネットワークパスワード、コマンド末尾の LF を使用します。

```rust
use roland_rs::devices::v160hd::{self, VideoSource};
use roland_rs::TelnetClient;

let mut client = TelnetClient::connect_v160hd("192.168.0.1", "0000")?;
client.send_command(&v160hd::select_pgm(VideoSource::hdmi(1)?))?;
client.press_and_release(v160hd::switch::CUT)?;
```

## roland-core

`roland-core`は、Roland VR-6HD / V-160HD との通信プロトコルを実装したコアライブラリです。

- **`no_std`対応**: 組み込み環境で使用可能（`alloc`が必要）
- **ゼロ外部依存**: 外部クレートに依存しない純粋なプロトコル実装
- コマンドのエンコード/デコード
- レスポンスのパース
- エラーハンドリング
- SysExアドレスの管理
- `Write`トレイトを使用したヒープ割り当て不要のエンコード機能

内部的にはTelnetプロトコルを使用してデバイスと通信します。Telnet経由での通信ではSTX（0x02）は省略され、RS-232経由の場合はSTXが必要です。

詳細な使用方法やAPIについては、公式ドキュメントとソースコードを参照してください。

## 免責事項

このプロジェクトは、Roland Corporationとは無関係の第三者によって開発・提供されています。
Rolandの公式プロジェクトではありません。

## ライセンス

MIT License

Copyright (c) 2026 Shugo Kawamura
