# リポジトリ構造定義書

## 1. 全体構造

```
taskctl/
├── .github/
│   └── workflows/
│       ├── ci.yml              # PR / push 時の CI（fmt, clippy, test）
│       └── release.yml         # タグプッシュ時のリリースビルド
├── docs/
│   ├── product-requirements.md # プロダクト要求定義書
│   ├── functional-design.md    # 機能設計書
│   ├── architecture.md         # 技術仕様書
│   ├── repository-structure.md # リポジトリ構造定義書（本ファイル）
│   ├── development-guidelines.md # 開発ガイドライン
│   └── glossary.md             # ユビキタス言語定義
├── src/
│   ├── main.rs                 # エントリーポイント
│   ├── cli/                    # CLI Layer
│   │   ├── mod.rs
│   │   ├── args.rs             # clap の引数定義
│   │   ├── output.rs           # 出力フォーマッター
│   │   └── commands/           # サブコマンド実装
│   │       ├── mod.rs
│   │       ├── add.rs
│   │       ├── show.rs
│   │       ├── list.rs
│   │       ├── edit.rs
│   │       ├── delete.rs
│   │       ├── search.rs
│   │       ├── status.rs       # start / done / pending
│   │       ├── pin.rs          # pin / unpin
│   │       ├── depends.rs      # depends / undepends / tree
│   │       ├── next.rs
│   │       ├── today.rs
│   │       ├── init.rs
│   │       ├── migrate.rs
│   │       └── completions.rs
│   ├── domain/                 # Domain Layer
│   │   ├── mod.rs
│   │   ├── task.rs             # Task 構造体と操作
│   │   ├── status.rs           # ステータス遷移ロジック
│   │   ├── scoring.rs          # スコアリングアルゴリズム
│   │   ├── dependency.rs       # 依存関係管理
│   │   └── date_parser.rs      # 相対日付パーサー
│   ├── storage/                # Storage Layer
│   │   ├── mod.rs
│   │   ├── markdown.rs         # Markdown 読み書き
│   │   ├── repository.rs       # タスクの永続化
│   │   ├── lock.rs             # ロックファイル管理
│   │   ├── meta.rs             # .meta.json 管理
│   │   └── migration.rs        # スキーママイグレーション
│   ├── config/                 # Config Layer
│   │   ├── mod.rs
│   │   └── settings.rs         # 設定読み込み
│   └── error.rs                # エラー型定義
├── tests/                      # 統合テスト
│   ├── cli_add_test.rs
│   ├── cli_list_test.rs
│   ├── cli_status_test.rs
│   ├── cli_scoring_test.rs
│   ├── cli_dependency_test.rs
│   ├── cli_pin_test.rs
│   ├── cli_search_test.rs
│   ├── cli_config_test.rs
│   └── helpers/
│       └── mod.rs              # テストユーティリティ
├── benches/                    # ベンチマーク
│   └── scoring_bench.rs
├── .steering/                  # 作業単位のドキュメント
├── .gitignore
├── Cargo.toml
├── Cargo.lock
├── CLAUDE.md                   # プロジェクトメモリ
├── LICENSE
└── README.md
```

## 2. ディレクトリの役割

### `src/` — ソースコード

アプリケーションの全ソースコードを格納する。技術仕様書で定義した4レイヤー（CLI / Domain / Storage / Config）に対応するサブディレクトリで構成する。

| ディレクトリ | 役割 | 外部への公開 |
|---|---|---|
| `src/cli/` | コマンドライン引数の解析、コマンド実行、出力フォーマット | なし（バイナリクレート） |
| `src/cli/commands/` | 各サブコマンドの実装。1コマンド1ファイル | なし |
| `src/domain/` | ビジネスロジック。タスク操作、スコアリング、依存関係管理 | なし |
| `src/storage/` | ファイルシステムへの永続化。Markdownパース、ロック管理 | なし |
| `src/config/` | 設定ファイルの読み込みとデフォルト値管理 | なし |

### `tests/` — 統合テスト

CLIコマンドのエンドツーエンドテストを格納する。各テストファイルは `assert_cmd` を使用してバイナリを実行し、標準出力と終了コードを検証する。

`tests/helpers/mod.rs` には一時データディレクトリの作成やテスト用タスク生成などの共通ユーティリティを配置する。

### `benches/` — ベンチマーク

`criterion` を使用したパフォーマンスベンチマーク。スコアリング計算と全タスク読み込みの性能を計測する。

### `docs/` — 永続的ドキュメント

アプリケーション全体の設計を定義する恒久的なドキュメント。基本設計や方針が変わらない限り更新しない。

### `.steering/` — 作業単位のドキュメント

特定の開発作業における要求・設計・タスクリストを格納する。作業ごとに `[YYYYMMDD]-[開発タイトル]/` ディレクトリを作成する。CLAUDE.mdの開発プロセスに従う。

### `.github/workflows/` — CI/CD

GitHub Actionsのワークフロー定義。CI（テスト・リント）とリリースビルドの2ファイル構成。

## 3. ファイル配置ルール

### 3.1 ソースコード

| ルール | 説明 |
|---|---|
| 1コマンド1ファイル | `src/cli/commands/` 内のファイルは1サブコマンドに対応。関連コマンド（start/done/pending）は1ファイルにまとめる |
| レイヤー間の依存方向 | `cli → domain → storage/config` の一方向のみ。逆方向の依存は禁止 |
| `mod.rs` の役割 | 各ディレクトリの `mod.rs` はモジュールの公開インターフェースを定義する。ロジックは含めない |
| エラー型は集約 | `src/error.rs` に全エラー型を定義する。各モジュール固有のエラーも `TaskCtlError` 内のバリアントとする |

### 3.2 テスト

| ルール | 説明 |
|---|---|
| 単体テストの配置 | 対象モジュールと同一ファイル内に `#[cfg(test)] mod tests` として記述 |
| 統合テストの配置 | `tests/` ディレクトリ内。テスト対象の機能カテゴリでファイルを分割 |
| テストデータ | `tempfile` クレートで一時ディレクトリを作成。固定のテストデータファイルは持たない |

### 3.3 ドキュメント

| ルール | 説明 |
|---|---|
| 永続ドキュメント | `docs/` に配置。アプリケーション全体の設計を記述 |
| 作業ドキュメント | `.steering/[YYYYMMDD]-[タイトル]/` に配置 |
| 図表 | ドキュメント内にMermaid記法で直接記述。独立した画像ファイルは原則作成しない |

### 3.4 設定・メタファイル

| ファイル | 役割 |
|---|---|
| `Cargo.toml` | クレート定義、依存関係、ビルド設定 |
| `Cargo.lock` | 依存関係のロックファイル。バイナリクレートのためコミット対象 |
| `.gitignore` | `target/` 等のビルド成果物を除外 |
| `CLAUDE.md` | プロジェクトメモリ。開発プロセスとルールを定義 |
| `LICENSE` | ライセンスファイル（MIT） |
| `README.md` | プロジェクト概要、インストール手順、使用方法 |

## 4. `.gitignore`

```gitignore
# ビルド成果物
/target/

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db
```
