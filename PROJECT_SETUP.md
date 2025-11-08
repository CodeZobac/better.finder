# Global Search Launcher - Project Setup

## Overview
This is a Tauri-based desktop application for Windows that provides a global search launcher accessible via keyboard shortcut (Ctrl+K).

## Technology Stack
- **Frontend**: React 18 + TypeScript
- **Backend**: Rust + Tauri 2.x
- **Styling**: TailwindCSS with custom design tokens
- **Build Tool**: Vite

## Project Structure
```
better-finder/
├── src/                    # React frontend
│   ├── components/         # React components
│   ├── hooks/             # Custom React hooks
│   ├── stores/            # State management
│   ├── types/             # TypeScript type definitions
│   ├── App.tsx            # Main App component
│   ├── main.tsx           # Entry point
│   └── index.css          # Global styles with Tailwind
├── src-tauri/             # Rust backend
│   ├── src/               # Rust source code
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── public/                # Static assets
├── tailwind.config.js     # Tailwind configuration
├── tsconfig.json          # TypeScript configuration (strict mode enabled)
└── vite.config.ts         # Vite build configuration
```

## Configuration

### TypeScript
- Strict mode enabled
- Target: ES2020
- Module: ESNext
- JSX: react-jsx

### TailwindCSS
Custom design tokens configured for dark theme:
- Background: #1e1e2e
- Surface: #2a2a3e
- Primary: #89b4fa
- Text colors and more

### Vite
- Port: 1420 (fixed for Tauri)
- HMR enabled
- Optimized for Tauri development

## Development Commands
```bash
# Install dependencies
npm install

# Run development server
npm run tauri dev

# Build for production
npm run tauri build

# Run frontend only
npm run dev
```

## Next Steps
1. Implement global hotkey registration (Ctrl+K)
2. Create SearchBar UI component
3. Set up Rust backend with search providers
4. Implement keyboard navigation
5. Add search result display

## Requirements
- Node.js 18+
- Rust 1.70+
- Windows 10/11
