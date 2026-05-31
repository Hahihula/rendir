// Shared search engine for Rustpress Vue templates
// Provides ClientSearchIndex class and a Vue component that can be registered globally

// ============================================================
// ClientSearchIndex — Full-text search engine (mirror of Rust's BuiltSearchIndex)
// ============================================================
class ClientSearchIndex {
    constructor(indexData) {
        this.index = indexData.index || {};
        this.documents = indexData.documents || [];
    }

    search(query, limit = 10) {
        const queryLower = query.toLowerCase();
        const queryWords = queryLower.split(/\s+/).filter(w => w.length > 0);
        if (queryWords.length === 0) return [];

        const scores = {};
        for (const word of queryWords) {
            const docs = this.index[word] || [];
            for (const doc of docs) {
                scores[doc.id] = (scores[doc.id] || 0) + 1.0;
            }
        }

        for (const doc of this.documents) {
            const titleLower = doc.title.toLowerCase();
            const contentLower = doc.content.toLowerCase();
            for (const word of queryWords) {
                if (titleLower.includes(word)) {
                    scores[doc.id] = (scores[doc.id] || 0) + 2.0;
                }
                if (contentLower.includes(word)) {
                    scores[doc.id] = (scores[doc.id] || 0) + 1.0;
                }
            }
        }

        let results = Object.entries(scores)
            .filter(([_, score]) => score > 0)
            .map(([id, score]) => {
                const doc = this.documents.find(d => d.id === id);
                if (!doc) return null;
                return {
                    id: doc.id,
                    title: doc.title,
                    url: doc.url,
                    snippet: this.extractSnippet(doc.content, queryLower),
                    score
                };
            })
            .filter(r => r !== null);

        results.sort((a, b) => b.score - a.score);
        return results.slice(0, limit);
    }

    extractSnippet(content, query) {
        const queryWords = query.split(/\s+/).filter(w => w.length > 0);
        const contentLower = content.toLowerCase();
        for (const word of queryWords) {
            const pos = contentLower.indexOf(word);
            if (pos !== -1) {
                const start = Math.max(0, pos - 50);
                const end = Math.min(content.length, pos + 150);
                let snippet = content.slice(start, end);
                if (start > 0) snippet = '...' + snippet;
                if (end < content.length) snippet = snippet + '...';
                return snippet;
            }
        }
        return content.slice(0, 150) + (content.length > 150 ? '...' : '');
    }
}

// ============================================================
// RustpressSearch — Vue 3 component for search input + results
// ============================================================
// Props:
//   searchIndex (String) — JSON-serialized BuiltSearchIndex from Rust
//   placeholder (String) — Input placeholder text
//   minQueryLength (Number) — Minimum characters before searching (default 2)
//
// Events:
//   update:query — Emitted when the user types
//   navigate(url) — Emitted when a result is clicked
const RustpressSearch = {
    name: 'RustpressSearch',
    props: {
        searchIndex: { type: String, default: '' },
        placeholder: { type: String, default: 'Search...' },
        minQueryLength: { type: Number, default: 2 }
    },
    emits: ['update:query', 'navigate'],
    setup(props, { emit }) {
        const { ref, computed, watch } = Vue;
        const query = ref('');

        let engine = null;
        if (props.searchIndex) {
            try {
                engine = new ClientSearchIndex(JSON.parse(props.searchIndex));
            } catch (e) {
                console.error('RustpressSearch: parse error', e);
            }
        }

        const results = computed(() => {
            const q = query.value.trim();
            if (q.length < props.minQueryLength || !engine) return null;
            return engine.search(q, 20);
        });

        watch(query, (val) => emit('update:query', val));

        function handleNavigate(url) {
            emit('navigate', url);
        }

        return { query, results, handleNavigate };
    },
    template: `
        <div class="rustpress-search">
            <input
                type="text"
                v-model="query"
                :placeholder="placeholder"
                autocomplete="off"
                class="rustpress-search-input"
            />
            <div class="rustpress-search-results" v-if="results !== null">
                <template v-if="results.length > 0">
                    <div v-for="r in results" :key="r.id"
                         class="rustpress-search-result"
                         @click="handleNavigate(r.url)">
                        <div class="rst-sr-title">{{ r.title }}</div>
                        <div class="rst-sr-snippet" v-html="r.snippet"></div>
                    </div>
                </template>
                <div v-else class="rst-sr-none">No results found</div>
            </div>
        </div>
    `
};

// Export for use in templates
window.ClientSearchIndex = ClientSearchIndex;
window.RustpressSearch = RustpressSearch;
