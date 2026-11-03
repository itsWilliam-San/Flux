import "./App.css";
import "./i18n.ts";
import PluginWindow from "./Plugin";
import ModalTitleBar from "./components/ModalTitleBar";
import { useState, useEffect } from "react";
import SettingsWindow from "./Settings";
import { invoke } from "@tauri-apps/api/core";

export const enum Windows {
    Settings,
    Plugins,
}

function App() {
    const [currentWindow, setCurrentWindow] = useState<Windows>(Windows.Plugins);
    const [nativeDecorations, setNativeDecorations] = useState<boolean>(() => {
        return localStorage.getItem("window_decorations") === "true";
    });

    useEffect(() => {
        const isDeco = localStorage.getItem("window_decorations") === "true";
        setNativeDecorations(isDeco);
        invoke("set_window_decorations", { decorations: isDeco }).catch((err) => {
            console.error("Failed to apply initial window decorations:", err);
        });
    }, []);

    const handleDecorationsChange = (enabled: boolean) => {
        setNativeDecorations(enabled);
    };

    return (
        <main className="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground">
            {!nativeDecorations && (
                <ModalTitleBar
                    onSettings={() => setCurrentWindow(Windows.Settings)}
                    showSettings={currentWindow === Windows.Plugins}
                />
            )}

            <div className="flex min-h-0 flex-1 overflow-hidden">
                {currentWindow === Windows.Settings && (
                    <SettingsWindow
                        onClose={() => setCurrentWindow(Windows.Plugins)}
                        onDecorationsChange={handleDecorationsChange}
                    />
                )}
                {currentWindow === Windows.Plugins && (
                    <PluginWindow
                        nativeDecorations={nativeDecorations}
                        onOpenSettings={() => setCurrentWindow(Windows.Settings)}
                    />
                )}
            </div>
        </main>
    );
}

export default App;
