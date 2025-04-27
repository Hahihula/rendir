### Project Overview: Rustpress

Rustpress will be a flexible static site generator written in Rust, with both CLI and WASM build targets. It will convert markdown content (with enhanced capabilities) into static sites using Vue.js for theming and interactivity.

Here's the refined project specification based on our discussion:

1. **Core Architecture**
   - Primary CLI tool with full functionality
   - Core generator compilable to WASM for playground/demo usage
   - Support for different templates (blog, documentation, slides)
   - Plugin system for extending functionality

2. **Content Processing**
   - Markdown-based content with folder structure organization
   - Support for metadata (tags, categories, dates)
   - Enhanced markdown with:
     - Custom components via both special markdown syntax and HTML-like tags
     - Shortcodes for common content patterns
   - Content preprocessing capabilities via custom Rust functions

3. **Template & Theme System**
   - Templates as Rust modules dropped in before build time
   - Themes as pre-built Vue.js components/templates
   - Default themes for each template type
   - Configuration via TOML or JSON (no YAML)

4. **Output & Features**
   - Single-page application as default output
   - Client-side full-text search (using existing libraries if suitable)
   - Asset handling (basic copying in v1, processing in future versions)
   - SEO-friendly output structure
   - Optimized for output performance and flexibility

5. **Development & Testing**
   - Unit tests for core functionality
   - Designed for CI integration
   - Primary deployment targets: GitHub Pages and GitLab Pages

6. **WASM Interface**
   ```js
   const rustpress = await Rustpress.init();
   const html = rustpress.renderMarkdown(markdownString, options);
   ```

7. **Future Enhancements**
   - Server-side rendering via Vite
   - Dev mode with live reload
   - Enhanced asset processing (image optimization, link checking, video processing)
   - Multi-page output option

### Next Steps

Before diving into implementation, we should:

1. Research suitable markdown parsers that compile to WASM -> pulldown-cmark 
2. Design the core data structures and module interfaces
3. Plan the specific implementation approach for custom components and shortcodes
4. Define the plugin API structure