[English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md)

# Copiwaifu

日々のコーディングを見守る Live2D AI ナビゲーター。

Copiwaifu は Tauri ベースのデスクトップペットです。AI コーディングツールの状態をデスクトップ上の小さな Live2D キャラクターに同期して表示します。現在は Claude Code、GitHub Copilot、Codex、Gemini CLI、OpenCode との同期を主な対象にしています。

## 公式サイト

- Copiwaifu: https://copiwaifu.panzer-jack.cn/

<img width="1512" height="824" alt="image" src="https://github.com/user-attachments/assets/7c387482-58ba-4e61-b500-0b7c0b01404a" />

## macOS にインストールした後（重要）

アプリを `/Applications` に移動したあと、次を実行してください。

```bash
xattr -dr com.apple.quarantine /Applications/copiwaifu.app
```

その後 Copiwaifu を開けば通常どおり利用できます。

## できること

- AI セッションの状態変化に反応する Live2D デスクトップペットを表示します。
- `idle`、`thinking`、`tool_use`、`error`、`complete`、`needs_attention` などの状態を同期します。
- 現在の Agent と状態に応じた短い吹き出しメッセージを表示します。
- 内蔵 Live2D モデルとカスタムモデルフォルダーの両方に対応します。
- 実行状態ごとにモデルのモーショングループを割り当てられます。
- 設定ウィンドウで言語、名前、自動起動、モデル、ウィンドウサイズを変更できます。
- トレイメニューから表示切替、設定、終了を操作できます。
- GitHub Releases からアプリの更新を確認できます。

## 動作の流れ

アプリ起動時に次の処理を行います。

1. `127.0.0.1` でローカル HTTP サーバーを起動し、既定ではポート `23333` を使い、使用中なら次の空きポートを順に試します。
2. 対応する AI CLI にローカル hooks をインストールします。
3. hook イベントを受け取り、Copiwaifu の状態に変換します。
4. 実行時データを `~/.copiwaifu` に保存します。

現在 hook を導入する設定ファイルは次のとおりです。

- Claude Code: `~/.claude/settings.json`
- GitHub Copilot: `~/.config/github-copilot/config.json`
- Codex: `~/.codex/config.toml`
- Gemini CLI: `~/.gemini/settings.json`
- OpenCode: `~/.config/opencode/opencode.json`

元の hook 定義は `~/.copiwaifu/hooks/original-hooks.json` にバックアップされます。

## 対応プラットフォーム

Copiwaifu は現在 macOS 向けに開発・リリースされています。アプリは macOS 固有のウィンドウ挙動を利用しており、リリースワークフローも macOS 向け成果物のみを公開します。

## 技術スタック

- Vue 3
- TypeScript
- Vite
- Tauri 2
- Rust
- PixiJS
- `easy-live2d`

## ローカル実行に必要なもの

ローカルで実行する前に、次を準備してください。

- Node.js
- `pnpm`
- Rust toolchain
- macOS 向け Tauri のシステム要件
- リアルタイム同期を使う場合は、少なくとも 1 つの対応 AI CLI

hook ブリッジは標準的なシェルパスから `node` を実行できることも前提にしています。

## クイックスタート

```bash
pnpm install
pnpm run
```

`pnpm run` は Tauri の開発アプリを起動します。明示的に実行する場合は次でも構いません。

```bash
pnpm tauri dev
```

## 利用可能なスクリプト

```bash
pnpm dev                   # Vite のみ起動
pnpm build                 # フロントエンドをビルド
pnpm tauri dev             # デスクトップアプリを開発モードで起動
pnpm run                   # pnpm tauri dev のショートカット
pnpm tauri build           # デスクトップ成果物をローカルでビルド
pnpm release               # release-it を実行
pnpm release:sync-version  # package.json / tauri.conf.json / Cargo.toml のバージョンを同期
```

## 設定

設定ウィンドウでは次の内容を変更できます。

- ペット名
- UI 言語: 英語 / 中国語 / 日本語
- ログイン時に自動起動
- Live2D モデルディレクトリ
- ウィンドウサイズプリセット
- 各 Agent 状態のモーショングループ割り当て

カスタムモデルを選択していない場合、Copiwaifu は内蔵の `Yulia` モデルを使用します。

## カスタム Live2D モデル

カスタムモデルフォルダーは保存前に検証されます。利用可能なモデルディレクトリには、有効な `.model3.json` エントリーファイルと、それが参照するアセットが含まれている必要があります。

状態ごとのアニメーションを活用したい場合は、モデル側で対応する motion group を用意し、設定画面で割り当ててください。明示的な割り当てがない場合、Copiwaifu は `Idle`、`Thinking`、`ToolUse`、`Complete` のような一般的なグループ名との自動一致を試みます。見つからなければ、その状態は未割り当てのままになります。

## 更新とリリース

- アプリは Tauri updater プラグインで更新を確認します。
- 更新メタデータは GitHub Releases から取得します。
- `app-v*` 形式のタグを push すると、リポジトリのワークフローが macOS Apple Silicon 版と Intel 版の成果物を公開します。

## 注意事項

- Copiwaifu は hooks を導入するためにローカルの AI CLI 設定ファイルを変更します。すでに独自の hook チェーンを管理している場合は差分を確認してください。
- セッション状態やポート情報などの実行時ファイルは `~/.copiwaifu` に書き込まれます。
- 予備のポートファイルが `/tmp/copiwaifu-port` にも書き込まれます。

## ライセンス

このプロジェクトは MIT License の下で配布されています。詳細は [LICENSE](LICENSE) を参照してください。

同梱されている Live2D Cubism Core ファイルには独自のライセンス条件があります。詳細は [public/Core/LICENSE.md](public/Core/LICENSE.md) と [public/Core/README.md](public/Core/README.md) を参照してください。
