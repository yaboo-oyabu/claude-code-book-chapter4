# ユビキタス言語定義

## 1. ドメイン用語

| 用語（日本語） | 用語（英語） | コード上の命名 | 定義 |
|---|---|---|---|
| タスク | Task | `Task` | ユーザーが管理する作業の最小単位。タイトル、ステータス、期限等の属性を持つ |
| ステータス | Status | `Status` | タスクの進行状態。`pending`（未着手）、`in_progress`（進行中）、`done`（完了）の3値 |
| 未着手 | Pending | `Status::Pending` | タスクがまだ着手されていない状態。作成時のデフォルト |
| 進行中 | In Progress | `Status::InProgress` | タスクに着手し作業中の状態 |
| 完了 | Done | `Status::Done` | タスクの作業が終了した状態 |
| 期限 | Due / Due Date | `due` | タスクを完了すべき日付 |
| タグ | Tag | `tags` | タスクを分類するためのラベル。1つのタスクに複数付与可能 |
| 見積もり | Estimate | `estimate` | タスクの作業量の見込み。分（m）、時間（h）、ポイント（p）で指定 |
| メモ | Note | `note` | タスクに付随する自由記述テキスト。Markdownファイルの本文部分 |
| 依存関係 | Dependency | `depends_on` | タスク間の前後関係。「タスクAはタスクBに依存する」= Bが完了しなければAに着手できない |
| 依存先 | Dependency Target | `depends_on` の要素 | あるタスクが依存しているタスク（先に完了する必要がある側） |
| 依存元 | Dependent | — | ある依存先タスクの完了を待っているタスク |
| ブロック | Block | `blocked` | 依存先タスクが未完了のため着手できない状態 |
| ブロック解除 | Unblock | `unblocked` | 依存先タスクがすべて完了し、着手可能になること |

## 2. 優先度関連の用語

| 用語（日本語） | 用語（英語） | コード上の命名 | 定義 |
|---|---|---|---|
| 優先度スコア | Priority Score | `score` | 4つのシグナルと重みから算出される数値。ソート順を決定する。ユーザーには直接公開しない |
| シグナル | Signal | `*_signal` | スコア算出の入力となる指標。urgency, blocking, staleness, quick_win の4種類 |
| 緊急度 | Urgency | `urgency_signal` | 期限の接近度を表すシグナル（0.0〜10.0） |
| ブロック数 | Blocking Count | `blocking_signal` | このタスクの完了を待っている他タスクの数を表すシグナル（0.0〜10.0） |
| 放置期間 | Staleness | `staleness_signal` | 最終更新からの経過日数を表すシグナル（0.0〜10.0） |
| クイックウィン | Quick Win | `quick_win_signal` | 見積もりが小さいタスクを優先するシグナル（0.0〜10.0） |
| 重み | Weight | `weights` | 各シグナルに対する乗数。設定ファイルでカスタマイズ可能 |
| ブロックペナルティ | Blocked Penalty | `blocked_penalty` | 依存先が未完了のタスクに適用される大きな負のスコア（-1000.0） |
| pinned（固定） | Pinned | `pinned` | ユーザーが手動で優先度を固定した状態。自動スコアリングの対象外になる |
| ソート順 | Sort Order | — | タスク一覧の表示順。pinned → スコア降順 → 作成日時昇順 |

## 3. データ関連の用語

| 用語（日本語） | 用語（英語） | コード上の命名 | 定義 |
|---|---|---|---|
| Front Matter | Front Matter | — | Markdownファイルの先頭にある `---` で囲まれたYAMLメタデータ部分 |
| スキーマバージョン | Schema Version | `schema_version` | データ形式のバージョン番号。マイグレーション時に使用 |
| メタデータファイル | Metadata File | `.meta.json` | ID採番を管理するJSONファイル |
| データディレクトリ | Data Directory | `data_directory` | タスクファイルを格納するディレクトリ。デフォルトは `~/.local/share/taskctl/` |
| ロックファイル | Lock File | `.lock` | 同時書き込みを防止するためのファイル |
| マイグレーション | Migration | `migrate` | スキーマバージョンの変更に伴うデータ形式の変換処理 |

## 4. CLI関連の用語

| 用語（日本語） | 用語（英語） | コード上の命名 | 定義 |
|---|---|---|---|
| サブコマンド | Subcommand | — | `task` に続く操作指示（`add`, `list`, `done` 等） |
| グローバルオプション | Global Option | — | 全サブコマンドで使用可能なオプション（`--json`, `--no-color` 等） |
| カラー出力 | Color Output | — | ターミナルのANSIカラーコードを使用した出力 |
| JSON出力 | JSON Output | — | `--json` フラグ指定時のスクリプト連携用出力 |
| シェル補完 | Shell Completion | `completions` | bash/zsh/fish 向けのコマンド補完スクリプト |
| 冪等 | Idempotent | — | 同じ操作を複数回実行しても結果が変わらない性質 |

## 5. アーキテクチャ関連の用語

| 用語（日本語） | 用語（英語） | コード上の命名 | 定義 |
|---|---|---|---|
| CLI層 | CLI Layer | `src/cli/` | コマンドライン引数の解析と出力フォーマットを担当するレイヤー |
| ドメイン層 | Domain Layer | `src/domain/` | ビジネスロジック（タスク操作、スコアリング、依存関係管理）を担当するレイヤー |
| ストレージ層 | Storage Layer | `src/storage/` | ファイルシステムへの永続化を担当するレイヤー |
| 設定層 | Config Layer | `src/config/` | 設定ファイルの読み込みとデフォルト値管理を担当するレイヤー |
