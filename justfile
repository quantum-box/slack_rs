# このファイルを使用するには、以下のURLからjustをインストールしてください:
# https://github.com/casey/just

# デフォルトのレシピ（`just`コマンドで実行）
default:
    @just --list

# 開発サーバーを起動
run:
    cargo run

# Webhookサンプルを実行
run-webhook:
    cargo run --example webhook_example

# Socket Modeサンプルを実行
run-socket:
    cargo run --example socket_mode_example --features socket_mode

# テストを実行
test:
    cargo test

# コードをフォーマット
fmt:
    cargo fmt

# リントチェック
lint:
    cargo clippy

# 環境変数ファイルを初期化
init-env:
    cp .env.sample .env

# ビルドチェック
check:
    cargo check

# すべての検証を実行（フォーマット、リント、テスト）
verify: fmt lint test
    @echo "すべての検証が完了しました"
