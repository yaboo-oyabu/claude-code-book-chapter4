# 初回実装 設計

## 1. 実装アプローチ

レイヤーの下位（Storage / Config）から上位（Domain → CLI）へ積み上げる**ボトムアップ方式**で実装する。各レイヤーの単体テストを書きながら進め、最後にCLI層の統合テストで全体を結合する。

### 実装順序

```
Phase 1: 基盤
  Storage Layer (markdown, meta, lock) + Config Layer

Phase 2: ビジネスロジック
  Domain Layer (task, status, scoring, dependency, date_parser)

Phase 3: コマンド実装
  CLI Layer (args, output, commands/*)

Phase 4: 横断的機能
  エラーハンドリング統合、JSON出力、カラー制御、環境変数対応

Phase 5: 品質
  統合テスト、ベンチマーク、CI設定
```

### 各フェーズの依存関係

```
Phase 1 ──→ Phase 2 ──→ Phase 3 ──→ Phase 4 ──→ Phase 5
基盤         ロジック      CLI          横断機能      品質
```

Phase 1〜2 は CLI なしで単体テストのみで検証する。Phase 3 以降で `cargo run` による動作確認が可能になる。

## 2. Phase 1: 基盤

### 2.1 プロジェクト初期化

- `cargo init --name taskctl`
- `Cargo.toml` に依存クレートを追加（architecture.md のテクノロジースタック参照）
- `src/` のモジュール構造を作成（空の `mod.rs` で骨格を作る）
- `src/error.rs` にエラー型を定義

### 2.2 Config Layer (`src/config/`)

- `settings.rs`: `Config` 構造体の定義とデシリアライズ
- デフォルト値の `impl Default`
- 設定ファイルの読み込み（存在しない場合はデフォルト）
- `--config` / `TASKCTL_CONFIG` の解決順序

### 2.3 Storage Layer (`src/storage/`)

- `markdown.rs`: Front Matter のパース / シリアライズ
  - `---` で囲まれたYAML部分を抽出
  - `serde_yaml` でデシリアライズ → `Task` 構造体
  - `Task` 構造体 → YAML + Markdown body のシリアライズ
  - パース失敗時のエラーハンドリング

- `meta.rs`: `.meta.json` の読み書き
  - `next_id` の取得と更新
  - ファイルが存在しない場合の初期化

- `lock.rs`: ロックファイル管理
  - `fs2` によるアドバイザリロック
  - タイムアウト（5秒）
  - ステールロック検出（PID確認）

- `repository.rs`: タスクの永続化
  - `create(task)` → ファイル書き込み + ID採番
  - `read(id)` → ファイル読み込み → Task
  - `read_all()` → 全タスク読み込み
  - `update(task)` → ファイル上書き
  - `delete(id)` → ファイル削除 + 依存参照の除去
  - データディレクトリの自動作成

## 3. Phase 2: ビジネスロジック

### 3.1 Task構造体 (`src/domain/task.rs`)

- `Task` 構造体（architecture.md の型定義に準拠）
- `Estimate` 列挙型とパース/正規化ロジック
- タスクの作成（デフォルト値の設定）

### 3.2 ステータス遷移 (`src/domain/status.rs`)

- `Status` 列挙型
- `transition(current, target) → Result<Status>` 関数
- 冪等性: 同一ステータスへの遷移は `Ok` を返し副作用なし
- `done` → `pending` 時の依存タスク再ブロック処理

### 3.3 スコアリング (`src/domain/scoring.rs`)

- `calculate_score(task, all_tasks, config) → ScoreResult`
- 4シグナルの算出関数（各シグナル個別にテスト可能にする）
  - `urgency_signal(due, today) → f64`
  - `blocking_signal(task_id, all_tasks) → f64`
  - `staleness_signal(updated_at, today) → f64`
  - `quick_win_signal(estimate, config) → f64`
- `blocked_penalty(depends_on, all_tasks) → f64`
- `sort_tasks(tasks, config) → Vec<Task>` — pinnedを最上位にした上でスコア降順
- `generate_summary(task, all_tasks) → Vec<String>` — 簡易説明の生成

### 3.4 依存関係管理 (`src/domain/dependency.rs`)

- `add_dependency(task_id, depends_on_id, all_tasks) → Result<()>`
  - 自己参照チェック
  - 循環依存チェック（DFS）
- `remove_dependency(task_id, depends_on_id) → Result<()>`
- `get_dependency_tree(task_id, all_tasks) → Tree` — ツリー構造の構築
- `is_blocked(task, all_tasks) → bool`
- `get_blocking_tasks(task_id, all_tasks) → Vec<u32>` — このタスクがブロックしているタスクID一覧

