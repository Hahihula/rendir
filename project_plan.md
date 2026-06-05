### Rustpress Project Plan

#### Project Overview
Rustpress is a flexible static site generator written in Rust, with both CLI and WASM targets. It converts markdown content (with enhanced capabilities) into static sites using Vue.js for theming and interactivity.

#### Technology Stack
- **Core Language**: Rust
- **Markdown Parser**: pulldown-cmark
- **UI Framework**: Vue.js
- **Build Target**: Native (CLI) and WebAssembly (WASM)

#### Core Components Architecture

1. **Core Library** (shared between CLI and WASM)
   ```
   core/
   ├── markdown/       # Enhanced pulldown-cmark with component support
   ├── content/        # Content organization and metadata handling
   ├── render/         # HTML generation pipeline
   └── types/          # Shared type definitions
   ```

2. **CLI Application**
   ```
   cli/
   ├── commands/       # CLI commands (build, serve, etc.)
   ├── config/         # Configuration handling
   └── utils/          # CLI-specific utilities
   ```

3. **WASM Interface**
   ```
   wasm/
   ├── bindings/       # JavaScript API bindings
   └── demo/           # Browser playground
   ```

4. **Templates & Themes**
   ```
   templates/          # Built-in templates (blog, docs, slides)
   themes/             # Default Vue.js themes
   ```

5. **Extension Points**
   ```
   plugins/            # Plugin system
   ```

#### Key Data Structures

```rust
// Core content representation
struct ContentItem {
    path: PathBuf,
    content: String,
    metadata: HashMap<String, Value>,
    rendered_content: Option<String>,
    related_items: Vec<RelatedContent>,
}

// Complete generated site
struct Site {
    output_path: PathBuf,
    pages: Vec<Page>,
    assets: Vec<Asset>,
    search_index: Option<SearchIndex>,
    metadata: HashMap<String, Value>,
}

// Individual page in the site
struct Page {
    path: String,
    html: String,
    metadata: HashMap<String, Value>,
    assets: Vec<Asset>,
}

// Site asset (images, CSS, JS, etc.)
struct Asset {
    source: PathBuf,
    destination: String,
    needs_processing: bool,
    asset_type: AssetType,
}

// Site search index
struct SearchIndex {
    entries: Vec<SearchEntry>,
    format: SearchIndexFormat,
}
```

#### Key Interfaces

```rust
// Template interface
trait Template {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn render(&self, content: &[ContentItem], config: &Config) -> Result<Site>;
}

// Plugin interface
trait Plugin {
    fn name(&self) -> &str;
    fn on_init(&self, config: &Config) -> Result<()>;
    fn on_content_loaded(&self, items: &mut [ContentItem]) -> Result<()>;
    fn on_pre_render(&self, items: &mut [ContentItem]) -> Result<()>;
    fn on_post_render(&self, site: &mut Site) -> Result<()>;
}

// WASM API
#[wasm_bindgen]
impl Rustpress {
    pub fn new() -> Self;
    pub fn render_markdown(&self, content: &str, options: JsValue) -> String;
    pub fn render_site(&self, content: &[u8], options: JsValue) -> Vec<u8>;
}
```

#### Implementation Phases

1. **Phase 1: Core Markdown Processing**
   - Implement basic markdown -> HTML conversion using pulldown-cmark
   - Ensure WASM compatibility from the start
   - Create minimal test cases

2. **Phase 2: WASM Integration**
   - Build WASM bindings for core functionality
   - Create simple browser demo for markdown rendering
   - Test cross-platform compatibility

3. **Phase 3: CLI Development**
   - Implement basic CLI structure
   - Add file system handling for content
   - Create configuration system (TOML parsing)

4. **Phase 4: Enhanced Markdown**
   - Implement custom component system
   - Add shortcode support
   - Extend pulldown-cmark for custom syntax

5. **Phase 5: Template System**
   - Create template interface
   - Implement basic templates (blog, docs, slides)
   - Build theme integration with Vue.js

6. **Phase 6: Advanced Features**
   - Implement search indexing
   - Add asset optimization
   - Build plugin system
   - Create deployment integrations

#### Testing Strategy

1. **Unit Tests**
   - Core markdown parsing and rendering
   - Component and shortcode processing
   - Configuration handling

2. **Integration Tests**
   - End-to-end CLI functionality
   - Template rendering with different inputs
   - WASM API functionality

3. **Cross-platform Tests**
   - Ensure consistent behavior between native and WASM targets

#### Project Structure (Cargo Workspace)

```
rustpress/
├── Cargo.toml           # Workspace definition
├── rustpress-core/      # Core library (works in both contexts)
├── rustpress/       # CLI application
├── rustpress-wasm/      # WASM bindings
├── rustpress-templates/ # Built-in templates
└── examples/            # Example projects
```