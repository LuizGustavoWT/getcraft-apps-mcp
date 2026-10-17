# GetCraft Apps MCP

[English](README.md) · [Contribuindo](CONTRIBUTING.pt-BR.md) · [Contributing](CONTRIBUTING.md)

Um gateway MCP em Rust e um bundle de plugin instalável para os aplicativos Craft da Storytold: [FilmCraft](https://github.com/storytold/filmcraft), [PhotoCraft](https://github.com/storytold/photocraft), [PdfCraft](https://github.com/storytold/pdfcraft), [DesignCraft](https://github.com/storytold/designcraft), [EffectCraft](https://github.com/storytold/effectcraft), [LightCraft](https://github.com/storytold/lightcraft) e [VectorCraft](https://github.com/storytold/vectorcraft).

O gateway encontra as CLIs instaladas, verifica versões, pesquisa seus catálogos de comandos, executa comandos finitos sem shell e gera configurações para o MCP nativo de cada aplicativo. Ele não reimplementa os editores nem suas ferramentas MCP. As sete skills dão ao Codex e ao Claude Code fluxos de trabalho e observações de segurança específicos para cada app.

## Conteúdo

| Componente | Finalidade |
| --- | --- |
| `getcraft-mcp serve` | Servidor MCP stdio em Rust com as sete ferramentas abaixo. |
| `getcraft-mcp doctor` | Versões das CLIs instaladas e snapshot da auditoria upstream; `apps` faz a descoberta básica e `--version` mostra a versão do gateway. |
| `plugins/getcraft-apps-mcp/` | Um diretório de plugin compatível com Codex e Claude Code. |
| Sete skills | Instruções focadas em FilmCraft, PhotoCraft, PdfCraft, DesignCraft, EffectCraft, LightCraft e VectorCraft. |
| Dois marketplaces | `.agents/plugins/marketplace.json` para Codex e `.claude-plugin/marketplace.json` para Claude Code. |
| [Snapshot upstream](upstream-versions.json) | Tags de release, commits auditados exatos, versões de CLI testadas e notas sobre recursos ainda não lançados. |

## Ferramentas MCP

| Ferramenta | Finalidade |
| --- | --- |
| `craft_list_apps` | Encontrar executáveis das CLIs e suas variáveis de configuração. |
| `craft_app_info` | Consultar a versão instalada de um app e as informações de compatibilidade registradas. |
| `craft_cli_help` | Ler a ajuda da CLI instalada antes de montar um comando. |
| `craft_command_catalog` | Retornar o catálogo nativo completo, com o JSON original em `stdout` no resultado. |
| `craft_search_commands` | Pesquisar e paginar o catálogo instalado sem carregar tudo no contexto. |
| `craft_run_cli` | Executar diretamente um comando finito da CLI nativa. |
| `craft_native_mcp_config` | Gerar uma configuração MCP nativa, sem instalá-la nem iniciar o servidor. |

`craft_search_commands` recebe `app` e, opcionalmente, `query`, `kind`, `limit` e `offset`; retorna `{app, kind, total, matched, offset, next_offset, entries}`. O tipo padrão é `commands`; `controls` existe apenas para LightCraft. Os metadados nativos são preservados. Exemplos: `{"app":"photocraft","query":"actions","limit":10}` e `{"app":"lightcraft","kind":"controls","query":"exposure"}`. Use `next_offset` para buscar mais resultados.

## Modelo de segurança

`craft_run_cli` não usa shell, fecha a entrada padrão do processo, limita argumentos/saída e permite no máximo 120 segundos por chamada. Rejeita servidores persistentes (`mcp`, `serve`, `denoise-serve`), mesmo com opções antes do subcomando. Uma falha ou timeout pode ocorrer depois de alterações no documento ou da gravação de saídas parciais; inspecione antes de repetir. Para trabalhos longos e sessões persistentes, use o MCP nativo.

No Unix, cada execução tem seu próprio grupo de processos, encerrado ao concluir, atingir o timeout ou ser cancelada. Isso também encerra os processos auxiliares comuns da CLI. Processos que saem deliberadamente desse grupo ficam fora dessa garantia; em plataformas não Unix, atualmente apenas o filho direto é encerrado.

O gateway **não é uma sandbox do sistema operacional**: comandos finitos da CLI usam as permissões da sua conta. As raízes de uma configuração MCP nativa gerada não restringem `craft_run_cli`. Conceda apenas os caminhos necessários ao trabalho:

| App | Confinamento / observação do MCP nativo |
| --- | --- |
| FilmCraft | Não possui flags nativas de confinamento; use caminhos explícitos e confiáveis. |
| PhotoCraft | Raízes de leitura/escrita independentes no headless; raízes ausentes não concedem acesso a arquivos. O bridge usa raízes e autenticação do desktop, não flags de raiz do gateway. |
| PdfCraft | `workspace` vira `--root` para leitura e escrita. Raízes de automação separadas surgiram após a v0.8.0; ao combinar, ambas as restrições valem. Sem raízes, o acesso é irrestrito. |
| DesignCraft | Raízes de automação separadas surgiram após a v0.8.0, apenas no headless. O bridge desktop não é confinado. Sem raízes, o acesso é irrestrito. |
| EffectCraft | Raízes repetíveis: leitura nas raízes de leitura ou escrita; gravação apenas nas de escrita. Sem raízes, não há confinamento. `autosave` persiste checkpoints headless intencionalmente. |
| LightCraft | `workspace` seleciona uma biblioteca persistente, **não uma sandbox**. Apenas um processo detém o lock; conecte ao desktop para usar uma biblioteca já aberta nele. |
| VectorCraft | Configurações headless usam `--headless` explicitamente. Com uma raiz, o acesso omitido é negado; sem ambas, não há confinamento. No bridge, configure as raízes no próprio desktop. |

`craft_native_mcp_config` aceita `mode: "headless"` (padrão) ou `"bridge"`, com endereço `connect` restrito a loopback para bridges. Opções não suportadas ou combinações de modo inválidas causam erro, sem serem ignoradas. As raízes de PdfCraft e DesignCraft são conferidas na ajuda da CLI instalada; um binário antigo é recusado, nunca substituído por acesso irrestrito.

Campos opcionais: `compact` para PdfCraft/LightCraft; `control_token_file` no bridge PhotoCraft; `autosave` no EffectCraft headless; arrays adicionais `read_roots`/`write_roots` para as flags repetidas do EffectCraft. O campo de token PhotoCraft recebe o caminho de um arquivo, não o segredo; o gateway nunca lê seu conteúdo. Bridges PhotoCraft/VectorCraft rejeitam campos de raiz — configure-os no desktop. Em PdfCraft/DesignCraft, informar apenas uma raiz de automação nega o outro acesso; omitir ambas desativa essa política adicional.

## Pré-requisitos

- Rust 1.88 ou superior.
- Ao menos uma CLI Craft compatível no `PATH`, ou o caminho absoluto do executável em `GETCRAFT_<APP>_CLI`, como `GETCRAFT_PHOTOCRAFT_CLI` ou `GETCRAFT_VECTORCRAFT_CLI`.
- Codex ou Claude Code para instalar o plugin.

O gateway não baixa, empacota nem licencia os aplicativos upstream. Instale cada um a partir de suas próprias releases ou instruções oficiais de build.

## Instalar ou atualizar o servidor Rust

Enquanto a publicação no crates.io não for configurada, instale direto do GitHub:

```sh
cargo install --git https://github.com/LuizGustavoWT/getcraft-apps-mcp --locked --force --bin getcraft-mcp
getcraft-mcp --version
getcraft-mcp doctor
```

Para desenvolvimento local:

```sh
cargo install --path . --locked --force --bin getcraft-mcp
```

## Instalar no Codex

Instale primeiro o binário. Em seguida, adicione o marketplace e o plugin:

```sh
codex plugin marketplace add LuizGustavoWT/getcraft-apps-mcp --sparse .agents/plugins --sparse plugins
codex plugin add getcraft-apps-mcp@getcraft-apps
```

Se o marketplace tiver sido adicionado anteriormente apenas com `--sparse .agents/plugins`, substitua esse checkout incompleto antes de instalar:

```sh
codex plugin marketplace remove getcraft-apps
codex plugin marketplace add LuizGustavoWT/getcraft-apps-mcp --sparse .agents/plugins --sparse plugins
```

As duas pastas sparse são necessárias: `.agents/plugins` contém o catálogo e `plugins` contém o plugin. Para um marketplace Git já configurado, atualize o binário acima, depois atualize o marketplace e reinstale seu plugin:

```sh
codex plugin marketplace upgrade getcraft-apps
codex plugin add getcraft-apps-mcp@getcraft-apps
```

Preserve as duas pastas sparse ao alterar a origem do marketplace. Esse é o [fluxo oficial de marketplaces do Codex](https://developers.openai.com/plugins/build/plugins). Inicie um novo chat após instalar ou atualizar para carregar ferramentas e skills novas. O plugin inicia `getcraft-mcp serve`. Para usar somente o servidor:

```sh
codex mcp add getcraft-apps -- getcraft-mcp serve
```

## Instalar no Claude Code

Instale primeiro o binário. Em seguida, adicione o marketplace e instale o plugin:

```sh
claude plugin marketplace add LuizGustavoWT/getcraft-apps-mcp
claude plugin install getcraft-apps-mcp@getcraft-apps
```

Reinicie o Claude Code ou comece uma nova sessão. As skills recebem namespace; por exemplo, `/getcraft-apps-mcp:vectorcraft`. Para atualizar o pacote, atualize o binário Rust acima e use o gerenciador de plugins do Claude Code para atualizar o marketplace/plugin. Para usar apenas o servidor:

```sh
claude mcp add getcraft-apps -- getcraft-mcp serve
```

Valide o pacote Claude depois de clonar ou alterar o projeto:

```sh
claude plugin validate ./plugins/getcraft-apps-mcp
```

## Fluxo de exemplo

1. Encontre as CLIs com `craft_list_apps` e confira o app escolhido com `craft_app_info`.
2. Use sua skill e `craft_search_commands` para localizar operações na versão instalada.
3. Leia a ajuda e os parâmetros atuais antes de chamar `craft_run_cli`; cada execução finita cria uma nova sessão nativa.
4. Para edição detalhada, chame `craft_native_mcp_config` e adicione diretamente ao cliente o bloco `mcpServers.<app>` retornado.
5. Use caminhos de saída explícitos e verifique uma renderização ou inspeção antes de substituir um original.

## Versões upstream auditadas

Snapshot verificado em **2026-10-17**; “CLI testada” significa verificações locais de compatibilidade somente leitura, não validação exaustiva de cada recurso dos editores. Commits exatos e links de documentação estão em [upstream-versions.json](upstream-versions.json).

| App | Última release na auditoria | CLI testada |
| --- | --- | --- |
| FilmCraft | 0.8.0 | 0.8.0 |
| PhotoCraft | 0.9.0 | 0.9.0 |
| PdfCraft | 0.8.0 | 0.8.0 |
| DesignCraft | 0.8.0 | 0.8.0 |
| EffectCraft | 0.10.0 | 0.10.0 |
| LightCraft | 0.8.0 | 0.8.0 |
| VectorCraft | 0.12.0 | 0.11.0 |

Atualizar este plugin **não** atualiza os aplicativos. Os recursos do VectorCraft 0.12.0 exigem atualizar o app nativo separadamente. O `main` auditado do EffectCraft declara 0.11.0, mas a última release publicada na auditoria é 0.10.0. Algumas opções de confinamento de PdfCraft/DesignCraft e fluxos do LightCraft também são posteriores às releases. As skills condicionam seu uso à ajuda e aos catálogos instalados; o snapshot não consulta releases em tempo real. Mudanças do gateway: [CHANGELOG.md](CHANGELOG.md).

## Desenvolvimento

Com Python 3.11+ (somente biblioteca padrão):

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
python3 scripts/check_package.py
cargo build --locked
python3 scripts/smoke_mcp.py target/debug/getcraft-mcp
```

O smoke test padrão não precisa de apps Craft. Com as sete CLIs instaladas, acrescente `--native` para verificações nativas somente leitura. Execute `claude plugin validate ./plugins/getcraft-apps-mcp` quando o Claude Code estiver disponível; valide a instalação real no cliente separadamente dos testes do pacote. Detalhes: [CONTRIBUTING.pt-BR.md](CONTRIBUTING.pt-BR.md).

## Governança

O repositório é público e recebe issues e pull requests. A branch `main` é protegida e `CODEOWNERS` nomeia apenas [@LuizGustavoWT](https://github.com/LuizGustavoWT); somente essa conta possui autoridade de administração/escrita. Assim, colaboradores externos podem propor mudanças por forks e pull requests, enquanto apenas o proprietário pode fazer merge e fechar issues abertas por outras pessoas.

## Licença

Licenciado sob [Apache-2.0](LICENSE). Este projeto é independente e não é afiliado aos projetos Craft upstream da Storytold.
