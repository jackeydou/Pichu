<p align="center">
  <img src="apps/pichu-client/resources/pichu-home-mark.png" alt="Pichu" width="160" />
</p>

<h1 align="center">Pichu</h1>

<p align="center">
  An open-source desktop coding agent built on the Pi SDK.<br />
  Bring your own model, endpoint, and API key.
</p>

Pichu is a desktop-first coding agent for working across repositories, files,
terminals, browsers, and long-running tasks from one persistent workspace. It
uses the [Pi SDK](https://github.com/earendil-works/pi) for the low-level agent
loop and model integration, while Pichu owns the desktop experience, session
lifecycle, tools, approvals, sandboxing, persistence, and plugin system.

This repository is the starting point for Pichu's public development. The
current codebase is treated as the first public version.

## Highlights

- **Pi-powered agent runtime** — built with `pi-agent-core`, `pi-ai`, and
  `pi-coding-agent` for streaming responses, agent control, and coding tools.
- **Bring your own key (BYOK)** — connect your own provider account, compatible
  endpoint, or local model server instead of depending on a bundled model plan.
- **Coding-native workspace** — work with project files, managed terminals,
  attachments, artifacts, repository context, and persistent chat sessions.
- **Safe local tools** — review sensitive actions through approval controls,
  remembered rules, audit events, and OS-level sandbox policies.
- **Browser and desktop tools** — navigate pages, capture screenshots, add
  annotations, and use browser or computer-control workflows from the agent.
- **Extensible by design** — install portable Agent Plugins with skills and MCP
  servers, plus namespaced Pichu hooks and declarative extensions.
- **Durable state** — SQLite-backed sessions, search, settings, continuations,
  human-input requests, and task history survive app restarts.

## Bring your own model

Pichu is BYOK-first. Add a model from **Settings → Models** and provide the
endpoint and credentials issued by your provider. Pichu currently supports
these API protocols:

- OpenAI Responses
- OpenAI-compatible Chat Completions
- Anthropic Messages
- Google Generative AI

Each model configuration can define its own model ID, display name, base URL,
API key, context window, output limit, reasoning support, and image support.
This makes it possible to use hosted providers, compatible gateways, or local
servers through the same Pichu workflow.

API keys are encrypted with Electron `safeStorage` before they are persisted in
the local Pichu data store. Model summaries sent to the renderer expose only
whether a key exists, not the key itself.

## How Pichu uses Pi

Pichu uses Pi as an SDK, not as an external CLI process or a second application
runtime:

- `@earendil-works/pi-agent-core` provides the low-level `Agent` loop, message
  events, tool definitions, steering, and continuation primitives.
- `@earendil-works/pi-ai` provides model types, provider APIs, streaming
  primitives, and token usage data.
- `@earendil-works/pi-coding-agent` provides reusable coding-tool and agent-file
  helpers that Pichu adapts to its own approval and sandbox boundaries.
- Pichu owns SQLite conversation history, prompts, context construction,
  compaction, tools, approvals, sandboxing, plugins, MCP, multi-agent behavior,
  Electron IPC, and UI.

Pichu is therefore not a GUI wrapper around the Pi CLI and does not use Pi
Packages or Pi Extensions as its plugin contract. See the
[agent runtime design](docs/RFC_PICHU_AGENT_RUNTIME.md) for the full boundary.

## Repository layout

| Path | Purpose |
| --- | --- |
| `apps/pichu-client` | Electron main process, preload bridge, React renderer, resources, and app tests |
| `packages` | Shared clients, bundled plugins, native macOS helpers, and developer tools |
| `docs` | Agent runtime, plugin, browser, IPC, workflow, and release references |

## Getting started

### Requirements

- Node.js 22 or newer
- pnpm `10.30.2`, as declared by the root `packageManager` field
- Platform build tools required by Electron and the native workspace packages

### Install

```bash
git clone <your-fork-or-repository-url>
cd Pichu
pnpm install
```

### Run the desktop app

```bash
pnpm dev
```

Pichu stores normal local app data under `~/.pichu`. To isolate development
data, use the supported command-line options rather than environment variables:

```bash
pnpm dev --pichu-dev-name "Local Development" \
  --pichu-data-root ~/.pichu-dev/local-development
```

## Development

Use focused checks for the surface you changed:

```bash
# Main process and Node-side code
pnpm --filter pichu-client typecheck:node

# Renderer and preload code
pnpm --filter pichu-client typecheck:web

# Full client typecheck
pnpm --filter pichu-client typecheck

# Plugin and agent/tool behavior
pnpm --filter pichu-client test:plugins
pnpm --filter pichu-client test:admin

```

Formatting and linting use Biome:

```bash
pnpm run lint:fix -- <paths>
pnpm run format -- <paths>
```

## Plugin development

Pichu plugins follow the vendor-neutral
[Agent Plugins specification](https://agent-plugins.org/specification). A plugin
can provide Agent Skills and MCP servers. Namespaced Pichu extensions add hooks
and other client-specific behavior without changing the portable core.

Local developer ZIP uploads stay on the user's machine. Pichu validates package
boundaries and extracts archives with limits on archive size, entry count,
uncompressed size, compression ratio, paths, and special files.

Start with:

- [Plugin system reference](docs/PLUGIN_SYSTEM.md)
- [Agent hook reference](docs/CODEX_AGENT_HOOKS.md)

## Security

Pichu treats IPC payloads, plugin packages, model and tool input, files, and
network responses as untrusted boundaries. Credentials, authorization headers,
cookies, and sensitive local paths should never be committed or written to
ordinary logs.

Please report security issues privately to the project maintainers. Do not open
a public issue containing credentials, private data, or working exploit details.

## Contributing

Development changes target the `develop` branch through pull requests. Keep
changes scoped, add focused tests for behavior that can regress, and update the
relevant documentation when a public contract changes.

Read [CONTRIBUTING.md](CONTRIBUTING.md) for setup, code style, testing, security,
and pull-request guidance.

## License

Pichu is open-source software licensed under the
[GNU Affero General Public License v3.0](LICENSE). You may use, study, modify,
and distribute it, including commercially, under the license terms. Modified
versions that are distributed or made available to users over a network must
offer those users the corresponding source code under AGPLv3.

Pi SDK packages are maintained separately by the
[Pi project](https://github.com/earendil-works/pi) and are distributed under
their own MIT license.
