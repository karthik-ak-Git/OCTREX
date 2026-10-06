# OCTREX CODE V4 — Desktop UI Architecture & Integration Specification

**Phase 0 Audit & Architecture Plan**  
**Target Platform:** Desktop Native Application (Windows / Cross-Platform)  
**Technology Stack:** Electron + React 18 + TypeScript + Tailwind CSS + Framer Motion + Vite/ESBuild

---

## 1. Executive Summary & Objective

The objective is to replace the presentation layer of **OCTREX CODE V4** with a sleek, pixel-perfect, highly responsive native desktop interface inspired by modern AI coding assistants (Claude Code, ChatGPT, Codex). 

All core backend capabilities (**Universal Model Gateway**, **Multi-Agent Orchestrator**, **Repository Intelligence**, **M4 Cloud-Code Consent Guard**, **Dynamic Crax-GPT Discovery**, and **Empirical Verification Engine**) remain intact and are integrated seamlessly via typed Electron IPC (Inter-Process Communication).

---

## 2. Backend Integration & IPC Contract Mapping

```
┌────────────────────────────────────────────────────────────────────────┐
│                   RENDERER PROCESS (React + Tailwind)                  │
│  - Sidebar (Collapsible)  - Chat Feed (Streaming) - Input Bar (@file) │
│  - Route Badges (Logos)   - Code Blocks (Copy)    - Verification Card  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ (Typed IPC via contextBridge)
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                 PRELOAD SCRIPT (`window.octrexCore`)                   │
│  - invoke(channel, ...args)       - onEvent(callback)                  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ (Electron ipcMain / EventEmitter)
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                     ELECTRON MAIN / BACKEND CORE                       │
│  ┌─────────────────────────┐      ┌─────────────────────────────────┐  │
│  │ Universal Model Gateway │      │   Agent Orchestrator & Tasks    │  │
│  │ (crax-gpt, Gemini, etc.)│      │   (Planner, Coder, Tester, etc.)│  │
│  └────────────┬────────────┘      └────────────────┬────────────────┘  │
│               │                                    │                   │
│  ┌────────────▼────────────┐      ┌────────────────▼────────────────┐  │
│  │ UIEventEmitter (Events) ├─────►│ Verification Engine & Repo Index│  │
│  └─────────────────────────┘      └─────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

### IPC Channel Registry

| Channel | Type | Payload / Params | Returns | Description |
| :--- | :--- | :--- | :--- | :--- |
| `octrex:task:execute` | `invoke` | `{ prompt: string, workspacePath?: string }` | `TaskResult` | Launches multi-agent task lifecycle |
| `octrex:task:cancel` | `invoke` | `{ taskId: string }` | `boolean` | Halts in-flight task immediately |
| `octrex:providers:list` | `invoke` | `void` | `ProviderMetadata[]` | Lists registered providers, health, logos |
| `octrex:crax:connect` | `invoke` | `{ apiKey: string }` | `{ success: boolean }` | Connects & validates crax-gpt API key |
| `octrex:crax:refresh` | `invoke` | `void` | `ModelDescriptor[]` | Dynamically discovers `/v1/models` |
| `octrex:strategy:set` | `invoke` | `{ mode: RoutingStrategyMode }` | `{ success: boolean }` | Changes routing mode (AUTO, POWERFUL, etc.) |
| `octrex:consent:toggle` | `invoke` | `void` | `{ consent: boolean }` | Toggles backend cloud-code consent (M4) |
| `octrex:repo:status` | `invoke` | `void` | `{ fileCount: number, path: string }` | Returns workspace intelligence status |
| `octrex:events:stream` | `on` (Event) | `(event: UIEventPayload) => void` | `Unsubscribe` | Streams real-time backend agent events |

---

## 3. UI Component Hierarchy & Architecture

```
App
├── TitleBar (Custom frameless window controls & drag region)
├── Layout (Flex row, full screen viewport)
│   ├── Sidebar (Collapsible with Framer Motion)
│   │   ├── BrandHeader (OCTREX CODE V4 + Version badge)
│   │   ├── NewChatButton (Shortcuts: Ctrl+N / Cmd+N)
│   │   ├── SessionHistoryList (Recent tasks & conversations)
│   │   ├── ProviderRegistrySection (Gemini, OpenRouter, crax-gpt, NVIDIA, Groq, Ollama)
│   │   │   └── CraxGptSetupCard (1-Click Get Key, Connect, Dynamic Model Picker)
│   │   └── WorkspaceContextCard (Indexed files, Cloud Consent status)
│   │
│   └── MainChatContainer (Flex col, flex-1)
│       ├── HeaderBar
│       │   ├── SidebarToggle (Hamburger button)
│       │   ├── ActiveRouteBadge ([logo] crax-gpt • GLM-5.3 with live fallback indicator)
│       │   ├── StrategyPillSelector (AUTO | POWERFUL | FAST | FREE | LOCAL)
│       │   └── CloudConsentBadge (Granted / Revoked indicator)
│       │
│       ├── ChatScrollArea (Auto-scroll on stream, smooth scroll)
│       │   ├── WelcomeBanner (When thread is empty)
│       │   └── MessageList
│       │       ├── UserMessageBubble (Clean right/left aligned card)
│       │       └── AssistantMessageBlock
│       │           ├── MultiAgentStepsAccordion (Planner, Coder, Tester reasoning chain)
│       │           ├── StreamingMarkdownContent (Framer Motion smooth token reveal)
│       │           │   └── CodeBlock (Syntax highlighted with 1-click "Copy Code" button)
│       │           └── EmpiricalVerificationReportCard (PASS badges, tests count, build status)
│       │
│       └── ChatInputArea (Sticky bottom)
│           ├── AttachmentPills (@file references, active context snippets)
│           ├── ExpandingTextArea (Auto-resizing on multiline, Shift+Enter for newline)
│           └── InputActionBar
│               ├── ModeTag (e.g. "Agent Mode")
│               ├── StopGenerationButton (Active when streaming)
│               └── SendButton (Icon with hover/active spring animations)
```

---

## 4. Visual Aesthetics & Styling Specifications

* **Theme:** Dark mode default with rich obsidian slate backgrounds (`#0B0F17`, `#111622`, `#161F30`).
* **Typography:** Inter for user interface elements, `Fira Code` / `JetBrains Mono` for code blocks, badges, and file paths.
* **Borders & Elevation:** Subtle 1px translucent borders (`border-[#1F293D]`, `border-purple-500/30`), frosted backdrops (`backdrop-blur-md`).
* **Brand Accents:**
  * `crax-gpt`: Neon Purple / Cyan (`#A855F7`, `#06B6D4`)
  * `Gemini`: Google Multimodal Gradient (`#4E82EE` → `#9C52FD` → `#DE4396`)
  * `OpenRouter`: Royal Indigo (`#6366F1`)
  * `NVIDIA`: NIM Green (`#76B900`)
  * `Groq`: Orange Speed Chevron (`#F97316`)
  * `Verified Status`: Emerald Green (`#10B981`)

