[English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md)

# Copiwaifu

日々のコーディングを見守る Live2D AI ナビゲーター。

Copiwaifu は Tauri ベースのデスクトップペットです。AI コーディングツールの状態をデスクトップ上の小さな Live2D キャラクターに同期して表示します。現在は Claude Code、GitHub Copilot、Codex、Gemini CLI、OpenCode との同期を主な対象にしており、コーディングターン完了後に保存済みセッションメタデータから AI Talk の吹き出しを生成することもできます。

## 公式サイト

- Copiwaifu: https://copiwaifu.panzer-jack.cn/

<img width="1512" height="824" alt="image" src="https://github.com/user-attachments/assets/bc2bead0-5c3b-4d61-9924-5c61455d9ce0" />

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
- AI Talk を有効にすると、完了またはエラー時に現在のセッション要約から短い吹き出しを生成できます。
- 内蔵 Live2D モデルとカスタムモデルフォルダーの両方に対応します。
- 実行状態ごとにモデルのモーショングループを割り当てられます。
- 設定ウィンドウで言語、名前、自動起動、モデル、ウィンドウサイズ、AI Talk のモデル設定、モーション割り当てを変更できます。
- トレイメニューから表示切替、設定、終了を操作できます。
- GitHub Releases からアプリの更新を確認できます。

## 動作の流れ

アプリ起動時に次の処理を行います。

1. `127.0.0.1` でローカル HTTP サーバーを起動し、既定ではポート `23333` を使い、使用中なら次の空きポートを順に試します。
2. 対応する AI CLI にローカル hooks をインストールします。
3. hook イベントを受け取り、Copiwaifu の状態に変換します。
4. 実行時データを `~/.copiwaifu` に保存し、セッション要約は `~/.copiwaifu/sessions` に書き込みます。
5. AI Talk が有効で、完了または失敗したターンに十分なコンテキストがある場合、Vercel AI SDK 経由でローカル AI runtime を実行し、静的な complete/error 吹き出しを生成文に差し替えます。

現在 hook を導入する設定ファイルは次のとおりです。

- Claude Code: `~/.claude/settings.json`
- GitHub Copilot: `~/.config/github-copilot/config.json`
- Codex: `~/.codex/config.toml`
- Gemini CLI: `~/.gemini/settings.json`
- OpenCode: `~/.config/opencode/opencode.json`

元の hook 定義は `~/.copiwaifu/hooks/original-hooks.json` にバックアップされます。

## 対応プラットフォーム

Copiwaifu は macOS のネイティブなデスクトップペット向けウィンドウ挙動を維持しつつ、Windows でも保守しやすい開発・パッケージング経路を追加しています。

- macOS: 引き続き `tauri-nspanel`、Dock 非表示、全スペース表示などの macOS 専用機能を使います。
- Windows: macOS 専用プラグインは読み込まず、Tauri 標準の透明・枠なし・常時前面ウィンドウ、システムトレイ、自動起動プラグインを使います。
- hook と実行時ファイルはユーザーディレクトリ配下の `.copiwaifu` を優先し、予備のポートファイルは固定の `/tmp` ではなくシステム一時ディレクトリへ書き込みます。
- Codex `notify` 設定では、Windows のバックスラッシュを TOML 文字列として正しくエスケープします。

Windows の保守メモは [docs/WINDOWS.zh-CN.md](docs/WINDOWS.zh-CN.md) にあります。

## 技術スタック

- Vue 3
- TypeScript
- Vite
- Tauri 2
- Rust
- PixiJS
- `easy-live2d`
- Vercel AI SDK
- Node.js sidecar runtime

## ローカル実行に必要なもの

ローカルで実行する前に、次を準備してください。

- Node.js
- `pnpm`
- Rust toolchain
- macOS 向け Tauri のシステム要件
- リアルタイム同期を使う場合は、少なくとも 1 つの対応 AI CLI
- AI Talk の生成吹き出しを使う場合は、モデルサービスの API Key

