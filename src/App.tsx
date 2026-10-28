import "./App.css";
import PluginWindow from "./Plugin";
import { getCurrentWindow } from "@tauri-apps/api/window";

function App() {
    const appWindow = getCurrentWindow();
    return (
        <main>
            <div className="flex flex-row">
                <PluginWindow onClose={() => { appWindow.hide(); }} />
            </div>
        </main>
    );
}

export default App;
