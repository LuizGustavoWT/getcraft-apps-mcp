# Como contribuir

[English](CONTRIBUTING.md)

Obrigado por ajudar a tornar os aplicativos Craft mais fáceis de usar com agentes. Contribuições de qualquer pessoa são bem-vindas.

## Antes de começar

- Leia a documentação MCP/CLI atual do aplicativo upstream antes de alterar um adaptador, uma skill ou um exemplo.
- Mantenha o gateway independente. Não copie código de implementação, assets nem dependências vendorizadas dos apps upstream.
- Preserve a fronteira de segurança do gateway: nenhuma execução por shell a partir de entrada MCP, nenhum processo MCP nativo aninhado por `craft_run_cli`, execução limitada e autorização de arquivos clara.
- Discuta em uma issue primeiro alterações amplas de arquitetura ou mudanças incompatíveis no schema das ferramentas.

## Ambiente de desenvolvimento

```sh
git clone https://github.com/LuizGustavoWT/getcraft-apps-mcp
cd getcraft-apps-mcp
cargo test
```

Execute estas verificações antes de abrir um pull request:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
claude plugin validate ./plugins/getcraft-apps-mcp
```

Também teste o marketplace do Codex a partir de um checkout local ou fonte GitHub antes de alterar seu empacotamento.

## Pull requests

1. Faça fork do repositório e crie uma branch focada.
2. Adicione ou atualize testes para qualquer comportamento Rust alterado.
3. Atualize juntas as documentações em inglês e português.
4. Inclua links à documentação upstream oficial para qualquer sintaxe CLI/MCP nativa alterada.
5. Informe quais apps Craft e clientes você testou.

Não faça commit de credenciais, arquivos privados de projeto, assets criativos gerados que você não possui, nem bancos de dados locais dos aplicativos. Mantenha o PR focado e explique qualquer comportamento que escreva, substitua, importe, exporte ou persista arquivos.

## Política de manutenção

O projeto aceita issues públicas e pull requests por fork. A branch `main` exige a revisão do proprietário conforme `CODEOWNERS`, e somente `@LuizGustavoWT` possui acesso de escrita/administração. Apenas essa conta faz merge de pull requests e fecha issues abertas por outras pessoas.

## Relato de vulnerabilidades

Não publique uma vulnerabilidade em uma issue. Use o fluxo de security advisory privado do GitHub para este repositório quando ele estiver disponível, ou entre em contato com o proprietário pelo GitHub.
