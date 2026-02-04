# 技術仕様書

## 1. テクノロジースタック

### 言語

- **Rust** (edition 2021, MSRV: 1.75.0)
- 選定理由: CLI起動速度（100ms以下要件）、シングルバイナリ配布、クロスプラットフォーム対応

### 主要クレート

| クレート | 用途 | 選定理由 |
|---|---|---|
| `clap` (v4) | コマンドライン引数解析 | derive マクロによる宣言的定義、サブコマンド対応、シェル補完生成機能 |
| `serde` + `serde_yaml` | YAML Front Matter のシリアライズ/デシリアライズ | Rust のデファクトスタンダード |
| `serde_json` | JSON出力 | `--json` フラグ対応 |
| `toml` | 設定ファイル読み込み | TOML形式の設定ファイルパース |
| `chrono` | 日付・時刻操作 | ISO 8601対応、相対日付の算出 |
| `colored` | ターミナルカラー出力 | シンプルなAPI、`NO_COLOR` 環境変数対応 |
| `fs2` | ファイルロック | クロスプラットフォームなアドバイザリロック（`flock`相当） |
| `dirs` | 標準ディレクトリ取得 | XDG Base Directory準拠（`~/.config`, `~/.local/share`） |
| `thiserror` | エラー型定義 | derive マクロによる簡潔なエラー型定義 |
| `anyhow` | エラーハンドリング | CLI層でのエラーチェイン表示 |
| `regex` | テキスト検索 | `task search` のパターンマッチング |

### 開発用クレート

| クレート | 用途 |
|---|---|
| `assert_cmd` | CLIの統合テスト |
| `predicates` | テストのアサーション |
| `tempfile` | テスト用の一時ディレクトリ |
| `criterion` | ベンチマークテスト |

## 2. アプリケーションアーキテクチャ

### 2.1 モジュール構成

```
src/
├── main.rs              # エントリーポイント
├── cli/                 # CLI Layer
│   ├── mod.rs
│   ├── commands/        # サブコマンド定義
│   │   ├── mod.rs
│   │   ├── add.rs
│   │   ├── show.rs
│   │   ├── list.rs
│   │   ├── edit.rs
│   │   ├── delete.rs
│   │   ├── search.rs
│   │   ├── status.rs   # start / done / pending
│   │   ├── pin.rs      # pin / unpin
│   │   ├── depends.rs  # depends / undepends / tree
│   │   ├── next.rs
│   │   ├── today.rs
│   │   ├── init.rs
│   │   ├── migrate.rs
│   │   └── completions.rs
│   ├── args.rs          # clap の引数定義
│   └── output.rs        # 出力フォーマッター（カラー / JSON / プレーン）
├── domain/              # Domain Layer
│   ├── mod.rs
│   ├── task.rs          # Task 構造体と操作
│   ├── status.rs        # ステータス遷移ロジック
│   ├── scoring.rs       # スコアリングアルゴリズム
│   ├── dependency.rs    # 依存関係管理（循環検出含む）
│   └── date_parser.rs   # 相対日付・曜日パーサー
├── storage/             # Storage Layer
│   ├── mod.rs
│   ├── markdown.rs      # Markdownファイルの読み書き（Front Matter パース）
│   ├── repository.rs    # タスクの永続化（CRUD操作）
│   ├── lock.rs          # ロックファイル管理
│   ├── meta.rs          # .meta.json の読み書き
│   └── migration.rs     # スキーママイグレーション
├── config/              # Config Layer
│   ├── mod.rs
│   └── settings.rs      # 設定ファイル読み込みとデフォルト値
└── error.rs             # エラー型定義
```

### 2.2 レイヤー間のデータフロー

```
ユーザー入力
    │
    ▼
CLI Layer (args.rs)
    │  コマンドライン引数を解析し、構造体に変換
    ▼
CLI Layer (commands/*.rs)
    │  Domain Layer の関数を呼び出す
    ▼
Domain Layer (task.rs / scoring.rs / dependency.rs)
    │  ビジネスロジックを実行
    │  Config Layer から設定値を取得
    │  Storage Layer にデータ読み書きを委譲
    ▼
Storage Layer (repository.rs / markdown.rs)
    │  Markdownファイルへの永続化
    ▼
CLI Layer (output.rs)
    │  結果をフォーマットして出力
    ▼
標準出力
```

### 2.3 主要な型定義

```rust
// domain/task.rs
pub struct Task {
    pub id: u32,
    pub title: String,
    pub status: Status,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub due: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub estimate: Option<Estimate>,
    pub depends_on: Vec<u32>,
    pub pinned: bool,
    pub pinned_at: Option<DateTime<Local>>,
    pub schema_version: u32,
    pub note: String,
}

// domain/status.rs
pub enum Status {
    Pending,
    InProgress,
    Done,
}

// domain/task.rs
pub enum Estimate {
    Minutes(u32),    // 30m → Minutes(30)
    Hours(f32),      // 2h  → Hours(2.0)
    Points(f32),     // 3p  → Points(3.0)
}

// domain/scoring.rs
pub struct ScoreResult {
    pub score: f64,
    pub primary_factors: Vec<String>,
}
```

