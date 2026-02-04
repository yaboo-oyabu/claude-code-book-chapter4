# 初回実装 タスクリスト

## Phase 1: 基盤

- [x] 1.1 プロジェクト初期化
  - `cargo init --name taskctl`
  - `Cargo.toml` に依存クレートを追加
  - モジュール構造の骨格を作成（空の `mod.rs`）
  - `.gitignore` 作成

- [x] 1.2 エラー型定義 (`src/error.rs`)
  - `TaskCtlError` 列挙型の定義
  - `exit_code()` メソッドの実装
  - 単体テスト

- [x] 1.3 Config Layer (`src/config/`)
  - `Config` 構造体の定義と `impl Default`
  - TOML設定ファイルの読み込み
  - 解決順序（CLI引数 > 環境変数 > ファイル > デフォルト）
  - 単体テスト

- [x] 1.4 Markdownパーサー (`src/storage/markdown.rs`)
  - Front Matter の抽出（`---` 区切り）
  - YAML → Task構造体のデシリアライズ
  - Task構造体 → YAML + Markdown body のシリアライズ
  - パース失敗時のエラーハンドリング
  - 単体テスト

- [x] 1.5 メタデータ管理 (`src/storage/meta.rs`)
  - `.meta.json` の読み書き
  - `next_id` の取得と更新
  - ファイル未存在時の初期化
  - 単体テスト

- [x] 1.6 ロックファイル管理 (`src/storage/lock.rs`)
  - `fs2` によるアドバイザリロック
  - タイムアウト（5秒）
  - ステールロック検出（PID確認）
  - 単体テスト

- [x] 1.7 リポジトリ層 (`src/storage/repository.rs`)
  - `create(task)` — ファイル書き込み + ID採番
  - `read(id)` — ファイル読み込み
  - `read_all()` — 全タスク読み込み
  - `update(task)` — ファイル上書き
  - `delete(id)` — ファイル削除 + 依存参照の除去
  - データディレクトリの自動作成
  - 単体テスト

## Phase 2: ビジネスロジック

- [x] 2.1 Task構造体 (`src/domain/task.rs`)
  - `Task` 構造体の定義
  - `Estimate` 列挙型のパースと時間正規化
  - タスク作成ヘルパー（デフォルト値設定）
  - 単体テスト

- [x] 2.2 ステータス遷移 (`src/domain/status.rs`)
  - `Status` 列挙型と文字列変換
  - `transition()` 関数（冪等性対応）
  - 単体テスト（全遷移パターン + 冪等ケース）

- [x] 2.3 日付パーサー (`src/domain/date_parser.rs`)
  - 絶対日付パース（`YYYY-MM-DD`）
  - 相対日付パース（`today`, `tomorrow`, `+3d`, `+1w`）
  - 曜日パース（`monday` 〜 `sunday`）
  - 不正入力のエラーハンドリング
  - 単体テスト

- [x] 2.4 依存関係管理 (`src/domain/dependency.rs`)
  - `add_dependency()` — 自己参照・循環依存チェック付き
  - `remove_dependency()`
  - `get_dependency_tree()` — ツリー構造構築
  - `is_blocked()` — ブロック状態判定
  - `get_blocking_tasks()` — ブロック対象タスク一覧
  - 単体テスト（循環検出、自己参照、複雑なDAG）

- [x] 2.5 スコアリング (`src/domain/scoring.rs`)
  - `urgency_signal()` — 期限の接近
  - `blocking_signal()` — 他タスクのブロック数
  - `staleness_signal()` — 放置期間
  - `quick_win_signal()` — タスク粒度
  - `blocked_penalty()` — ブロックペナルティ
  - `calculate_score()` — 総合スコア算出
  - `sort_tasks()` — pinned + スコア + created_at のソート
  - `generate_summary()` — 簡易説明の生成
  - 単体テスト（各シグナルの境界値、未設定時の挙動、ソート順）

## Phase 3: コマンド実装

- [x] 3.1 CLI引数定義 (`src/cli/args.rs`)
  - clap derive マクロで全コマンド・オプションを定義
  - グローバルオプション（`--json`, `--no-color`, `--data-dir`, `--config`）

- [x] 3.2 出力フォーマッター (`src/cli/output.rs`)
  - `OutputFormat` 列挙型（Color / Plain / Json）
  - タスク一覧のフォーマット
  - タスク詳細のフォーマット
  - next / today のフォーマット
  - ツリー表示のフォーマット
  - カラー制御（`--no-color`, `NO_COLOR`, TTY検出）

- [x] 3.3 `task init` コマンド (`src/cli/commands/init.rs`)
  - デフォルト設定ファイルの生成
  - 既存ファイル時のエラー / `--force` 対応