hook ブリッジと AI Talk runtime は、標準的なシェルパスから `node` を実行できることも前提にしています。

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
pnpm build                 # 型チェック、フロントエンドビルド、AI runtime のバンドル
pnpm sidecar:build         # sidecar/ai-runtime を sidecar/ai-runtime/bundle/main.mjs にバンドル
pnpm tauri dev             # デスクトップアプリを開発モードで起動
pnpm run                   # pnpm tauri dev のショートカット
pnpm tauri:build:windows   # Windows で NSIS / MSI インストーラーをビルド
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
- AI Talk の有効化、プロバイダー、モデル ID、API Key、任意の Base URL、任意のカスタムヘッダー
- 各 Agent 状態のモーショングループ割り当て

カスタムモデルを選択していない場合、Copiwaifu は内蔵の `Yulia` モデルを使用します。

## AI Talk

AI Talk は既定で無効です。有効化してプロバイダーを設定すると、セッションが `complete` または `error` になったときに短い吹き出しを 1 つ生成できます。独立したチャットウィンドウは開かず、idle 中にランダム起動せず、元の AI CLI に指示を書き戻すこともありません。

AI Talk が使うのは、Copiwaifu がすでに保存しているセッションメタデータだけです。Agent 種別、session id、作業ディレクトリ、セッションタイトル、最近のイベント/ツール、利用可能な要約を使い、完全なチャットログ、プロジェクトファイル、ソースコードは読み取りません。

現在のプロバイダー選択肢は OpenAI、Anthropic、Google Gemini、DeepSeek、Alibaba Bailian / Qwen、Moonshot Kimi、Zhipu GLM、Volcengine Ark / Doubao、Baidu Qianfan / ERNIE、Tencent Hunyuan、MiniMax、OpenAI-compatible API です。OpenAI-compatible では Base URL が必須です。プロキシやサービス固有の追加情報が必要な場合は、高度な設定でカスタムヘッダーを指定できます。

AI Talk が無効、API Key やモデルが未設定、有効なセッションコンテキストがない、またはモデル呼び出しに失敗した場合、Copiwaifu は通常の静的な状態吹き出しに戻ります。

## カスタム Live2D モデル

カスタムモデルフォルダーは保存前に検証されます。利用可能なモデルディレクトリには、有効な `.model3.json` エントリーファイルと、それが参照するアセットが含まれている必要があります。

状態ごとのアニメーションを活用したい場合は、モデル側で対応する motion group を用意し、設定画面で割り当ててください。明示的な割り当てがない場合、Copiwaifu は `Idle`、`Thinking`、`ToolUse`、`Complete` のような一般的なグループ名との自動一致を試みます。見つからなければ、その状態は未割り当てのままになります。

## 更新とリリース

- アプリは Tauri updater プラグインで更新を確認します。
- 更新メタデータは GitHub Releases から取得します。
- `app-v*` 形式のタグを push すると、リポジトリのワークフローが macOS Apple Silicon 版、macOS Intel 版、Windows 版の成果物を公開します。

## 注意事項

- Copiwaifu は hooks を導入するためにローカルの AI CLI 設定ファイルを変更します。すでに独自の hook チェーンを管理している場合は差分を確認してください。
- セッション状態や主要なポート情報などの実行時ファイルは `~/.copiwaifu` に書き込まれます。
- AI Talk 設定は API Key を含めてローカルのアプリ設定ファイルに保存され、選択したモデルサービスの呼び出しにのみ使われます。
- セッションファイルには、復元と AI Talk のために最近のイベント要約、`lastMeaningfulSummary`、`aiTalkContext` などの限定的なメタデータが保存されます。
- 予備のポートファイルはシステム一時ディレクトリにも書き込まれます。

## ライセンス

このプロジェクトは MIT License の下で配布されています。詳細は [LICENSE](LICENSE) を参照してください。

同梱されている Live2D Cubism Core ファイルには独自のライセンス条件があります。詳細は [public/Core/LICENSE.md](public/Core/LICENSE.md) と [public/Core/README.md](public/Core/README.md) を参照してください。
