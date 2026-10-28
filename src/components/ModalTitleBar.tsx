import { X } from "lucide-react";

interface ModalTitleBarProps {
    onClose: () => void;
}

export default function ModalTitleBar({ onClose }: ModalTitleBarProps) {
    return (
        <div className="flex flex-row justify-end bg-background dark:bg-background pt-2 pb-2 pl-3 pr-3">
            <div className="flex flex-row flex-nowrap content-stretch justify-end gap-3">
                <X className="w-5 h-5 cursor-pointer stroke-black dark:stroke-white self-center" onClick={onClose} />
            </div>
        </div>
    );
}
