# human-emulator — 健常な人間エミュレータ

五つの欲求(眠気・空腹・寂しさ・不潔感・ストレス)と一枚の予定表だけで、人間の生活を
分単位で回すシミュレータ。出てきた一週間がどれだけ「ふつう」だったかを健常度として採点する。

同じ実装が二つの顔を持つ:

- **CLI** — `cargo run` で端末に一週間を出力する
- **Web** — Yew で WebAssembly に焼き、ブラウザの中だけでシミュレーションが走る

## 動かす

```bash
cargo run -- --seed 42 --days 7 --timeline   # CLI
trunk serve                                   # http://localhost:8080
trunk build --release                         # dist/ に静的サイトが出る
```

`trunk` が要る: `cargo install trunk` と `rustup target add wasm32-unknown-unknown`。

## モデル

行動の選択は毎分ではなく「行動の塊」単位で行われ、次の予定の境界で切られる。
自由時間の選択は次の積で決まる:

```
欲求の切迫さ × 余暇の下駄 × その日の飽き × その時間帯としてのふつうさ × ゆらぎ
```

健常さの本体は意志ではなく **時間帯としてのふつうさ**(`Activity::appropriateness`)と
**平日の予定表**(`human::obligation`)にある。眠くても七時の目覚ましで起き、出勤する。

食事も同じ理屈で動く。空腹だから食べるのではなく、朝(6〜9時)・昼(11〜14時)・
夕(17〜21時)の枠が来たから食べる。枠は一日に一度しか使えないので、健常な人間の食事は
一日三回に落ち着く。

何を食べるかは、そのときのストレスと眠気で決まる気力次第。元気なら定食、疲れていれば
カップ麺になる。献立には栄養と満腹度があり、軽いものは空腹を残すので間食が増える。

一日の終わりに、睡眠不足・欠食・食事の中身・野菜の有無・孤立・入浴なし・勤務不足・
夜更かし・気分の落ち込みを減点して健常度を出す。

詳しい構成は [CLAUDE.md](CLAUDE.md) を参照。

## Cloudflare Pages へのデプロイ

[.github/workflows/deploy.yml](.github/workflows/deploy.yml) が検査 → ビルド → 配置まで通す。
`main` への push は本番へ、それ以外のブランチと同一リポジトリからの PR はプレビュー URL へ配られる。

準備は三つ:

1. Cloudflare のダッシュボードで Pages プロジェクトを **Direct Upload** で作る
   (名前は既定で `human-emulator`。変えるならワークフローの `PAGES_PROJECT` も合わせる)
2. API トークンを発行する。テンプレートは **Edit Cloudflare Workers**、
   または `Account / Cloudflare Pages / Edit` 権限のカスタムトークン
3. GitHub リポジトリの Secrets に登録する
   - `CLOUDFLARE_API_TOKEN`
   - `CLOUDFLARE_ACCOUNT_ID`

ビルド成果物は完全な静的ファイル(HTML + CSS + JS + wasm、約 180 KB)で、
サーバ側の処理は何もない。キャッシュ指定は [_headers](_headers) にある。
