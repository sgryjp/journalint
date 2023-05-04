# journalint

Linter for my personal journal files.

## Basic design

大まかな構造を chumsky で実施し、パース失敗したなら
そのときに出てきたエラーを diagnostic として返す。

大まかな構造をパースした結果は AST ではないが、
それに相当するデータとする。

その構造に対して各種の lint を追加で行う。
lint は、基本的にエラー終了しない。