- [x] 3.4 `task add` コマンド (`src/cli/commands/add.rs`)
  - 全オプションの処理（`--due`, `--tag`, `--estimate`, `--note`, `--depends`）
  - 日付パーサー連携
  - 成功メッセージ表示

- [x] 3.5 `task show` コマンド (`src/cli/commands/show.rs`)
  - 全属性の詳細表示
  - 依存関係（depends on / blocks）の表示
  - pinned状態の表示

- [x] 3.6 `task list` コマンド (`src/cli/commands/list.rs`)
  - スコア順ソート表示
  - フィルタ（`--tag`, `--status`, `--due-before`, `--due-after`, `--all`）
  - 完了タスクのデフォルト非表示
  - 0件時のメッセージ

- [x] 3.7 `task edit` コマンド (`src/cli/commands/edit.rs`)
  - 属性の更新
  - 属性の削除（空文字指定）
  - `--remove-tag` 対応
  - `depends_on` 変更時の循環依存チェック

- [x] 3.8 `task delete` コマンド (`src/cli/commands/delete.rs`)
  - 確認プロンプト
  - `--force` でスキップ
  - 依存参照の自動除去

- [x] 3.9 `task start` / `task done` / `task pending` (`src/cli/commands/status.rs`)
  - ステータス遷移の実行
  - 冪等性処理
  - `task done` のブロック解除メッセージ
  - `task pending` の再ブロック処理

- [x] 3.10 `task pin` / `task unpin` (`src/cli/commands/pin.rs`)
  - pinned フラグと pinned_at の更新
  - 冪等性処理

- [x] 3.11 `task depends` / `task undepends` / `task tree` (`src/cli/commands/depends.rs`)
  - 依存関係の追加・解除
  - ツリー表示
  - エラーメッセージ（循環依存、自己参照）

- [x] 3.12 `task next` コマンド (`src/cli/commands/next.rs`)
  - ブロック中タスクを除外してスコア最上位の1件を表示
  - 0件時のメッセージ

- [x] 3.13 `task today` コマンド (`src/cli/commands/today.rs`)
  - 条件合致タスクの抽出（期限今日以前 / in_progress / pinned）
  - 0件時のフォールバック（task next と同じ結果）

- [x] 3.14 `task search` コマンド (`src/cli/commands/search.rs`)
  - タイトル・メモの部分一致検索（case-insensitive）
  - フィルタとの組み合わせ

- [x] 3.15 `task migrate` コマンド (`src/cli/commands/migrate.rs`)
  - スキーマバージョンの確認
  - バックアップディレクトリの作成
  - `--dry-run` 対応

- [x] 3.16 `task completions` コマンド (`src/cli/commands/completions.rs`)
  - clap の補完生成機能を利用
  - bash / zsh / fish 対応

## Phase 4: 横断的機能

- [x] 4.1 エントリーポイント統合 (`src/main.rs`)
  - コマンドディスパッチ
  - `TaskCtlError` → 終了コード変換
  - `Error:` / `Hint:` プレフィックス表示

- [x] 4.2 グローバルオプション統合
  - `--json` での出力切り替え
  - `--no-color` でのカラー無効化
  - `--data-dir` / `--config` の解決

- [x] 4.3 環境変数統合
  - `TASKCTL_CONFIG` / `TASKCTL_DATA_DIR` の読み込み
  - グローバルオプションとの優先順位

## Phase 5: 品質

- [x] 5.1 統合テスト
  - タスクライフサイクル（add → start → done）
  - 依存関係フロー（depends → done → unblock）
  - スコアリング検証（list / next のソート順）
  - pin/unpin の動作
  - JSON出力の検証
  - エラーケース（存在しないID、循環依存、自己参照）
  - 設定ファイルのカスタム重み
  - データ破損時の挙動
  - エッジケーステーブル全ケースのカバー

- [ ] 5.2 ベンチマーク
  - 1000タスクの読み込み + スコア計算（500ms以下を検証）
  - `benches/scoring_bench.rs` 作成

- [x] 5.3 CI設定
  - `.github/workflows/ci.yml`（fmt + clippy + test）
  - `.github/workflows/release.yml`（クロスビルド + GitHub Release）

- [x] 5.4 最終検証
  - `cargo fmt --check` パス
  - `cargo clippy -- -D warnings` パス
  - `cargo test` 全テストパス
  - リリースビルドの起動時間100ms以下を確認

## 完了条件

全タスクにチェックが入り、Phase 5.4 の最終検証がパスした状態でMVP v0.1 完了とする。
