# kanafix

iPad / iPhone の **Windows App（RDP）** から Windows へ日本語を入力すると、
一部アプリ（Windows 11 の新しいメモ帳など）で文字が二重になる問題を補正する、
Windows 常駐ツールです。

例: 「変換が入るとおかしくなる」と入力 →
`へんかんがはいるとおか変換入るとおかしくなる`（読みの残骸＋変換）になってしまう。

## なぜ起きるのか

iPad 側で日本語変換を行い、確定文字を `VK_PACKET`（Unicode 注入）で Windows へ
送ります。変換確定時は「読みを Backspace で消す → 変換結果を注入」をミリ秒単位の
バーストで送るため、新しいメモ帳の TSF エディタが消去を処理し切れず二重化します。
（詳細は [`docs/rdp-input-analysis.md`](docs/rdp-input-analysis.md)）

kanafix はこのバーストをわずかに間引いて（再ペーシング）、エディタが追従できる
ようにします。

## ダウンロード

[GitHub Releases](https://github.com/misc1999/KanaFix/releases) から
`kanafix.exe` を入手します。各リリースには `SHA256SUMS.txt` を同梱しています。

### 入手後の検証と「不明な発行元」対処

現在 `kanafix.exe` はまだコード署名されていないため、Windows が
SmartScreen / 「不明な発行元」の警告を表示することがあります。次の手順で
ファイルの完全性を確認し、ブロックを解除できます。

```powershell
# 1. 配布ハッシュと一致するか検証する
Get-FileHash .\kanafix.exe -Algorithm SHA256
#   出力されたハッシュが Releases の SHA256SUMS.txt と一致することを確認

# 2. ダウンロード由来のブロック(Mark of the Web)を解除する
Unblock-File .\kanafix.exe
```

SmartScreen の画面が出た場合は「詳細情報」→「実行」で起動できます。

## 使い方

1. `kanafix.exe` を実行する（初回に設定ファイル `kanafix.toml` を exe と同じ場所へ
   自動生成します）。
2. 対象アプリ（既定はメモ帳）で日本語を入力する。二重化が直ります。
3. 終了は実行中のウィンドウで **Ctrl+C**。止めれば補正は完全に無効化されます。

## 設定 — `kanafix.toml`

```toml
apps = ["notepad.exe"]   # 補正したいアプリの exe 名（小文字、複数可）
pace_ms = 6              # 再注入の間隔(ms)。直り切らなければ増やす(例 12, 20)
```

- `apps` … フォアグラウンドのプロセス名と大小無視で一致したときだけ補正します。
  他アプリ・他のキーには一切触れません。アプリ名はタスクマネージャーの「詳細」
  タブのプロセス名で確認できます（例: `notepad.exe`）。
- `pace_ms` … 大きいほど安全側（入力反映がわずかに遅くなる）。

## ビルド

```powershell
cargo build --release
# 出力: target/release/kanafix.exe
```

`kanafix.exe` 単体で動作します（設定ファイルは初回に自動生成）。

## 注意

- 入力を一度奪って打ち直す方式です。挙動が不安定になったら **Ctrl+C で即停止**
  してください。元の素の入力に戻ります。
- 管理者として動くアプリを対象にしたい場合は、本ツールも管理者として起動する
  必要があります（フックが届かないため）。
- グローバルキーボードフックを使いますが、キーストロークの記録・保存は行いません。
  対象アプリ前面時の `VK_PACKET` / `Backspace` を再注入するだけです。

## コード署名ポリシー

kanafix の公式バイナリは、GitHub の `main` リポジトリ（`misc1999/KanaFix`）に
タグ `v*` が push されたときに、GitHub Actions の
[`release` ワークフロー](.github/workflows/release.yml) でビルドされます。
ローカルでビルドしたバイナリを手動でアップロードすることはありません。

- **リリース体制**: 本プロジェクトは個人メンテナ（amketta）による単独運用です。
  Author・Reviewer・Approver はいずれも amketta が担い、タグの作成と
  リリースの承認を行います。
- **配布物の完全性**: 各リリースに `SHA256SUMS.txt` を同梱します。
- **コード署名**: コード署名は [SignPath Foundation](https://signpath.org/) が
  オープンソース向けに提供する証明書での署名導入を予定しています。署名が
  有効化されるまでの間、バイナリは未署名で配布され、上記のハッシュ検証で
  完全性を確認できます。
- **連絡先**: セキュリティ上の懸念や署名に関する問い合わせは
  `contact@amketta.com` まで。

## ライセンス

[MIT](LICENSE)
