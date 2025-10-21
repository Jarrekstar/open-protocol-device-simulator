# Device Simulator Frontend

SvelteKit-based web UI for the Open Protocol Device Simulator.

## Features

- **Real-time WebSocket Updates** - Live device state and event stream
- **Dashboard** - Device status, latest tightening results, statistics
- **Control Panel** - Simulate tightenings, configure auto-tightening, multi-spindle mode
- **Event Log** - Full event history with filtering and search

## Setup

### Prerequisites

- Node.js 18+ and npm

### Installation

```bash
cd frontend
npm install
```

### Environment Configuration

Copy `.env.example` to `.env` and adjust if needed:

```bash
cp .env.example .env
```

Default configuration connects to backend at `http://localhost:8081`.

### Development

Start the dev server:

```bash
npm run dev
```

The UI will be available at `http://localhost:5173`.

### Building for Production

```bash
npm run build
npm run preview
```

## Project Structure

```
frontend/
├── src/
│   ├── lib/
│   │   ├── api/          # HTTP API client
│   │   ├── stores/       # Svelte stores (state management)
│   │   ├── types/        # TypeScript type definitions
│   │   └── components/   # Reusable components (future)
│   ├── routes/           # SvelteKit pages
│   │   ├── +page.svelte         # Dashboard
│   │   ├── control/+page.svelte # Control Panel
│   │   └── events/+page.svelte  # Event Log
│   ├── app.html          # HTML template
│   └── app.css           # Global styles (Tailwind + Skeleton)
├── static/               # Static assets
└── package.json
```

## Technology Stack

- **Framework**: SvelteKit 2.x
- **Language**: TypeScript
- **Styling**: Tailwind CSS + Skeleton UI
- **Build Tool**: Vite
- **Real-time**: Native WebSocket API

## Usage

1. **Start the Rust backend** first (see main README)
2. **Start the frontend** dev server: `npm run dev`
3. **Navigate to** `http://localhost:5173`

The UI will automatically connect to the backend WebSocket and start receiving real-time updates.

### Dashboard

- View current device state
- See latest tightening result
- View statistics
- Quick simulate tightening button

### Control Panel

- **Simulate Tightening**: Send custom tightening parameters
- **Auto-Tightening**: Start/stop automated tightening cycles
- **Multi-Spindle**: Enable/disable multi-spindle mode

### Event Log

- View all events in real-time
- Filter by event type
- Search through events
- Clear event history

## Development

### Type Definitions

All TypeScript types in `src/lib/types/` match the Rust backend types. When backend types change, update these accordingly.

### State Management

Svelte stores in `src/lib/stores/`:
- `device.ts` - Device state
- `events.ts` - Event stream
- `tightening.ts` - Tightening results & statistics
- `websocket.ts` - WebSocket connection management
- `ui.ts` - UI state (toasts, modals)

### Adding New Pages

1. Create `src/routes/your-page/+page.svelte`
2. Add navigation link in `src/routes/+layout.svelte`

### Adding New Components

Create components in `src/lib/components/` and import them in pages.
