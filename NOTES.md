# 開発メモ

## エラー例

- minute value of an env of time-range exceeds 99: `- 09:45-23:100`

## 2023-05-04

大まかな構造を chumsky で実施し、パース失敗したならそのときに出てきたエラーを
diagnostic として返す。

大まかな構造をパースした結果は AST に相当するデータとする。

その構造に対して各種の lint を追加で行う。 lint は、基本的にエラー終了しない。

## 2023-07-15

- 日記ファイルは markdown ではあるが、markdown としてのパースは意図的に行ってい
  ない
- markdown の YAML Front-matter と、作業一つ一つを記録する箇条書き（エントリ）だ
  けを解析の対象とする
- エントリ以外の markdown 文書の行は、すべて無視する
- 以上の方針から本文の解析は、エントリとしての解釈を試みて、reject されたなら無
  視すべき行として accept する
- 「無視すべき行」はどんな内容であっても確実に accept されるものとする
- よって、パーサー全体として解析に失敗することは無い

ただ、タイポ等によりエントリ行がエントリ行として accept されない内容になっている
と見逃すことになるため、linter としてはこの問題に対策をしたい。とはいえ、エント
リ行のような非エントリ行との混同も避けたい。そこで、以下のような方針を採用するこ
ととした:

- エントリ中、時刻のペアの解析に失敗した場合、エントリとして reject する
- エントリ中、時刻のペアより後の要素の解析に失敗した場合、エントリとして accept
  する

要素の解析失敗時にも accept するため、エラーリカバリを実装するということになる。

    - 09:00-10:00 X123456 012 1.00 foo: bar: Developer mtg␤
      ─┬─── ─┬─── ─┬───── ─┬─ ─┬── ─┬─────────────────────
       │     │     │       │   │    ╰── Activity
       │     │     │       │   ╰── Duration
       │     │     │       ╰── Code[1]
       │     │     ╰── Code[0]
       │     ╰── Time
       ╰── Time

## 2023-07-18

リカバリした上での accept は reject の一種とみなされるらしい。よって `A.or(B)`
のうち A がリカバリした上で accept できるとしても B の評価も行われてしまうようだ
った。したがって、次のようにパーサーを構成して:

    journal_entry.or(other_line)

journal_entry がリカバリした上で accept すると、other_line として解釈される。

色々試したが、一回のパースで全部解釈する今のやり方を踏襲するなら、上記で言う
other_line をリカバリで実装する手があると思われる。ただし、この場合は other_line
でリカバリした際に記録されるエラーと、 journal_entry 内の各要素のパースエラーで
リカバリした際に記録されるエラーとを後で選別しなければならなくなる。これがリーズ
ナブルなのかどうかは、本日時点では分からない。

---

その後に少し検討を進めたところ、そもそも Valid な文法としてエラー node を AST と
して定義しておき、`foo().or(erroneous_foo())` という感じでパースすれば絶対に失敗
しないパーサーとして対応できる。

なお、`chumsky::Parser::or` で実装すると失敗理由が不明になるので
`chumsky::Parser::or_else` で実装すると良い感じだった。

## 2023-07-30

比較的マジメに AST っぽいものを作って処理するアプローチに変えたので、Ruff の設計
を改めて勉強させてもらった。まず文法チェック (lint) は AST を Visitor パターンで
traverse する中で、ある種別のノードを発見したときに呼ばれる処理、という形で複数
のチェック処理を実行して Diagnostic を生成している。 Auto-fix については、Edit
のリストである Fix を、 Diagnostic を生成する際に付与可能なルールで、かつ付与す
べきオプションが指定されているようであれば付与するようになっている。これの実行処
理は、トリガーする側がどの Diagnostic に対する fix なのかを指定してきているはず
なので、おそらく素直に該当する Diagnostic から fix を取り出して実行するだけだと
思う。

## 2023-08-06

front matter の date, start, end を別々の文法上のノードとして定義してパーサーを
構成していたが、その実現のために (date or start or end) というパーサーを repeat
させていた。正常系であれば、たしかに問題なく順不同で各フィールドの値を区別して取
得できるのだけれど、一つでも不足または accept 不可能な内容になっていたりすると、
repeat の部分でのエラーということになり分かりにくいエラーメッセージが表示される
問題が起こった (unexpectedly found an end of input といった内容)。これは、おそら
くパーサーとしては素直でない作りを採用していたことに根本原因があると考え、素直に
front matter のフィールド解釈はフィールドの種別を区別せず行うように変更した方が
良いかもしれない。

## 2023-08-19

Quick Fix / Code action の実装を開始する。LSP 仕様で関連するメッセージは
`textDocument/codeAction`。VSCode で軽く試すと、カーソル移動のたびに
`textDocument/codeAction` のメッセージが飛んできて非同期な実行を要求される。また
、それを無視する実装のままでカーソルを動かしたりすると `$/cancelRequest` の通知
が飛んでくる。

## 2023-08-27

Quick Fix を実装するにあたって Diagnostic に Code を割り当てることにした。という
のも Code action がトリガーされたときにサーバーに飛んでくるメッセージに含められ
る情報のうち、実行すべきアクションの特定に使えるのは code (NumberOrString) と
message (String) しかないようだったから。

