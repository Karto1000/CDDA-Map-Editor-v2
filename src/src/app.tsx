import React, {createContext, useEffect, useState} from 'react';
import {Header} from "./components/header.tsx";
import {Theme, useTheme} from "./hooks/useTheme.tsx";
import Window from "./components/window.tsx";
import {invoke} from "@tauri-apps/api/core";
import {EditorData} from "./lib/editor_data/recv";
import {TabType, useTabs, UseTabsReturn} from "./hooks/useTabs.ts";
import {NoTabScreen} from "./mainScreens/noTabScreen.tsx";
import {WelcomeScreen} from "./mainScreens/welcomeScreen.tsx";

export const ThemeContext = createContext<{ theme: Theme, setTheme: (theme: Theme) => void }>({
    theme: Theme.Dark,
    setTheme: () => {
    }
});

export const TabContext = createContext<UseTabsReturn>({
    tabs: [],
    openedTab: null,
    addTab: () => {
    },
    removeTab: () => {
    },
    setOpenedTab: () => {
    }
})

function MainEditor() {
    return null;
}

function App() {
    const [theme, setTheme] = useTheme();
    const [isSettingsWindowOpen, setIsSettingsWindowOpen] = useState<boolean>(false);
    const tabs = useTabs()

    useEffect(() => {
        (async () => {
            const data = await invoke<EditorData>("get_editor_data", {})

            if (!data.config.cdda_path) {
                tabs.addTab(
                    {
                        name: "Welcome to the CDDA Map Editor",
                        type: TabType.Welcome,
                        icon: null
                    }
                )

                tabs.setOpenedTab(0)
            }
        })()
    }, []);

    function getMainBasedOnTab(): React.JSX.Element {
        if (tabs.openedTab !== null) {
            if (tabs.tabs[tabs.openedTab].type === TabType.MapEditor)
                return <MainEditor/>
            if (tabs.tabs[tabs.openedTab].type === TabType.Welcome)
                return <WelcomeScreen/>
        }

        return <NoTabScreen/>
    }

    return (
        <div className={`app ${theme}-theme`}>
            <ThemeContext.Provider value={{theme, setTheme}}>
                <TabContext.Provider value={tabs}>
                    <Header
                        isSettingsWindowOpen={isSettingsWindowOpen}
                        setIsSettingsWindowOpen={setIsSettingsWindowOpen}
                    />
                </TabContext.Provider>

                <Window isOpen={isSettingsWindowOpen} title={"Settings"} setIsOpen={setIsSettingsWindowOpen}>
                    <button onClick={() => setTheme(theme === Theme.Dark ? Theme.Light : Theme.Dark)}>Switch Theme
                    </button>
                </Window>

                {getMainBasedOnTab()}
            </ThemeContext.Provider>
        </div>
    );
}

export default App;
