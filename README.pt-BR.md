# GetCraft Apps MCP

[English](README.md) · [Contribuindo](CONTRIBUTING.pt-BR.md) · [Contributing](CONTRIBUTING.md)

Um gateway MCP em Rust e um bundle de plugin instalável para os aplicativos Craft da Storytold: [FilmCraft](https://github.com/storytold/filmcraft), [PhotoCraft](https://github.com/storytold/photocraft), [PdfCraft](https://github.com/storytold/pdfcraft), [DesignCraft](https://github.com/storytold/designcraft), [EffectCraft](https://github.com/storytold/effectcraft), [LightCraft](https://github.com/storytold/lightcraft) e [VectorCraft](https://github.com/storytold/vectorcraft).

O gateway não tenta reimplementar os servidores MCP, ricos em recursos, desses aplicativos. Ele encontra as CLIs instaladas, consulta a ajuda e o catálogo de comandos atuais, executa comandos finitos sem usar shell e gera a configuração copiável para o MCP nativo de cada aplicativo. As skills incluídas dão ao Codex e ao Claude Code fluxos de trabalho e observações de segurança específicos para cada app.

## Conteúdo

| Componente | Finalidade |
| --- | --- |
| `getcraft-mcp serve` | Servidor MCP stdio em Rust com cinco ferramentas: `craft_list_apps`, `craft_cli_help`, `craft_command_catalog`, `craft_run_cli` e `craft_native_mcp_config`. |
| `plugins/getcraft-apps-mcp/` | Um diretório de plugin compatível com Codex e Claude Code. |
| Sete skills | Instruções focadas em FilmCraft, PhotoCraft, PdfCraft, DesignCraft, EffectCraft, LightCraft e VectorCraft. |
| Dois marketplaces | `.agents/plugins/marketplace.json` para Codex e `.claude-plugin/marketplace.json` para Claude Code. |

## Modelo de segurança

`craft_run_cli` chama uma CLI instalada diretamente — nunca por um shell — e rejeita os subcomandos `mcp` e `serve`, impedindo a abertura de um servidor persistente aninhado. Também limita quantidade/tamanho de argumentos, saída capturada e tempo de execução.

Para edição detalhada, prefira o MCP nativo de cada app, pois ele expõe as ferramentas reais do aplicativo. Use `craft_native_mcp_config` para gerar a configuração. Quando houver confinamento de sistema de arquivos, conceda o menor escopo possível:

| App | Confinamento / observação do MCP nativo |
| --- | --- |
| FilmCraft | A documentação de agentes não oferece flags de raiz do sistema de arquivos; use apenas caminhos explícitos e confiáveis. |
| PhotoCraft | Flags separadas `--automation-read-root` e `--automation-write-root`; uma permissão omitida continua ausente. |
| PdfCraft | `--root` confina todas as leituras e escritas MCP. |
| DesignCraft | Use o modo headless ou de porta de controle documentado. |
| EffectCraft | Use o MCP nativo para schemas completos; `--autosave` persiste checkpoints headless intencionalmente. |
| LightCraft | `--library` é persistente e só pode estar aberta por um processo; para uma biblioteca desktop aberta, use connect mode. |
| VectorCraft | Flags separadas `--automation-read-root` e `--automation-write-root`; uma permissão omitida continua ausente. |

## Pré-requisitos

- Rust 1.88 ou superior.
- Ao menos uma CLI Craft compatível no `PATH`, ou um caminho absoluto na variável adequada: `GETCRAFT_FILMCRAFT_CLI`, `GETCRAFT_PHOTOCRAFT_CLI`, `GETCRAFT_PDFCRAFT_CLI`, `GETCRAFT_DESIGNCRAFT_CLI`, `GETCRAFT_EFFECTCRAFT_CLI`, `GETCRAFT_LIGHTCRAFT_CLI` ou `GETCRAFT_VECTORCRAFT_CLI`.
- Codex ou Claude Code para instalar o plugin.

O gateway não baixa, empacota nem licencia os aplicativos upstream. Instale cada um a partir de suas próprias releases ou instruções oficiais de build.

## Instalar o servidor Rust

Enquanto a publicação no crates.io não for configurada, instale direto do GitHub:

```sh
cargo install --git https://github.com/LuizGustavoWT/getcraft-apps-mcp --bin getcraft-mcp
getcraft-mcp apps
```

Para desenvolvimento local:

```sh
cargo install --path .
getcraft-mcp apps
```

## Instalar no Codex

Instale primeiro o binário. Em seguida, adicione o marketplace e o plugin:

```sh
codex plugin marketplace add LuizGustavoWT/getcraft-apps-mcp --sparse .agents/plugins
codex plugin add getcraft-apps-mcp@getcraft-apps
```

Inicie um novo chat no Codex depois da instalação. O plugin inicia `getcraft-mcp serve` e disponibiliza as sete skills. Para usar somente o servidor:

```sh
codex mcp add getcraft-apps -- getcraft-mcp serve
```

## Instalar no Claude Code

Instale primeiro o binário. Em seguida, adicione o marketplace e instale o plugin:

```sh
claude plugin marketplace add LuizGustavoWT/getcraft-apps-mcp
claude plugin install getcraft-apps-mcp@getcraft-apps
```

Reinicie o Claude Code ou comece uma nova sessão. As skills recebem namespace; por exemplo, `/getcraft-apps-mcp:vectorcraft`. Para usar apenas o servidor:

```sh
claude mcp add getcraft-apps -- getcraft-mcp serve
```

Valide o pacote Claude depois de clonar ou alterar o projeto:

```sh
claude plugin validate ./plugins/getcraft-apps-mcp
```

## Fluxo de exemplo

1. Liste as CLIs nativas com `craft_list_apps`.
2. Use a skill do aplicativo desejado, por exemplo VectorCraft.
3. Para trabalho finito de rotina, chame `craft_cli_help` ou `craft_command_catalog` e depois `craft_run_cli`.
4. Para edição detalhada, chame `craft_native_mcp_config` e adicione diretamente ao cliente o bloco `mcpServers.<app>` retornado.
5. Use caminhos de saída explícitos e verifique uma renderização ou inspeção antes de substituir um original.

## Desenvolvimento

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
claude plugin validate ./plugins/getcraft-apps-mcp
```

O Codex valida o manifesto de compatibilidade e a entrada do marketplace ao adicionar o marketplace a partir do checkout local ou do GitHub.

## Governança

O repositório é público e recebe issues e pull requests. A branch `main` é protegida e `CODEOWNERS` nomeia apenas [@LuizGustavoWT](https://github.com/LuizGustavoWT); somente essa conta possui autoridade de administração/escrita. Assim, colaboradores externos podem propor mudanças por forks e pull requests, enquanto apenas o proprietário pode fazer merge e fechar issues abertas por outras pessoas.

## Licença

Licenciado sob [Apache-2.0](LICENSE). Este projeto é independente e não é afiliado aos projetos Craft upstream da Storytold.