### 3.5 日付パーサー (`src/domain/date_parser.rs`)

- `parse_due(input, today) → Result<NaiveDate>`
- 対応フォーマット:
  - 絶対日付: `YYYY-MM-DD`
  - 相対日付: `today`, `tomorrow`, `+3d`, `+1w`
  - 曜日: `monday` 〜 `sunday`（次のその曜日）

## 4. Phase 3: コマンド実装

### 4.1 引数定義 (`src/cli/args.rs`)

- `clap` の derive マクロで全コマンドとオプションを定義
- サブコマンドを `enum` で列挙

### 4.2 出力フォーマッター (`src/cli/output.rs`)

- `format_task_list(tasks, format) → String` — リスト表示
- `format_task_detail(task, all_tasks, format) → String` — 詳細表示
- `format_task_next(task, all_tasks, format) → String` — next表示
- `format_tree(tree, format) → String` — ツリー表示
- `OutputFormat` 列挙型: `Color` / `Plain` / `Json`
- カラー制御: `--no-color`, `NO_COLOR` 環境変数, TTY検出

### 4.3 コマンド実装の方針

各コマンドファイル（`commands/*.rs`）は以下の責務を持つ:

1. CLI引数から Domain Layer の入力に変換
2. Domain Layer の関数を呼び出す
3. 結果を output.rs でフォーマットして表示
4. エラーを適切な終了コードに変換

実装順序（依存関係の少ないものから）:

```
1. init         — 設定ファイル生成（他コマンドに依存しない）
2. add          — タスク作成（基本中の基本）
3. show         — 詳細表示（add のテスト確認に使う）
4. list         — 一覧表示（スコアリングの動作確認）
5. edit         — 編集
6. delete       — 削除
7. status       — start / done / pending
8. pin          — pin / unpin
9. depends      — depends / undepends / tree
10. next        — 次のタスク
11. today       — 今日のタスク
12. search      — 検索
13. migrate     — マイグレーション
14. completions — シェル補完
```

## 5. Phase 4: 横断的機能

### 5.1 エラーハンドリング統合

- `main.rs` での `TaskCtlError` → 終了コード変換
- `Error:` / `Hint:` プレフィックスの表示

### 5.2 グローバルオプション

- `--json`: 全表示コマンドでJSON出力に切り替え
- `--no-color`: カラー無効化
- `--data-dir`: データディレクトリの一時変更
- `--config`: 設定ファイルパスの一時変更

### 5.3 環境変数

- `TASKCTL_CONFIG` / `TASKCTL_DATA_DIR` の読み込みと解決順序

## 6. Phase 5: 品質

### 6.1 統合テスト

- `tests/` ディレクトリに各機能カテゴリのテストを作成
- `assert_cmd` + `tempfile` で実際のバイナリを実行して検証
- エッジケーステーブル（functional-design.md 10章）の全ケースをカバー

### 6.2 ベンチマーク

- `benches/scoring_bench.rs`: 1000タスクの読み込み + スコア計算が500ms以下を検証

### 6.3 CI設定

- `.github/workflows/ci.yml`: push/PR時に fmt + clippy + test を実行
- `.github/workflows/release.yml`: タグプッシュ時にクロスビルド + GitHub Release

## 7. 変更するコンポーネント

初回実装のため全ファイルが新規作成。変更ではなく追加のみ。

| 対象 | 操作 | ファイル数 |
|---|---|---|
| `src/` | 新規作成 | 約25ファイル |
| `tests/` | 新規作成 | 約10ファイル |
| `benches/` | 新規作成 | 1ファイル |
| `.github/workflows/` | 新規作成 | 2ファイル |
| `Cargo.toml` | 新規作成 | 1ファイル |
| `.gitignore` | 新規作成 | 1ファイル |

## 8. 影響範囲

初回実装のため既存コードへの影響はない。

## 9. リスクと対策

| リスク | 影響 | 対策 |
|---|---|---|
| YAML Front Matter のパース精度 | タスクデータの読み書き不良 | serde_yaml の制約を早期に検証。Phase 1 で徹底的にテスト |
| スコアリングの直感性 | ユーザーが「次のタスク」に違和感を持つ | デフォルト重みを保守的に設定。設定ファイルで調整可能にしておく |
| ファイルロックのクロスプラットフォーム互換性 | macOS/Linux間で挙動差 | fs2 クレートの CI を確認。GitHub Actions で両プラットフォームテスト |
| 起動時間100ms要件 | リリースビルドでも超過する可能性 | Phase 5 のベンチマークで早期検出。遅い場合は遅延読み込みを検討 |