---

## 5. Animations & Performance Strategy (60fps)

1. **Streaming Text Engine:**
   * Efficient token batching (~16ms RAF buffer) to prevent DOM reflow thrashing during high-speed generation.
2. **Framer Motion Elements:**
   * Sidebar Collapse/Expand: `layout`, spring transition `{ type: "spring", stiffness: 350, damping: 30 }`.
   * Message Entry: `initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }}`.
   * Accordion & Report Cards: `AnimatePresence` with height auto animation.
3. **Zero Main-Thread Blocking:**
   * LLM network requests, file indexing, and background tool execution occur in the backend process; only UI update events cross the IPC bridge.

---

## 6. Desktop Packaging Architecture (`electron-builder`)

* **Packaging Tool:** `electron-builder`
* **Artifacts:**
  * Windows NSIS Installer (`.exe`)
  * Portable Standalone Executable (`.exe`)
* **Bundle Structure:**
  * `dist/main.js` (Compiled Electron Main Process + OCTREX Core)
  * `dist/preload.js` (Context Bridge)
  * `dist/renderer/` (Vite/React Production Assets)

---

## 7. Ordered Phase Execution Plan

| Phase | Milestone | Gate Verification |
| :--- | :--- | :--- |
| **Phase 0** | **Audit & Architecture** | Complete `docs/ARCHITECTURE.md` approval *(Current Gate)* |
| **Phase 1** | **Desktop Scaffold** | Electron + React + Tailwind + IPC bridge initializes blank window |
| **Phase 2** | **Core UI & Chat Interface** | Sidebar, Message Feed, Input Box, Markdown syntax highlighting & Copy button |
| **Phase 3** | **Animations & State Integration** | Framer Motion streaming text, multi-agent pipeline connected to live backend |
| **Phase 4** | **Desktop Packaging** | `electron-builder` configuration, executable build & verification report |

---
*Ready for Phase 0 review and approval to proceed with Phase 1 Scaffold.*