続いてコマンドでの autofix を実装する。自動修正はアドホックに diagnostic 生成時
に期待される正しい値を文字列として算出しておき、それに該当範囲を置換する形で進め
る。ただし置換は後方から前方に向けて連続実行する。本格的な仕組みを考えると、AST
をシリアライズできる機能を作り、パース時から AST を保持して quick fix で指定され
た diagnostic から該当するノードを探索し、それを前提に前後の文脈から修正を行って
シリアライズすることで修正されたコンテンツを生成することになると思う。

## 2023-08-30

コマンドでの autofix が実装できたので、今度は言語サーバーとして Code action に対
応していく。以下のように処理の流れが整理できると思う:

1. サーバーは、初期化フェーズにおいて以下の [Server
   Capability][lsp_types::ServerCapabilities] をクライアントに申告する
   - [Text Document Sync][lsp_types::TextDocumentSyncCapability]
   - [Code Action Provider][lsp_types::CodeActionProviderCapability]
   - [Execute Command Provider][lsp_types::ExecuteCommandOptions]
2. サーバーは、`textDocument/didOpen` および `textDocument/didChange` 通知を受信
   するたびにエラーチェックを行って [`Diagnostic`][lsp_types::Diagnostic] を作成
   し、[`textDocument/publishDiagnostics`] 通知でクライアントにそれらを報告する
3. クライアントは、報告された Diagnostic を UI に提示する
4. ユーザーは、Diagnostic のいずれかを選択する
5. クライアントは、言語サーバーにファイルの URL、カーソル位置、その位置に該当す
   る Diagnostic 一覧などを添えた [`textDocument/codeAction`] リクエストをサーバ
   ーに送信し、Code action の一覧を問い合わせる
6. サーバーは、指定されたカーソル位置と添付された
   [`Diagnostic`][lsp_types::Diagnostic] のリストから code を手がかりに、実行可
   能な[コマンド][lsp_types::Command]をリストアップしてクライアントに返送する
   - なお、ここでの「コマンド」とはユニークな名前が付けられクライアントにサーバ
     ーが公開している処理で、VSCode 拡張機能でいうところ
     `vscode.commands.registerCommand` で登録する関数を指す。
   - なお `registerCommand` を実行しても `package.json` の
     [contributes.commands][vscodeapi-contributes.commands] でリストアップしなけ
     ればユーザーには表向き全くアクセスできない状態になる（VSCode のコマンドパレ
     ットにも登録されないし、キーバインドの割当もできない）。たとえば言語サーバ
     ーの quick fix を code action として提供する場合、警告・エラーごとに異なる
     コマンドを大量に作成することになりがちなので、一般的にこれらはユーザーから
     はアクセスできないようにした方が良いと思われる（コマンドとしてユーザーから
     見えなくなろうとも、警告やエラーがある位置で表示される豆電球アイコンから
     code action は実行できる）
7. クライアントは、サーバーから得た実行可能なコマンドの一覧を UI に提示する
8. ユーザーは、実行可能なコマンドのうち一つを選択する
9. クライアントは、選択された実行可能コマンドに関連付けられた関数が呼び出される
   ので、その中で対応するサーバーのコマンドを指定した
   [`workspace/executeCommand`] リクエストをサーバーに送信する。
   - もちろんクライアントが直接その関数で処理しても良いといえば良いのだが、そう
     すると VSCode でしか実現されない code action になってしまうため、面倒だがサ
     ーバーに処理を移譲することが望ましい
10. サーバーは、[`workspace/executeCommand`] リクエストに含まれるコマンド名を手
    がかりに、該当するコマンドの処理を実行する。ただし、ここでは直接ファイルを書
    き換えたりせず [`WorkspaceEdit`][lsp_types::WorkspaceEdit] の配列を作成して
    [`workspace/applyEdit`][lsp_types::request::ApplyWorkspaceEdit] リクエストを
    クライアントに送信する
    - 直接書き換える方式では、ユーザーが保存していない編集内容が強制的に破棄され
      ることになる

[vscodeapi-contributes.commands]:
  https://code.visualstudio.com/api/references/contribution-points#contributes.commands
[lsp_types::Command]:
  https://docs.rs/lsp-types/latest/lsp_types/struct.Command.html
[lsp_types::Diagnostic]:
  https://docs.rs/lsp-types/latest/lsp_types/struct.Diagnostic.html
[`textDocument/codeAction`]:
  https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_codeAction
[`textDocument/publishDiagnostics`]:
  https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics
[`workspace/executeCommand`]:
  https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_executeCommand
[lsp_types::request::ApplyWorkspaceEdit]:
  https://docs.rs/lsp-types/latest/lsp_types/request/enum.ApplyWorkspaceEdit.html
[lsp_types::ServerCapabilities]:
  https://docs.rs/lsp-types/latest/lsp_types/struct.ServerCapabilities.html
[lsp_types::TextDocumentSyncCapability]:
  https://docs.rs/lsp-types/latest/lsp_types/enum.TextDocumentSyncCapability.html
[lsp_types::CodeActionProviderCapability]:
  https://docs.rs/lsp-types/latest/lsp_types/enum.CodeActionProviderCapability.html
[lsp_types::ExecuteCommandOptions]:
  https://docs.rs/lsp-types/latest/lsp_types/struct.ExecuteCommandOptions.html
[lsp_types::WorkspaceEdit]:
  https://docs.rs/lsp-types/latest/lsp_types/struct.WorkspaceEdit.html
