import { Search } from 'lucide-react';
import { useTranslation } from 'react-i18next';

interface SearchProps {
    value: string;
    onChange: (value: string) => void;
}

export default function SearchComponent({ value, onChange }: SearchProps) {
    const { t } = useTranslation();
    return (
        <div className="flex flex-row items-start border-black dark:border-white border-1 rounded-xl p-2 gap-4">
            <Search className="w-5 h-5 stroke-black dark:stroke-white" />
            <input
                type="search"
                placeholder={t('placeholders.search_plugin')}
                value={value}
                onChange={(e) => onChange(e.target.value)}
                className="bg-transparent w-full text-zinc-800 dark:text-zinc-100 text-sm placeholder:text-zinc-500 outline-none"
            />
        </div>
    );
}
