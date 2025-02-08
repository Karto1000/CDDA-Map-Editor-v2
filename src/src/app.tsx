import React, {createContext, useEffect, useState} from 'react';
import {Header} from "./components/header.tsx";
import {Theme, useTheme} from "./hooks/useTheme.tsx";
import Window from "./components/window.tsx";
import {invoke} from "@tauri-apps/api/core";
import {EditorConfig, EditorData, EditorDataRecvEvent} from "./lib/editor_data/recv";
import {TabType, useTabs, UseTabsReturn} from "./hooks/useTabs.ts";
import {NoTabScreen} from "./mainScreens/noTabScreen.tsx";
import {WelcomeScreen} from "./mainScreens/welcomeScreen.tsx";
import {listen} from "@tauri-apps/api/event";

export const ThemeContext = createContext<{ theme: Theme, setTheme: (theme: Theme) => void }>({
    theme: Theme.Dark,
    setTheme: () => {
    }
});

export const TabContext = createContext<UseTabsReturn>(null)
export const EditorDataContext = createContext<EditorData>(null)

function MainEditor() {
    return null;
}

function App() {
    const [theme, setTheme] = useTheme();
    const [isSettingsWindowOpen, setIsSettingsWindowOpen] = useState<boolean>(false);
    const [editorData, setEditorData] = useState<EditorData>()
    const tabs = useTabs()

    useEffect(() => {
        (async () => {
            const data = await invoke<EditorData>("get_editor_data", {})

            setEditorData(data)

            await listen<EditorData>(EditorDataRecvEvent.EditorDataChanged, e => {
                setEditorData(e.payload)
            })

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
        // Disable the warning since we do not want to re-run this
        // eslint-disable-next-line react-hooks/exhaustive-deps
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
            <EditorDataContext.Provider value={editorData}>
                <ThemeContext.Provider value={{theme, setTheme}}>
                    <TabContext.Provider value={tabs}>
                        <Header
                            isSettingsWindowOpen={isSettingsWindowOpen}
                            setIsSettingsWindowOpen={setIsSettingsWindowOpen}
                        />

                        <Window isOpen={isSettingsWindowOpen} title={"Settings"} setIsOpen={setIsSettingsWindowOpen}>
                            <button onClick={() => setTheme(theme === Theme.Dark ? Theme.Light : Theme.Dark)}>Switch
                                Theme
                            </button>
                        </Window>

                        {getMainBasedOnTab()}
                    </TabContext.Provider>
                </ThemeContext.Provider>
            </EditorDataContext.Provider>
        </div>
    );
}

export default App;
