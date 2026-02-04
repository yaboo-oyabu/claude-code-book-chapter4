# 初回実装 要求内容

## 概要

プロダクト要求定義書（docs/product-requirements.md）で定義したMVP（v0.1）の全機能を初回実装として開発する。

## 実装対象機能

### コアCRUD
- `task add` — タスク作成（全オプション対応）
- `task show` — タスク詳細表示
- `task list` — タスク一覧表示（フィルタ、ソート、カラー/JSON出力）
- `task edit` — タスク編集（属性の更新・削除）
- `task delete` — タスク削除（確認プロンプト + --force）
- `task search` — フリーテキスト検索

### ステータス管理
- `task start` — pending → in_progress
- `task done` — pending/in_progress → done（依存タスクのブロック解除）
- `task pending` — in_progress/done → pending（依存タスクの再ブロック）
- 同一ステータスへの遷移は冪等に処理

### 優先度の自動調整
- 4シグナルによるスコアリング（urgency, blocking, staleness, quick_win）
- ブロックペナルティ（-1000.0）
- 設定ファイルによる重みカスタマイズ
- 簡易説明の生成（主要因の1行表示）

### 手動オーバーライド
- `task pin` / `task unpin`
- pinnedタスクはスコアリング対象外、リスト最上位表示

### 依存関係管理
- `task depends` / `task undepends`
- `task tree` — 依存関係ツリー表示
- 循環依存・自己参照の検出
- 依存先タスク削除時の自動除去

### 表示コマンド
- `task next` — 次にやるべきタスク（ブロック中を除外）
- `task today` — 今日のタスク一覧

### ユーティリティ
- `task init` — 設定ファイル生成
- `task migrate` — データマイグレーション
- `task completions` — シェル補完スクリプト出力

### 横断的機能
- カラー出力 / `--json` / `--no-color`
- `--data-dir` / `--config` グローバルオプション
- `TASKCTL_CONFIG` / `TASKCTL_DATA_DIR` 環境変数
- ロックファイルによる同時アクセス防止
- データ破損時のグレースフル処理

## 受け入れ条件

1. プロダクト要求定義書のユーザーストーリー US-01〜US-10 の受け入れ条件をすべて満たす
2. 機能設計書のエッジケース定義（10章）の全ケースが正しく処理される
3. `cargo test` で全テストがパスする
4. `cargo clippy -- -D warnings` で警告がない
5. `cargo fmt --check` でフォーマット違反がない
6. CLI起動時間が100ms以下（リリースビルド）

## 制約事項

- MVP範囲外の機能（Git連携、繰り返しタスク、TUI等）は実装しない
- ネイティブWindows対応はスコープ外（WSLのみ）
- Homebrew対応はスコープ外
