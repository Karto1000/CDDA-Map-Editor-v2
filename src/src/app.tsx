import React, {useState} from 'react';
import {Header} from "./components/header.tsx";
import {useTheme} from "./hooks/useTheme.tsx";
import Main from "./main.tsx";
import Window from "./components/window.tsx";

function App() {
    const [theme, _] = useTheme();

    const [isSettingsWindowOpen, setIsSettingsWindowOpen] = useState<boolean>(false);

    return (
        <div className={`app ${theme}-theme`}>
            <header>
            </header>
            <Window isOpen={isSettingsWindowOpen} title={"Settings"} setIsOpen={setIsSettingsWindowOpen}>
                <div></div>
            </Window>
            <Header
                isSettingsWindowOpen={isSettingsWindowOpen}
                setIsSettingsWindowOpen={setIsSettingsWindowOpen}
            />
            <Main/>
        </div>
    );
}

export default App;
