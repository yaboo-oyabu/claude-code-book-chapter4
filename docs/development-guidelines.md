# 開発ガイドライン

## 1. コーディング規約

### 1.1 フォーマット

- `rustfmt` のデフォルト設定に従う
- CIで `cargo fmt --check` を実行し、フォーマット違反をブロックする
- エディタ保存時の自動フォーマットを推奨

### 1.2 リント

- `clippy` のデフォルト警告に加え、以下のリントを有効化する

```toml
# Cargo.toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }

# pedantic のうち許容するもの
module_name_repetitions = "allow"
must_use_candidate = "allow"
```

- CIで `cargo clippy -- -D warnings` を実行し、警告をエラーとして扱う

### 1.3 エラーハンドリング

| 場面 | 方針 |
|---|---|
| Domain Layer / Storage Layer | `TaskCtlError` を返す。`unwrap()` / `expect()` は禁止 |
| CLI Layer | `anyhow::Result` で受け取り、エラーメッセージと終了コードを出力 |
| テストコード | `unwrap()` / `expect()` 許可 |

### 1.4 `unsafe` の使用

- `unsafe` は原則禁止。使用する場合はPRレビューで理由を明示する

### 1.5 コメント

- 公開関数・構造体には `///` docコメントを記述する
- 自明なコードにコメントは不要。「なぜ」が分かりにくい箇所にのみ `//` コメントを付ける
- TODO コメントは `// TODO: #<issue番号> 説明` の形式で記述する

## 2. 命名規則

### 2.1 Rust標準の命名規約に従う

| 対象 | 規約 | 例 |
|---|---|---|
| 構造体 / 列挙型 / トレイト | `PascalCase` | `Task`, `Status`, `ScoreResult` |
| 関数 / メソッド / 変数 | `snake_case` | `calculate_score`, `days_remaining` |
| 定数 | `SCREAMING_SNAKE_CASE` | `MAX_SCORE`, `DEFAULT_WEIGHT` |
| モジュール / ファイル名 | `snake_case` | `date_parser.rs`, `scoring.rs` |
| クレート名 | `snake_case`（ハイフン区切り） | `taskctl` |

### 2.2 ドメイン用語との対応

コード内の命名は `glossary.md` のユビキタス言語に準拠する。ドメイン用語と異なる命名を使用しない。

| ドメイン用語 | コード上の命名 | NG例 |
|---|---|---|
| タスク | `Task` | `Item`, `Todo` |
| ステータス | `Status` | `State`, `Phase` |
| 優先度スコア | `score` | `priority`, `rank` |
| 依存関係 | `depends_on` | `blocks`, `requires` |
| pinned | `pinned` | `fixed`, `locked` |

## 3. スタイリング規約

### 3.1 ターミナル出力のカラーリング

| 要素 | 色 | 用途 |
|---|---|---|
| `in_progress` のインジケータ `●` | 緑 | 進行中タスク |
| `pending` のインジケータ `○` | 白（デフォルト） | 未着手タスク |
| `done` のインジケータ `✓` | グレー | 完了タスク |
| `[blocked]` ラベル | 黄 | ブロック中の表示 |
| `Error:` プレフィックス | 赤 | エラーメッセージ |
| `Hint:` プレフィックス | シアン | ヒントメッセージ |
| タスクID `#12` | ボールド | 識別性の向上 |

### 3.2 カラー制御

- `--no-color` フラグまたは `NO_COLOR` 環境変数でカラーを無効化
- `--json` 出力時はカラーを自動的に無効化
- パイプ出力時（stdout が TTY でない場合）はカラーを自動的に無効化

## 4. テスト規約

### 4.1 テストの命名

```rust
#[test]
fn test_<対象>_<条件>_<期待結果>() {
    // ...
}
```

例:
```rust
#[test]
fn test_urgency_signal_due_tomorrow_returns_high_score() { ... }

#[test]
fn test_add_task_without_due_creates_task_with_null_due() { ... }

#[test]
fn test_cyclic_dependency_returns_error() { ... }
```

### 4.2 テストの構成

Arrange-Act-Assert パターンに従う。

```rust
#[test]
fn test_scoring_blocked_task_gets_penalty() {
    // Arrange
    let task = create_task_with_dependency(dep_id);
    let config = default_config();

    // Act
    let score = calculate_score(&task, &all_tasks, &config);

    // Assert
    assert!(score.score < 0.0);
}
```

### 4.3 統合テストの原則

- 各テストは独立した一時ディレクトリを使用する（テスト間の干渉を防止）
- テスト後のクリーンアップは `tempfile::TempDir` の `Drop` に任せる
- テストは並列実行可能にする（共有状態を持たない）

### 4.4 テスト実行

```bash
# 全テスト
cargo test

# 単体テストのみ
cargo test --lib

# 統合テストのみ
cargo test --test '*'

# 特定テストの実行
cargo test test_urgency_signal

# ベンチマーク
cargo bench
```

## 5. Git規約

### 5.1 ブランチ戦略

| ブランチ | 用途 |
|---|---|
| `main` | 安定版。リリース可能な状態を維持 |
| `feat/<機能名>` | 新機能開発 |
| `fix/<バグ名>` | バグ修正 |
| `docs/<内容>` | ドキュメント更新 |
| `refactor/<対象>` | リファクタリング |
| `chore/<内容>` | 開発環境・CI等の変更 |

### 5.2 コミットメッセージ

[Conventional Commits](https://www.conventionalcommits.org/) に従う。

```
<type>(<scope>): <description>

[optional body]
```

**type:**

| type | 用途 |
|---|---|
| `feat` | 新機能 |
| `fix` | バグ修正 |
| `docs` | ドキュメント |
| `refactor` | リファクタリング（機能変更なし） |
| `test` | テストの追加・修正 |
| `chore` | ビルド、CI、依存関係の更新 |
| `perf` | パフォーマンス改善 |

**scope（任意）:**

コマンド名またはモジュール名を使用する。

```
feat(add): support relative date parsing for --due option
fix(scoring): correct blocking signal direction
refactor(storage): extract markdown parser into separate module
test(dependency): add cyclic dependency edge cases
chore(ci): add cross-platform build targets
```

### 5.3 コミットの粒度

- 1コミット = 1つの論理的な変更
- コンパイルが通らないコミットは禁止
- テストの追加は機能実装と同じコミットに含める

### 5.4 PRルール

- `main` への直接プッシュは禁止
- PRはCIが全てパスしてからマージ
- PRの説明にはユーザーストーリーIDまたはタスクリストの項目を記載する

## 6. 開発コマンド一覧

```bash
# ビルド
cargo build                    # デバッグビルド
cargo build --release          # リリースビルド

# テスト
cargo test                     # 全テスト
cargo test --lib               # 単体テストのみ
cargo test --test '*'          # 統合テストのみ
cargo bench                    # ベンチマーク

# リント・フォーマット
cargo fmt                      # フォーマット実行
cargo fmt --check              # フォーマットチェック（CIと同等）
cargo clippy -- -D warnings    # リントチェック（CIと同等）

# 実行
cargo run -- add "タスク名"    # デバッグビルドで実行
cargo run -- list              # タスク一覧
cargo run -- next              # 次のタスク
```
