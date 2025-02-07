import React, {createContext, useState} from 'react';
import {Header} from "./components/header.tsx";
import {Theme, useTheme} from "./hooks/useTheme.tsx";
import MainEditor from "./main.tsx";
import Window from "./components/window.tsx";
import {StartupMain} from "./startupMain.tsx";

export const ThemeContext = createContext<{ theme: Theme, setTheme: (theme: Theme) => void }>({
    theme: Theme.Dark,
    setTheme: () => {
    }
});

function App() {
    const [theme, setTheme] = useTheme();
    const [isSettingsWindowOpen, setIsSettingsWindowOpen] = useState<boolean>(false);
    const [isDirSelected, setIsDirSelected] = useState<boolean>(false)

    return (
        <div className={`app ${theme}-theme`}>
            <header>
            </header>
            <ThemeContext.Provider value={{theme, setTheme}}>
                <Window isOpen={isSettingsWindowOpen} title={"Settings"} setIsOpen={setIsSettingsWindowOpen}>
                    <button onClick={() => setTheme(theme === Theme.Dark ? Theme.Light : Theme.Dark)}>Switch Theme
                    </button>
                </Window>
                <Header
                    isSettingsWindowOpen={isSettingsWindowOpen}
                    setIsSettingsWindowOpen={setIsSettingsWindowOpen}
                />
                {
                    isDirSelected &&
                    <MainEditor/>
                }
                {
                    !isDirSelected &&
                    <StartupMain/>
                }
            </ThemeContext.Provider>
        </div>
    );
}

export default App;
