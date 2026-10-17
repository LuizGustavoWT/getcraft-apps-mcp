# Como contribuir

[English](CONTRIBUTING.md)

Obrigado por ajudar a tornar os aplicativos Craft mais fáceis de usar com agentes. Contribuições de qualquer pessoa são bem-vindas.

## Antes de começar

- Leia a documentação MCP/CLI atual do aplicativo upstream antes de alterar um adaptador, uma skill ou um exemplo.
- Compare tags de release, o `main` auditado e a ajuda instalada: um recurso no código-fonte pode não estar lançado. Atualize [upstream-versions.json](upstream-versions.json) com commits exatos, versões testadas e notas de recursos não lançados; não apresente auditoria de código como teste de execução.
- Mantenha o gateway independente. Não copie código de implementação, assets nem dependências vendorizadas dos apps upstream.
- Preserve a fronteira de segurança: sem shell, entrada padrão do processo fechada, nenhum servidor persistente em `craft_run_cli`, execução limitada, bridges restritos a loopback e autorização de arquivos clara. Restrições solicitadas e não suportadas devem causar erro, nunca desaparecer silenciosamente. Não leia o conteúdo de tokens de autenticação para logs ou saídas de testes.
- Discuta em uma issue primeiro alterações amplas de arquitetura ou mudanças incompatíveis no schema das ferramentas.

## Ambiente de desenvolvimento

```sh
git clone https://github.com/LuizGustavoWT/getcraft-apps-mcp
cd getcraft-apps-mcp
cargo test --locked
```

Execute estas verificações antes de abrir um pull request (Python 3.11+, somente biblioteca padrão):

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
python3 scripts/check_package.py
cargo build --locked
python3 scripts/smoke_mcp.py target/debug/getcraft-mcp
```

O smoke test padrão verifica inicialização e descoberta de ferramentas MCP sem exigir CLIs Craft. Com as sete CLIs nativas instaladas, execute também:

```sh
python3 scripts/smoke_mcp.py target/debug/getcraft-mcp --native
```

As verificações nativas são somente leitura; teste alterações de documentos apenas com fixtures descartáveis e saídas explícitas. Cubra preservação de JSON/schemas do catálogo, paginação da busca, opções não suportadas, rejeição de servidores, timeouts e erros de confinamento ao alterar esses comportamentos.

Quando disponível, execute `claude plugin validate ./plugins/getcraft-apps-mcp` e teste a instalação no Codex separadamente. Não declare validação aprovada se o cliente estiver ausente. Mantenha alinhadas as versões dos manifestos e do Cargo. Exemplos de marketplaces Git devem incluir `.agents/plugins` e `plugins` no sparse checkout ou usar checkout completo. O README descreve a atualização normal do marketplace Git; caches de desenvolvimento local são separados de versões publicadas.

## Pull requests

1. Faça fork do repositório e crie uma branch focada.
2. Adicione ou atualize testes para qualquer comportamento Rust alterado.
3. Atualize juntas as documentações em inglês e português, além de `CHANGELOG.md` e do snapshot quando aplicável.
4. Inclua links à documentação upstream oficial para qualquer sintaxe CLI/MCP nativa alterada.
5. Informe quais versões de apps Craft e clientes você testou, separando testes simulados, leitura nativa e edição real.

Não faça commit de credenciais, arquivos privados de projeto, assets criativos gerados que você não possui, nem bancos de dados locais dos aplicativos. Mantenha o PR focado e explique qualquer comportamento que escreva, substitua, importe, exporte ou persista arquivos.

## Política de manutenção

O projeto aceita issues públicas e pull requests por fork. A branch `main` exige a revisão do proprietário conforme `CODEOWNERS`, e somente `@LuizGustavoWT` possui acesso de escrita/administração. Apenas essa conta faz merge de pull requests e fecha issues abertas por outras pessoas.

## Relato de vulnerabilidades

Não publique uma vulnerabilidade em uma issue. Use o fluxo de security advisory privado do GitHub para este repositório quando ele estiver disponível, ou entre em contato com o proprietário pelo GitHub.
