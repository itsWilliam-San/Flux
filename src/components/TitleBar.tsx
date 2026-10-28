import { Settings, X, Minus, Maximize2, Microscope } from "lucide-react";
import { useState } from 'react';
import { getCurrentWindow } from "@tauri-apps/api/window";


export default function TitleBar() {
    const appWindow = getCurrentWindow();

    const [title] = useState('No File');

    return (
        <div className="data-tauri-drag-region grid grid-rows-1 grid-cols-3 bg-background dark:bg-background dark:bg-background pt-2 pb-2 pl-3 pr-3">
            <div className="flex flex-row flex-nowrap items-center content-center justify-start gap-3">
                <Settings className="w-5 h-5 cursor-pointer stroke-black dark:stroke-white" />
                <Microscope className="w-5 h-5 cursor-pointer stroke-black dark:stroke-white" />
            </div>
            <div className="text-sm self-center justify-self-center select-none">
                <span className="text-dark dark:text-white">{title}</span>
            </div>
            <div className="flex flex-row flex-nowrap content-stretch justify-end gap-3">
                <Minus className="w-5 h-5 cursor-pointer stroke-black dark:stroke-white self-end" onClick={() => appWindow.minimize()} />
                <Maximize2 className="w-5 h-5 cursor-pointer dark:stroke-white self-center" onClick={() => appWindow.toggleMaximize()} />
                <X className="w-5 h-5 cursor-pointer stroke-black dark:stroke-white self-center" onClick={() => appWindow.close()} />
            </div>
        </div>
    );
}