## 3. ファイルI/O設計

### 3.1 Markdownファイルの読み込み

```
ファイル読み込み (fs::read_to_string)
    │
    ▼
Front Matter 分離 ("---" で囲まれた部分を抽出)
    │
    ▼
YAML デシリアライズ (serde_yaml::from_str → Task構造体のFront Matter部分)
    │
    ▼
Markdown Body 抽出 (Front Matter 以降のテキスト → note フィールド)
    │
    ▼
Task 構造体を返却
```

### 3.2 Markdownファイルの書き込み

```
Task 構造体を受け取る
    │
    ▼
Front Matter を YAML シリアライズ (serde_yaml::to_string)
    │
    ▼
"---\n" + YAML + "---\n\n" + note の Markdown を組み立て
    │
    ▼
ロック取得 (lock.rs)
    │
    ▼
ファイルに書き込み (fs::write)
    │
    ▼
ロック解放
```

### 3.3 全タスク読み込みの最適化

`task list` / `task next` / `task today` では全タスクを読み込む必要がある。

- データディレクトリ内の `*.md` ファイルを列挙する（`.lock`, `.meta.json` は除外）
- 各ファイルを読み込み、Front Matterをパースする
- パースに失敗したファイルは警告を出力してスキップする
- 1000タスク以内なら逐次読み込みで500ms以下の要件を満たせる見込み（Rustのファイルシステム操作は高速）
- パフォーマンスが問題になった場合の将来策: Front Matterのみを読み込むストリーミングパーサーの導入

## 4. エラーハンドリング設計

### 4.1 エラー型の階層

```rust
// error.rs
#[derive(thiserror::Error, Debug)]
pub enum TaskCtlError {
    // 入力エラー (終了コード: 1)
    #[error("タスク #{0} は存在しません")]
    TaskNotFound(u32),

    #[error("不正な引数: {0}")]
    InvalidArgument(String),

    #[error("循環依存が発生します ({0})")]
    CyclicDependency(String),

    #[error("タスクは自分自身に依存できません (#{0})")]
    SelfDependency(u32),

    // データエラー (終了コード: 2)
    #[error("ファイルのパースに失敗しました: {path}")]
    ParseError { path: String, source: anyhow::Error },

    #[error("スキーマバージョンが一致しません (期待: {expected}, 実際: {actual})")]
    SchemaMismatch { expected: u32, actual: u32 },

    // ロックエラー (終了コード: 3)
    #[error("ロックファイルの取得に失敗しました")]
    LockError(#[source] std::io::Error),

    // 設定エラー (終了コード: 4)
    #[error("設定ファイルの読み込みに失敗しました: {0}")]
    ConfigError(String),

    // IOエラー
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

### 4.2 終了コードのマッピング

```rust
impl TaskCtlError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::TaskNotFound(_)
            | Self::InvalidArgument(_)
            | Self::CyclicDependency(_)
            | Self::SelfDependency(_) => 1,

            Self::ParseError { .. }
            | Self::SchemaMismatch { .. } => 2,

            Self::LockError(_) => 3,

            Self::ConfigError(_) => 4,

            Self::Io(_) => 1,
        }
    }
}
```

## 5. 設定ファイルの解決

### 5.1 解決順序

```
1. コマンドライン引数 (--config / --data-dir)
    │  指定あり → 使用
    │  指定なし ↓
2. 環境変数 (TASKCTL_CONFIG / TASKCTL_DATA_DIR)
    │  設定あり → 使用
    │  設定なし ↓
3. 設定ファイル (~/.config/taskctl/config.toml)
    │  存在する → 読み込み
    │  存在しない ↓
4. デフォルト値を使用
```

### 5.2 設定の型定義

```rust
// config/settings.rs
#[derive(Deserialize)]
pub struct Config {
    pub priority: PriorityConfig,
    pub estimate: EstimateConfig,
    pub display: DisplayConfig,
    pub data: DataConfig,
}

#[derive(Deserialize)]
pub struct PriorityConfig {
    pub weights: Weights,
}

#[derive(Deserialize)]
pub struct Weights {
    pub urgency: f64,    // デフォルト: 1.0
    pub blocking: f64,   // デフォルト: 0.8
    pub staleness: f64,  // デフォルト: 0.5
    pub quick_win: f64,  // デフォルト: 0.3
}

#[derive(Deserialize)]
pub struct EstimateConfig {
    pub point_to_hours: f64,  // デフォルト: 1.0
}

#[derive(Deserialize)]
pub struct DisplayConfig {
    pub color: bool,           // デフォルト: true
    pub date_format: String,   // デフォルト: "%Y-%m-%d"
}

#[derive(Deserialize)]
pub struct DataConfig {
    pub directory: String,  // デフォルト: "~/.local/share/taskctl"
}
```

## 6. テスト戦略

### 6.1 テストレベル

| レベル | 対象 | ツール | 実行方法 |
|---|---|---|---|
| 単体テスト | 各モジュールの関数 | `#[cfg(test)]` | `cargo test` |
| 統合テスト | CLIコマンドのエンドツーエンド | `assert_cmd` + `tempfile` | `cargo test --test '*'` |
| ベンチマーク | パフォーマンス要件の検証 | `criterion` | `cargo bench` |

