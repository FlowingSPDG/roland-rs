# roland-rs

Roland VR-6HD リモートコントロール用のRustライブラリ

## 概要

このプロジェクトは、Roland VR-6HDのリモートコントロール機能をRustで実装したものです。
組み込み環境での使用を想定し、コア部分を`roland-core`として独立したライブラリとして提供しています。

## プロジェクト構成

```
roland-rs/
├── core/              # コアライブラリ（プロトコル実装）
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs
│   └── examples/      # 使用例
│       └── telnet_client.rs
├── Cargo.toml        # Workspace設定
└── README.md
```

## 機能

### roland-core

- **`no_std`対応**: 組み込み環境で使用可能（`alloc`が必要）
- **ゼロ外部依存**: 外部クレートに依存しない純粋なプロトコル実装
- コマンドのエンコード/デコード
- レスポンスのパース
- エラーハンドリング
- SysExアドレスの管理
- `Write`トレイトを使用したヒープ割り当て不要のエンコード機能

### サポートしているコマンド

- `DTH` - パラメーターの書き込み（SysEx互換）
- `RQH` - パラメーター値の取得（SysEx互換）
- `VER` - バージョン情報の取得

## 使用方法

### 基本的な使用例

#### ヒープ割り当てあり（`alloc`使用）

```rust
use roland_core::{Address, Command, Response};

// アドレスの作成
let address = Address::from_hex("123456")?;

// コマンドの作成
let cmd = Command::WriteParameter {
    address,
    value: 0x01,
};

// コマンドのエンコード（Telnet用、STXなし）
let encoded = cmd.encode();
// => "DTH:123456,01;"

// レスポンスのパース
let response = Response::parse("DTH:123456,01;")?;
```

#### ヒープ割り当てなし（`no_std`環境向け）

```rust
use roland_core::{Address, Command};
use core::fmt::Write;

// アドレスの作成
let address = Address::from_hex("123456")?;

// コマンドの作成
let cmd = Command::WriteParameter {
    address,
    value: 0x01,
};

// Writeトレイトを使用してエンコード（ヒープ割り当て不要）
let mut buf = heapless::String::<64>::new();
cmd.write(&mut buf)?;
// buf => "DTH:123456,01;"
```

### Telnetクライアントの使用例

```bash
# ビルド
cargo build --example telnet_client

# 実行
cargo run --example telnet_client -- 192.168.1.100
```

## 通信プロトコル

### コマンドフォーマット

```
stxコマンドコード:パラメーター;
```

- `stx`: ASCII 0x02（Telnet経由の場合は省略可能）
- コマンドコード: 3文字の英数字
- パラメーター: カンマ区切り
- 終端: `;`

### レスポンスフォーマット

- `ack` (0x06): 正常応答
- `DTH:address,value;`: データ応答
- `VER:product,version;`: バージョン情報
- `ERR:code;`: エラー応答

### エラーコード

- `0`: Syntax error
- `4`: Invalid
- `5`: Out of range error
- `6`: No stx error

## 組み込み環境での使用

`roland-core`は`no_std`対応です。組み込み環境で使用する場合：

1. `alloc`クレートが必要です（ヒープ割り当てを使用する場合）
2. `alloc`が使えない環境では、`Write`トレイトを使用したメソッド（`write()`, `write_hex()`など）を使用してください
3. 外部依存は一切ありません（`alloc`はRust標準ライブラリの一部）

### 依存関係

- **roland-core**: 外部依存なし（`alloc`のみ使用）
- **examples/telnet_client**: `std`環境用の実装例

## ライセンス

MIT License

Copyright (c) 2026 Shugo Kawamura
