# GUI Design Considerations

## Why GUI Instead of CLI/TUI?

The user's point is valid: **If it's just terminal commands, why not use the terminal directly?**

A GUI provides value through:
1. **Visual Organization**: Tables, cards, sidebars - easier to scan than terminal output
2. **One-Click Actions**: Buttons instead of typing commands
3. **Bulk Operations**: Select multiple items, update all at once
4. **Progress Visualization**: Real-time progress bars, not just text output
5. **Filtering & Sorting**: Click column headers, search bars, dropdowns
6. **Visual Status**: Color coding, icons, badges - instant understanding
7. **Responsive Layout**: Resizable windows, collapsible panels
8. **Notifications**: Toast notifications for completed actions

## GUI Framework Choice: egui

### Why egui?

**egui** (immediate mode GUI) is perfect for this use case:

1. **Tool/Dashboard Focus**: Built for exactly this type of application
2. **Great Tables**: Excellent table widget support (perfect for package lists)
3. **Cross-Platform**: Works on macOS, Windows, Linux
4. **Easy to Use**: Immediate mode is simpler than retained mode
5. **Good Performance**: Fast enough for our needs
6. **Active Development**: Popular in Rust community, well-maintained
7. **Native Feel**: Looks native on each platform

### egui Example Structure

```rust
use eframe::egui;

struct DepMgrApp {
    packages: Vec<Package>,
    selected_manager: Option<PackageManager>,
    search_query: String,
}

impl eframe::App for DepMgrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Sidebar
            egui::SidePanel::left("sidebar").show(ctx, |ui| {
                ui.heading("Package Managers");
                // Manager filters
            });
            
            // Main content
            egui::CentralPanel::default().show(ctx, |ui| {
                // Search bar
                ui.text_edit_singleline(&mut self.search_query);
                
                // Package table
                egui::Grid::new("packages")
                    .num_columns(6)
                    .show(ui, |ui| {
                        // Table headers and rows
                    });
            });
        });
    }
}
```

### Alternative: Iced

**Iced** (declarative, Elm-inspired) is another option:

**Pros:**
- More structured approach
- Better for complex UIs
- Type-safe message passing

**Cons:**
- Steeper learning curve
- More boilerplate
- Less immediate for simple dashboards

### Alternative: Tauri

**Tauri** (web frontend + Rust backend):

**Pros:**
- Modern web UI capabilities
- Can use React/Vue/etc.
- Very polished UIs possible

**Cons:**
- Larger bundle size (includes web runtime)
- More complex setup
- Overkill for this use case

**Recommendation**: Start with **egui** - it's the right tool for this job.

## GUI Layout Design

### Main Window Structure

```
┌─────────────────────────────────────────────────────────┐
│  [Logo] Dependency Manager                    [⚙️] [❌] │
├──────────┬──────────────────────────────────────────────┤
│          │  [Search: ___________]  [Filter ▼] [Refresh] │
│          ├──────────────────────────────────────────────┤
│          │  Name      │ Manager │ Version │ Status │ ...│
│          ├──────────────────────────────────────────────┤
│ Sidebar  │ ☑ git      │ brew    │ 2.42.0  │ ⚠️     │ ...│
│          │ ☑ node     │ brew    │ 20.8.0  │ ⚠️     │ ...│
│ [Filters]│ ☐ vim      │ brew    │ 9.0.0   │ ⚪     │ ...│
│          │ ☑ python   │ brew    │ 3.11.0  │ ⚠️     │ ...│
│ • Homebrew│ ...                                        │
│ • npm     │                                            │
│ • Cargo   │                                            │
│          │                                            │
│ Stats:   │                                            │
│ • 91 pkgs │                                            │
│ • 22 ⚠️   │                                            │
│ • 5 ⚪    │                                            │
│          │                                            │
│ [Update  │                                            │
│  Selected]                                            │
│ [Update  │                                            │
│  All ⚠️]  │                                            │
└──────────┴──────────────────────────────────────────────┘
```

### Key UI Components

1. **Left Sidebar**:
   - Package manager filters (checkboxes)
   - Quick stats (total, outdated, orphaned)
   - Action buttons

2. **Top Bar**:
   - Search input
   - Filter dropdown (All, Outdated, Orphaned, Up-to-date)
   - Refresh button

3. **Main Table**:
   - Sortable columns
   - Checkboxes for selection
   - Color-coded rows
   - Click row → show details panel

4. **Details Panel** (right side, collapsible):
   - Package name and description
   - Version info
   - Size and install date
   - Projects using it
   - Update button

5. **Update Progress Dialog** (modal):
   - Progress bar per package
   - Status text
   - Cancel button
   - Error display

## egui Widgets We'll Use

- `egui::Table`: For package list (sortable columns)
- `egui::SidePanel`: Left sidebar
- `egui::CentralPanel`: Main content area
- `egui::TextEdit`: Search bar
- `egui::Button`: Action buttons
- `egui::Checkbox`: Filters and selections
- `egui::ProgressBar`: Update progress
- `egui::ComboBox`: Filter dropdown
- `egui::Label`: Text display
- `egui::Separator`: Visual dividers
- `egui::CollapsingHeader`: Collapsible sections

## Color Scheme

- **Outdated**: Red/orange (`#FF6B6B` or `#FFA500`)
- **Orphaned**: Gray (`#888888`)
- **Up-to-date**: Green (`#51CF66`)
- **In Progress**: Blue (`#4DABF7`)
- **Error**: Dark red (`#C92A2A`)

## Responsive Design

- **Minimum Window Size**: 800x600
- **Resizable**: All panels resize proportionally
- **Collapsible Sidebar**: Can collapse to save space
- **Table Scrolling**: Virtual scrolling for large lists

## User Interactions

1. **Click Column Header**: Sort by that column
2. **Click Checkbox**: Select/deselect package
3. **Click Package Name**: Show details panel
4. **Click Update Button**: Start update (show progress dialog)
5. **Type in Search**: Filter packages in real-time
6. **Select Filter**: Show only matching packages
7. **Click Refresh**: Re-scan all package managers

## Update Flow

1. User clicks "Update Selected" or "Update All Outdated"
2. Confirmation dialog appears (optional, configurable)
3. Progress dialog opens showing:
   - List of packages being updated
   - Progress bar per package
   - Overall progress
   - Status messages
4. Updates happen in background (async)
5. Toast notification when complete
6. Table refreshes automatically

## Error Handling

- **Command Errors**: Show in progress dialog, continue with other packages
- **Permission Errors**: Show clear message, suggest solution
- **Network Errors**: Show retry button
- **Parse Errors**: Log but don't crash, show warning icon

## Performance Considerations

- **Virtual Scrolling**: Only render visible table rows
- **Lazy Loading**: Load package details on demand
- **Caching**: Cache scan results, refresh on demand
- **Async Updates**: Don't block UI during updates
- **Debounced Search**: Don't filter on every keystroke

## Accessibility

- **Keyboard Navigation**: Tab through controls, Enter to activate
- **Screen Reader Support**: Proper labels (egui has basic support)
- **High Contrast**: Respect system theme
- **Font Scaling**: Respect system font size

## Future Enhancements

- **Dark Mode**: Toggle between light/dark themes
- **Customizable Columns**: Show/hide columns
- **Export**: Export package list to CSV/JSON
- **Import**: Import package list from file
- **Keyboard Shortcuts**: Hotkeys for common actions
- **Tray Icon**: Minimize to tray (like klippy/dev-ports-tray)

