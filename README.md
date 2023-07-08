# journalint

Linter for my personal journal files.

## Test

This project uses [insta](https://insta.rs/) snapshot testing tool.

## Basic design

大まかな構造を chumsky で実施し、パース失敗したなら
そのときに出てきたエラーを diagnostic として返す。

大まかな構造をパースした結果は AST に相当するデータとする。

その構造に対して各種の lint を追加で行う。
lint は、基本的にエラー終了しない。
