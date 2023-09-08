# 開発メモ

## エラー例

- minute value of an env of time-range exceeds 99: `- 09:45-23:100`

## 設計メモ

```plaintext
message_loop(client):
    service_state: {
        "url": {
            "line_mapper": LineMapper,
            "diagnostics": [Diagnostic],
        },
    }

    for msg in client.recv():
        if msg is <textDocument/didOpen> or msg is <textDocument/didChange>:
            lint(msg.url) => line_mapper, diagnostics
            service_state[msg.url] = {
                "line_mapper": line_mapper,
                "diagnostics": diagnostics,
            }

        elif msg is <textDocument/codeAction>:
            actions = msg.diagnostics
                         .map(|d| available_actions_for(d.code))
                         .flatten()
            client.send(<textDocument/publishDiagnostics>, actions)

        elif msg is <workspace/executeCommand>:
            if msg.command == "journalint.autofix":
                url, range = msg.arguments
                diagnostic = find_matching_diagnostics_in(service_state)
                workspace_edit = make_workspace_edit(url, diagnostic)
                client.send(<textDocument/applyEdit>, [workspace_edit])
```

## 日誌

### 2023-05-04

大まかな構造を chumsky で実施し、パース失敗したならそのときに出てきたエラーを
diagnostic として返す。

大まかな構造をパースした結果は AST に相当するデータとする。

その構造に対して各種の lint を追加で行う。 lint は、基本的にエラー終了しない。

### 2023-07-15

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

### 2023-07-18

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

### 2023-07-30

比較的マジメに AST っぽいものを作って処理するアプローチに変えたので、Ruff の設計
を改めて勉強させてもらった。まず文法チェック (lint) は AST を Visitor パターンで
traverse する中で、ある種別のノードを発見したときに呼ばれる処理、という形で複数
のチェック処理を実行して Diagnostic を生成している。 Auto-fix については、Edit
のリストである Fix を、 Diagnostic を生成する際に付与可能なルールで、かつ付与す
べきオプションが指定されているようであれば付与するようになっている。これの実行処
理は、トリガーする側がどの Diagnostic に対する fix なのかを指定してきているはず
なので、おそらく素直に該当する Diagnostic から fix を取り出して実行するだけだと
思う。

### 2023-08-06

front matter の date, start, end を別々の文法上のノードとして定義してパーサーを
構成していたが、その実現のために (date or start or end) というパーサーを repeat
させていた。正常系であれば、たしかに問題なく順不同で各フィールドの値を区別して取
得できるのだけれど、一つでも不足または accept 不可能な内容になっていたりすると、
repeat の部分でのエラーということになり分かりにくいエラーメッセージが表示される
問題が起こった (unexpectedly found an end of input といった内容)。これは、おそら
くパーサーとしては素直でない作りを採用していたことに根本原因があると考え、素直に
front matter のフィールド解釈はフィールドの種別を区別せず行うように変更した方が
良いかもしれない。

### 2023-08-19

Quick Fix / Code action の実装を開始する。LSP 仕様で関連するメッセージは
`textDocument/codeAction`。VSCode で軽く試すと、カーソル移動のたびに
`textDocument/codeAction` のメッセージが飛んできて非同期な実行を要求される。また
、それを無視する実装のままでカーソルを動かしたりすると `$/cancelRequest` の通知
が飛んでくる。

### 2023-08-27

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

### 2023-08-30

コマンドでの autofix が実装できたので、今度は言語サーバーとして Code action に対
応していく。以下のように処理の流れが整理できると思う:

1. サーバーは、初期化フェーズにおいて以下の [Server
   Capability][lsp_types::ServerCapabilities] をクライアントに申告する
   - [Text Document Sync][lsp_types::TextDocumentSyncCapability]
   - [Code Action Provider][lsp_types::CodeActionProviderCapability]
   - [Execute Command Provider][lsp_types::ExecuteCommandOptions]
2. サーバーは、[`textDocument/didOpen` 通知][textDocument/didOpen]および
   [`textDocument/didChange` 通知][textDocument/didChange]を受信するたびにエラー
   チェックを行って [`Diagnostic`][lsp_types::Diagnostic] を作成し
   、[`textDocument/publishDiagnostics` 通知][textDocument/publishDiagnostics]で
   クライアントにそれらを報告する
3. クライアントは、報告された Diagnostic を UI で提示する。
   - VSCode の場合、エラーであれば赤い下波線が、警告であればオレンジ色の下波線が
     、そのエラーまたは警告の対象となる部分に対して引かれる。
