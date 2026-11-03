import { X, Settings } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface ModalTitleBarProps {
    onClose?: () => void;
    onSettings?: () => void;
    showSettings?: boolean;
}

export default function ModalTitleBar({
    onClose,
    onSettings,
    showSettings = true,
}: ModalTitleBarProps) {
    const handleClose = async () => {
        if (onClose) {
            onClose();
            return;
        }

        const closeAction = localStorage.getItem("close_action");
        if (closeAction === "tray") {
            const appWindow = getCurrentWindow();
            await appWindow.hide();
        } else {
            await invoke("exit_app");
        }
    };

    return (
        <div
            data-tauri-drag-region
            className="flex items-center justify-between bg-background dark:bg-background px-3 py-2 border-b border-border select-none"
        >
            <div className="flex items-center gap-2">
                {showSettings && onSettings ? (
                    <button
                        type="button"
                        onClick={onSettings}
                        className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors cursor-pointer"
                        title="Settings"
                    >
                        <Settings className="size-4" />
                    </button>
                ) : null}
            </div>
            <div className="flex items-center gap-2">
                <button
                    type="button"
                    onClick={handleClose}
                    className="rounded p-1 text-muted-foreground hover:bg-destructive/15 hover:text-destructive transition-colors cursor-pointer"
                    title="Close"
                >
                    <X className="size-4" />
                </button>
            </div>
        </div>
    );
}
