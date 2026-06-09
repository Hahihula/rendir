// Language Selector Vue 3 component for Rendir
// Props:
//   languages (Array) — List of { code, name, is_default }
//   currentLang (String) — Currently active language code

const LanguageSelector = {
    name: 'LanguageSelector',
    props: {
        languages: {
            type: Array,
            default: () => []
        },
        currentLang: {
            type: String,
            default: 'en'
        }
    },
    setup(props) {
        const { ref, computed } = Vue;

        const isOpen = ref(false);

        function getLanguageUrl(targetLangCode) {
            const path = window.location.pathname;
            const pathParts = path.split('/').filter(p => p);
            
            if (pathParts.length === 0) {
                return '/' + targetLangCode + '/';
            }
            
            const firstPart = pathParts[0];
            const isLangPrefix = props.languages.some(l => l.code === firstPart);
            
            if (isLangPrefix && pathParts.length >= 1) {
                return '/' + targetLangCode + '/' + pathParts.slice(1).join('/');
            } else {
                return '/' + targetLangCode + '/' + pathParts.join('/');
            }
        }

        const currentLanguage = computed(() => {
            return props.languages.find(l => l.code === props.currentLang) || {
                code: props.currentLang,
                name: props.currentLang.toUpperCase()
            };
        });

        const otherLanguages = computed(() => {
            return props.languages
                .filter(l => l.code !== props.currentLang)
                .map(lang => ({
                    ...lang,
                    url: getLanguageUrl(lang.code)
                }));
        });

        const hasMultipleLanguages = computed(() => props.languages.length > 1);

        function toggleDropdown() {
            isOpen.value = !isOpen.value;
        }

        function closeDropdown() {
            isOpen.value = false;
        }

        return {
            isOpen,
            currentLanguage,
            otherLanguages,
            hasMultipleLanguages,
            toggleDropdown,
            closeDropdown
        };
    },
    template: '<div v-if="hasMultipleLanguages" class="lang-selector"><button @click="toggleDropdown">{{ currentLanguage.name || currentLanguage.code }}</button><ul v-if="isOpen"><li v-for="lang in otherLanguages" :key="lang.code"><a :href="lang.url">{{ lang.name || lang.code }}</a></li></ul></div>'
};

window.LanguageSelector = LanguageSelector;