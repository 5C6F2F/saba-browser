# saba-browser
書籍「[［作って学ぶ］ブラウザのしくみ ──HTTP、HTML、CSS、JavaScriptの裏側](https://gihyo.jp/book/2024/978-4-297-14546-0)」の実装

## 参考
+ [d0iasm/saba](https://github.com/d0iasm/saba)
+ [d0iasm/sababook](https://github.com/d0iasm/sababook)

## おまけ
rust-analyzerがエラーを出すので、以下の手順を行った。

+ バージョン0.3.2291のrust-analyzerをインストール。
+ `Cargo.lock`の3行目を編集。

```diff
- version = 4
+ version = 3
```
