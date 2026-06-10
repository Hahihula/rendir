// Shared search engine for Rendir Vue templates
// Provides ClientSearchIndex class and RendirSearch Vue component

// ============================================================
// ClientSearchIndex - Trigram-based full-text search engine
// ============================================================
class ClientSearchIndex {
    constructor(indexData, docsText) {
        this.postings = indexData.postings || {};
        this.documents = indexData.documents || [];
        this.docsText = docsText || [];
    }

    search(query, limit = 10) {
        const queryLower = query.toLowerCase();
        const queryTris = this._trigrams(queryLower);
        if (queryTris.length === 0) return [];

        const results = [];

        for (let idx = 0; idx < this.documents.length; idx++) {
            const doc = this.documents[idx];
            const docId = idx;
            let matched = 0;
            let minPosition = Infinity;

            for (const tri of queryTris) {
                const postings = this.postings[tri] || [];
                for (const posting of postings) {
                    if (posting.doc_id === docId) {
                        matched++;
                        if (posting.positions[0] < minPosition) {
                            minPosition = posting.positions[0];
                        }
                        break;
                    }
                }
            }

            if (matched === 0) continue;

            const recall = matched / Math.max(queryTris.length, 1);
            let score = 2.0 * recall;

            if (doc.title.toLowerCase().includes(queryLower)) {
                score += 1.0;
            }

            const text = this.docsText[idx] || '';
            if (text.includes(queryLower)) {
                score += 0.5;
            }

            if (matched > 0) {
                score += 0.3 * (1 / (1 + minPosition / 1000));
            }

            results.push({
                id: doc.id,
                title: doc.title,
                url: doc.url,
                tags: doc.tags || [],
                score,
                matchPosition: minPosition === Infinity ? 0 : minPosition
            });
        }

        results.sort((a, b) => b.score - a.score);
        return results.slice(0, limit);
    }

    extractSnippet(text, position) {
        if (!text) return '';
        const pos = position;
        const start = Math.max(0, pos - 50);
        const end = Math.min(text.length, pos + 150);
        let snippet = text.slice(start, end);
        if (start > 0) snippet = '...' + snippet;
        if (end < text.length) snippet = snippet + '...';
        return snippet;
    }

    _trigrams(query) {
        if (query.length < 3) {
            return query.length === 0 ? [] : [query];
        }
        const tris = [];
        for (let i = 0; i <= query.length - 3; i++) {
            tris.push(query.slice(i, i + 3));
        }
        return tris;
    }
}

// ============================================================
// RendirSearch - Vue 3 component for search input + results
// ============================================================
// Props:
//   searchIndex (String) - JSON-serialized BuiltSearchIndex from Rust
//   docsText (Array) - Array of plain text strings indexed by doc_id
//   placeholder (String) - Input placeholder text
//   minQueryLength (Number) - Minimum characters before searching (default 2)
//
// Events:
//   update:query - Emitted when the user types
//   navigate(url) - Emitted when a result is clicked
const RendirSearch = {
    name: 'RendirSearch',
    props: {
        searchIndex: { type: String, default: '' },
        docsText: { type: Array, default: () => [] },
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
                engine = new ClientSearchIndex(JSON.parse(props.searchIndex), props.docsText);
            } catch (e) {
                console.error('RendirSearch: parse error', e);
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

        function getSnippet(result) {
            if (!engine || !result) return '';
            const idx = engine.documents.findIndex(d => d.id === result.id);
            if (idx < 0) return '';
            const text = engine.docsText[idx] || '';
            return engine.extractSnippet(text, result.matchPosition || 0);
        }

        return { query, results, handleNavigate, getSnippet };
    },
    template: `
        <div class="rendir-search">
            <input
                type="text"
                v-model="query"
                :placeholder="placeholder"
                autocomplete="off"
                class="rendir-search-input"
            />
            <div class="rendir-search-results" v-if="results !== null">
                <template v-if="results.length > 0">
                    <div v-for="r in results" :key="r.id"
                         class="rendir-search-result"
                         @click="handleNavigate(r.url)">
                        <div class="rst-sr-title">{{ r.title }}</div>
                        <div class="rst-sr-snippet">{{ getSnippet(r) }}</div>
                    </div>
                </template>
                <div v-else class="rst-sr-none">No results found</div>
            </div>
        </div>
    `
};

// Export for use in templates
window.ClientSearchIndex = ClientSearchIndex;
window.RendirSearch = RendirSearch;
