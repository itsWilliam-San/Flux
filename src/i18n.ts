import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import ptBR from './locales/pt-BR.json';
import enUS from './locales/en-US.json';

export function getInitialLanguage(): string {
    const saved = localStorage.getItem('app_language_preference');
    if (saved === 'pt-BR' || saved === 'en-US') {
        return saved;
    }
    const sysLang = (typeof navigator !== 'undefined' && navigator.language) || '';
    return sysLang.toLowerCase().startsWith('pt') ? 'pt-BR' : 'en-US';
}

const initialLang = getInitialLanguage();

i18n.use(initReactI18next).init({
    resources: {
        'pt-BR': { translation: ptBR },
        'en-US': { translation: enUS },
    },
    lng: initialLang,
    fallbackLng: 'en-US',
    interpolation: {
        escapeValue: false,
    },
});

i18n.on('languageChanged', (lng) => {
    localStorage.setItem('app_language_preference', lng);
});

export default i18n;