4. クライアントは、ユーザーが Diagnostic のいずれかにマウスカーソルやテキストカ
   ーソルを合わせると、その位置で使用可能な Code Action の一覧を
   [`textDocument/codeAction` 要求][textDocument/codeAction] メッセージでサーバ
   ーから取得する。このメッセージには、ファイルの URL、カーソル位置、その位置に
   該当する Diagnostic 一覧などが添えられる。
   - VSCode の場合、「豆電球 (Code Action lightbulb)」を表示して「そこで何らかの
     Code Action を実行できますよ」と教えてくれる。この UI を実現するために、カ
     ーソル位置が変更されるたびにサーバーへ一覧を要求するようだ。なお豆電球は設
     定で無効化できる。
   - 一般論としては「豆電球」UI は必須ではないため Code Action を実行する操作を
     したタイミングで使用可能な Code Action 一覧を取得しても良いだろう。
5. サーバーは、指定されたカーソル位置と添付された
   [`Diagnostic`][lsp_types::Diagnostic] のリストから code を手がかりに、実行可
   能な[コマンド][lsp_types::Command]をリストアップしてクライアントに返送する
6. クライアントは、サーバーから得た実行可能なコマンドの一覧を UI に提示する
7. ユーザーは、実行可能なコマンドのうち一つを選択する
8. クライアントは、選択されたコマンドをサーバーに実行するよう要求する
   - [`workspace/executeCommand`] リクエストを使用
   - VSCode の場合、コマンドの名前空間が VSCode および VSCode 専用拡張機能と共有
     なので、サーバーがクライアントに通達するコマンド名は VSCode や、その他の拡
     張機能などと重複しないような名前にすることが強く推奨される。この話は LSP と
     無関係な VSCode の拡張機能でも共通する注意点でもある。それらの場合は
     `{拡張機能名}.{コマンド名}` といった命名規則にするのが一般的であるようなの
     で、 LSP でも同様にすれば良いと思う。
9. サーバーは、[`workspace/executeCommand`] リクエストに含まれるコマンド名を手が
   かりに、該当するコマンドの処理を実行する。ただし、ここでは直接ファイルを書き
   換えたりせず [`WorkspaceEdit`][lsp_types::WorkspaceEdit] の配列を作成して
   [`workspace/applyEdit`][lsp_types::request::ApplyWorkspaceEdit] リクエストを
   クライアントに送信する
   - 直接書き換える方式では、ユーザーが保存していない編集内容が強制的に破棄され
     ることになる

補足。

- `workspace/execteCommand` で取り扱う「コマンド」は、一意な名前の付いたクライア
  ントが呼び出せる処理を指す。 多くの VSCode ユーザーはキーボードショートカット
  を設定するときに見かけていると思う（例: インデントを増やすコマンドの名前は
  `editor.action.indentLines`）。この「エディタが実行可能なコマンドに一意の名前
  が付けられており、コマンド名を指定すれば適切な処理を実行できる」という考え方は
  LSP というプロトコルがエディタ（クライアント）に対して暗に要求している仕様とも
  言える。

## 2023-09-04

Journalint が content、文書の内容全体への「参照」を保持することでサーバーを書き
にくくなっている。というのも、サーバー稼働時は textDocument/didChange などのメッ
セージに含まれるファイル内容を参照して Journalint インスタンスを生成するが、その
インスタンスは追って届くであろう workspace/executeCommand に反応するために保存し
ておきたい。すると、textDocument/didChange のメッセージパラメータをスタックから
破棄した後に Journalint インスタンスを使いたい、ということになるためライフタイム
制約に引っかかってしまう。

そもそも Journalint インスタンスが処理した文書データ全体への参照を持っていなけれ
ばならないというのは不自然な話。調べると、CLI での report 表示用にファイル名とフ
ァイル内容を保持していただけだった。そして CLI でのレポート表示を行う文脈では当
該データが普通にアクセス可能な状態になっている。したがって、これらのデータは
Journalint インスタンスから削除するのが良いと思われる。

[lsp_types::Command]:
  https://docs.rs/lsp-types/latest/lsp_types/struct.Command.html
[lsp_types::Diagnostic]:
  https://docs.rs/lsp-types/latest/lsp_types/struct.Diagnostic.html
[textDocument/codeAction]:
  https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_codeAction
[textDocument/didChange]:
  https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_didChange
[textDocument/didOpen]:
  https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_didOpen
[textDocument/publishDiagnostics]:
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
