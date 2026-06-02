════════════════════════════════════════════════════════════════════════
  GNOMAD SLATE
  Gnomad Slate · README.md
════════════════════════════════════════════════════════════════════════

GNOMAD SLATE
============

Version
Stack
Security
Tests

Gnomad Slate is a desktop scratchpad and encrypted secrets vault — quick markdown capture with live preview, clipboard history, and Vault-X for API keys and tokens. Summon with Option+Space from anywhere.

Built with ❤️ by Gnomad Studio 🦙

Repository: github.com/davidthegnomad/gnomad-slate

────────────────────────────────────────

WHY THIS EXISTS
---------------

You need a place to drop thoughts, snippets, and credentials without opening a browser or leaving secrets in plain notes. Slate stays ready in the background: scratchpad notes with markdown preview, persistent vault notes, filtered clipboard history, and a Rust-backed encrypted secret store the UI never holds in full.

────────────────────────────────────────

CAPABILITIES (V0.1.0)
---------------------

  Area         |  What you get                                                     
  Scratchpad   |  Ephemeral markdown notes; auto-archive after 48h                 
  Vault notes  |  Persistent vault_*.md buffers                                    
  Markdown     |  Edit, Split, and Preview modes (marked)                          
  Vault-X      |  AES-256-GCM secrets; Argon2id master password                    
  Security     |  Secrets in Rust only; zeroized heap; rate-limited unlock         
  Clipboard    |  History capsule (⌘⇧V); entropy + prefix filter; toggle in sidebar
  Capture      |  Global shortcut, command palette, hide on blur                   
  Themes       |  Dark, light, system                                              

────────────────────────────────────────

QUICK START
-----------

Developers

Prerequisites: Node.js LTS, Rust stable, Tauri v2 prerequisites

  [bash]
    cd 05_apps_and_extensions/gnomad-slate
    npm install
    npm run tauri dev      # Desktop + HMR (port 1420)
    npm test               # Vitest (4 tests)
    cd src-tauri && cargo test   # Rust (14 tests)

Production build: npm run tauri build

Users

See docs/USER_GUIDE.md · HTML · TXT

Vault security (passwords & encryption): docs/VAULT_SECURITY.md · HTML

────────────────────────────────────────

ARCHITECTURE (SUMMARY)
----------------------

  [code]
    ┌─────────────────────────────────────────────────────────┐
    │  React UI — notes, markdown preview, vault metadata    │
    └───────────────────────────┬─────────────────────────────┘
                                │ Tauri invoke
    ┌───────────────────────────▼─────────────────────────────┐
    │  Rust — crypto, vault session, notes FS, clipboard     │
    └─────────────────────────────────────────────────────────┘

Deep dive: docs/ARCHITECTURE.md · docs/TECH_STACK.md

────────────────────────────────────────

DOCUMENTATION INDEX
-------------------

All docs ship as Markdown, HTML, and TXT. Full index: docs/DOCS_INDEX.md. Regenerate: npm run docs:export.

  Document                   |  MD                       |  HTML                     
  All docs                   |  docs/DOCS_INDEX.md       |  docs/DOCS_INDEX.html     
  Project site               |  —                        |  docs/index.html          
  User Guide                 |  docs/USER_GUIDE.md       |  docs/USER_GUIDE.html     
  Vault & Password Security  |  docs/VAULT_SECURITY.md   |  docs/VAULT_SECURITY.html 
  Architecture               |  docs/ARCHITECTURE.md     |  docs/ARCHITECTURE.html   
  Test Strategy              |  docs/TEST_STRATEGY.md    |  docs/TEST_STRATEGY.html  
  Build Platforms            |  docs/BUILD_PLATFORMS.md  |  docs/BUILD_PLATFORMS.html
  Changelog                  |  CHANGELOG.md             |  CHANGELOG.html           

────────────────────────────────────────

LICENSE
-------

Private project — see repository settings. Contact Gnomad Studio for licensing.

────────────────────────────────────────

ACKNOWLEDGMENTS
---------------

Built with Tauri, React, Vite, and marked.

════════════════════════════════════════════════════════════════════════
Built with ❤️ by Gnomad Studio 🦙
https://gnomadstudio.org
════════════════════════════════════════════════════════════════════════
