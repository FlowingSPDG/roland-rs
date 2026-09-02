# roland-rs

[![CI](https://github.com/FlowingSPDG/roland-rs/workflows/CI/badge.svg)](https://github.com/FlowingSPDG/roland-rs/actions)
[![crates.io](https://img.shields.io/crates/v/roland-rs.svg)](https://crates.io/crates/roland-rs)
[![docs.rs](https://docs.rs/roland-rs/badge.svg)](https://docs.rs/roland-rs)

Roland VR-6HD / V-160HD / V-60HD リモートコントロール用のRustライブラリ

## 概要

このプロジェクトは、Roland VR-6HD、V-160HD、V-60HD のリモートコントロール機能をRustで実装したものです。
組み込み環境での使用を想定し、コア部分を`roland-core`として独立したライブラリとして提供しています。

V-160HD のアドレスマップは公式リモート・コントロール・ガイドと [companion-module-roland-v160hd](https://github.com/bitfocus/companion-module-roland-v160hd) を参照しています。
V-60HD のコマンド表は公式リファレンスと [companion-module-roland-v60hd](https://github.com/bitfocus/companion-module-roland-v60hd) を参照しています。V-60HD は DTH/RQH ではなく、3文字オペコードを STX でフレームする別プロトコルです。

## リンク

- [crates.io](https://crates.io/crates/roland-rs)
- [docs.rs](https://docs.rs/roland-rs)

## 公式ドキュメント

プロトコルの詳細については、以下の公式ドキュメントを参照してください：

- [VR-6HD リモート・コントロール・ガイド](https://static.roland.com/assets/media/pdf/VR-6HD_Control_jpn03_W.pdf)
- [V-160HD リモート・コントロール・ガイド](https://static.roland.com/assets/media/pdf/V-160HD_Control_jpn04_W.pdf)
- [V-60HD Reference Manual (LAN / RS-232)](https://static.roland.com/assets/media/pdf/V-60HD_reference_v31_eng04_W.pdf)

V-160HD の LAN 制御は TCP ポート **8023**、4桁のネットワークパスワード、コマンド末尾の LF を使用します。STX は Telnet では省略され、RS-232 では必要です。

```rust
use roland_rs::devices::v160hd::{self, VideoSource};
use roland_rs::TelnetClient;

let mut client = TelnetClient::connect_v160hd("192.168.0.1", "0000")?;
client.send_command(&v160hd::select_pgm(VideoSource::hdmi(1)?))?;
client.press_and_release(v160hd::switch::CUT)?;
```

V-60HD の LAN 制御は同じ TCP ポート **8023** ですが、パスワードはなく、**すべてのコマンドに STX (0x02)** が必要です。応答の ACK (`0x06`) を待ってから次コマンドを送ってください。本体は同時に **1 本の TCP 接続** しか受け付けません（V-60HD RCS と併用不可）。

```rust
use roland_rs::devices::v60hd::{self, Channel};
use roland_rs::V60HdClient;

let mut client = V60HdClient::connect("192.168.2.254")?;
let (product, version) = client.ver()?;
client.send(&v60hd::pgm(Channel::Sdi1))?;
client.send(&v60hd::cut())?;
```

Firmware 3.02 (LAN) notes from hardware check:

- Wait for ACK, then drain a possible **duplicate ACK** before the next command.
- Poll `TLY;` (8 lamps: Red=PGM, Green=PST) and `QPL:7` (PGM/PST/AUX/PinP-SPLIT/DSK/OUTPUT FADE). Unsolicited TLY/QPL was not observed.
- `ACS` did not return ACK or a payload within 2s — do not block a command queue on it.
- Out-of-range `PGM:99` returns `ERR` (parameter out of range) once leftover ACKs are drained.

## roland-core

`roland-core`は、Roland VR-6HD / V-160HD / V-60HD との通信プロトコルを実装したコアライブラリです。

- **`no_std`対応**: 組み込み環境で使用可能（`alloc`が必要）
- **ゼロ外部依存**: 外部クレートに依存しない純粋なプロトコル実装
- コマンドのエンコード/デコード
- レスポンスのパース
- エラーハンドリング
- SysExアドレスの管理（VR-6HD / V-160HD）
- V-60HD の STX 付き 3 文字オペコード
- `Write`トレイトを使用したヒープ割り当て不要のエンコード機能（DTH/RQH）

VR-6HD / V-160HD では、Telnet 経由の DTH/RQH は STX（0x02）を省略し、RS-232 では STX が必要です。V-60HD はこの例外で、LAN でも STX が必須です。

詳細な使用方法やAPIについては、公式ドキュメントとソースコードを参照してください。

## 免責事項

このプロジェクトは、Roland Corporationとは無関係の第三者によって開発・提供されています。
Rolandの公式プロジェクトではありません。

## ライセンス

MIT License

Copyright (c) 2026 Shugo Kawamura
