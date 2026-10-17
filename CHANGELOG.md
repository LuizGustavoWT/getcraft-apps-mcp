# Changelog / Histórico

## 0.2.0 — 2026-10-17

### English

- Add `craft_app_info`, `getcraft-mcp doctor` and `--version`; record seven upstream releases, audited commits and locally tested CLI versions in `upstream-versions.json`.
- Add paginated `craft_search_commands`, including LightCraft control discovery. Preserve native catalog JSON and request FilmCraft JSON, EffectCraft schemas and LightCraft JSON explicitly.
- Extend native configuration with PdfCraft/LightCraft compact mode, PhotoCraft bridge token-file paths, EffectCraft repeated roots and headless autosave. Support newer PdfCraft/DesignCraft root flags only when advertised by the installed binary.
- Explicitly select VectorCraft headless mode; reject unsupported fields, invalid mode combinations and non-loopback bridges. Clarify each application's actual filesystem defaults.
- Harden finite CLI execution with null stdin and persistent-subcommand detection even after options, including LightCraft `denoise-serve`, without blocking action names or other data values. Clean up Unix process groups on completion, timeout or cancellation; document non-Unix and partial-output limitations.
- Refresh all seven skills and English/PT-BR installation, update and contribution guides. Distinguish shipped features from newer upstream code; updating this plugin does not update native apps.
- Add portable package checks and MCP smoke tests, with an optional read-only native-CLI mode for local verification.

### Português (Brasil)

- Adiciona `craft_app_info`, `getcraft-mcp doctor` e `--version`; registra releases, commits auditados e versões de CLI testadas dos sete apps em `upstream-versions.json`.
- Adiciona `craft_search_commands` paginado, incluindo controles do LightCraft. Preserva o JSON nativo e solicita explicitamente JSON do FilmCraft, schemas do EffectCraft e JSON do LightCraft.
- Amplia a configuração nativa com modo compacto PdfCraft/LightCraft, caminho de token no bridge PhotoCraft, raízes repetidas e autosave headless do EffectCraft. Aceita raízes novas de PdfCraft/DesignCraft somente se o binário instalado anunciar as flags.
- Seleciona headless do VectorCraft explicitamente; rejeita campos não suportados, combinações de modo inválidas e bridges fora de loopback. Esclarece as permissões padrão reais de cada aplicativo.
- Reforça a execução finita com entrada padrão fechada e detecção de subcomandos persistentes após opções, incluindo `denoise-serve` do LightCraft, sem bloquear nomes de ações ou outros valores. Encerra grupos de processos Unix ao concluir, atingir timeout ou cancelar; documenta limitações não Unix e saídas parciais.
- Atualiza as sete skills e guias de instalação, atualização e contribuição em inglês/PT-BR. Distingue recursos lançados de código upstream mais novo; atualizar o plugin não atualiza os apps nativos.
- Adiciona verificações portáveis do pacote e smoke tests MCP, com modo nativo opcional e somente leitura para verificação local.

## 0.1.0 — Initial version / Versão inicial

- Rust stdio gateway with five tools, seven skills and Codex/Claude Code marketplaces.
- Gateway stdio Rust com cinco ferramentas, sete skills e marketplaces Codex/Claude Code.