### 6.2 単体テストの範囲

| モジュール | テスト対象 |
|---|---|
| `domain/scoring.rs` | 各シグナルの算出（境界値含む）、未設定時の挙動、ソートルール |
| `domain/dependency.rs` | 循環依存検出、自己参照検出、依存解除時の整合性 |
| `domain/status.rs` | ステータス遷移の正当性、冪等性 |
| `domain/date_parser.rs` | 相対日付パース（today, tomorrow, +3d, friday等）、不正入力 |
| `storage/markdown.rs` | Front Matterのパース/シリアライズ、不正ファイルのハンドリング |
| `config/settings.rs` | 設定ファイルの読み込み、デフォルト値のフォールバック、部分設定 |

### 6.3 統合テストの範囲

| テストシナリオ | 検証内容 |
|---|---|
| タスクのライフサイクル | add → start → done の一連フロー |
| 依存関係フロー | depends → done（ブロック解除）→ pending（再ブロック） |
| 優先度スコアリング | 複数タスク作成後の `task next` / `task list` のソート順 |
| pin/unpin | pinしたタスクが最上位に表示されること |
| JSON出力 | `--json` フラグで有効なJSONが出力されること |
| エラーケース | 存在しないID、循環依存、自己参照 |
| 設定ファイル | カスタム重みでのスコアリング挙動の変化 |
| データ破損 | 不正なMarkdownファイルがある状態での操作 |

### 6.4 ベンチマーク対象

| 対象 | 要件 |
|---|---|
| CLI起動時間 | 100ms以下 |
| 1000タスクの読み込み + スコア計算 | 500ms以下 |
| 単一タスクの読み書き | 100ms以下 |

## 7. ビルドとリリース

### 7.1 ビルド設定

```toml
# Cargo.toml
[package]
name = "taskctl"
version = "0.1.0"
edition = "2021"
rust-version = "1.75.0"
description = "A CLI task manager with automatic priority adjustment"
license = "MIT"

[profile.release]
opt-level = 3
lto = true
strip = true
```

`lto = true` と `strip = true` でバイナリサイズを最小化し、起動速度を改善する。

### 7.2 クロスプラットフォームビルド

| ターゲット | プラットフォーム |
|---|---|
| `x86_64-unknown-linux-gnu` | Linux (x86_64) |
| `aarch64-unknown-linux-gnu` | Linux (ARM64) |
| `x86_64-apple-darwin` | macOS (Intel) |
| `aarch64-apple-darwin` | macOS (Apple Silicon) |

GitHub ActionsでCI/CDパイプラインを構成し、タグプッシュ時にリリースビルドを実行する。

### 7.3 配布

| 方法 | 対象 |
|---|---|
| `cargo install taskctl` | Rust ツールチェインを持つユーザー |
| GitHub Releases | ビルド済みバイナリを直接ダウンロードするユーザー |

### 7.4 CI/CD パイプライン

```
push / PR
    │
    ▼
┌─────────────────┐
│  cargo fmt       │  フォーマットチェック
│  cargo clippy    │  リントチェック
│  cargo test      │  全テスト実行
│  cargo bench     │  ベンチマーク（main ブランチのみ）
└─────────────────┘
    │
    ▼ (タグプッシュ時のみ)
┌─────────────────┐
│  クロスビルド    │  4ターゲットのリリースビルド
│  GitHub Release  │  バイナリをアップロード
│  crates.io 公開  │  cargo publish
└─────────────────┘
```

## 8. 技術的制約と要件

### 8.1 パフォーマンス制約

| 要件 | 対策 |
|---|---|
| CLI起動100ms以下 | Rustのネイティブバイナリ、リリースビルドでLTO有効化 |
| 1000タスクのスコア計算500ms以下 | 逐次ファイル読み込み。必要に応じてFront Matterのみのストリーミングパースを導入 |
| ファイルI/O 100ms以下 | 1タスク1ファイルで個別アクセス。バッファリング書き込み |

### 8.2 プラットフォーム制約

| 制約 | 対策 |
|---|---|
| ファイルロック方式がOS間で異なる | `fs2` クレートでクロスプラットフォームな `flock` 互換ロックを使用 |
| パス区切り文字の違い（`/` vs `\`） | `std::path::PathBuf` を使用し、パス結合はOS依存にしない |
| `~` のチルダ展開 | `dirs` クレートで `home_dir()` を取得し、プログラム側で展開 |
| `NO_COLOR` 環境変数 | `colored` クレートが自動対応。`--no-color` フラグでも制御可能 |

### 8.3 データ制約

| 制約 | 対策 |
|---|---|
| Markdownファイルの手動編集による不整合 | パース失敗時は警告を出力してスキップ。不正フィールドはデフォルト値で補完 |
| タスクIDの上限 | `u32` 型（最大 4,294,967,295）。実用上問題なし |
| ファイル数の上限 | OS依存（ext4で約6万ファイル/ディレクトリ）。1000タスク上限のスコープでは問題なし |
