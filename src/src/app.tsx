import React, {createContext, useEffect, useState} from 'react';
import {Header} from "./components/header.tsx";
import {Theme, useTheme} from "./hooks/useTheme.tsx";
import Window from "./components/window.tsx";
import {invoke} from "@tauri-apps/api/core";
import {EditorData, EditorDataRecvEvent} from "./lib/editor_data/recv";
import {TabType, useTabs, UseTabsReturn} from "./hooks/useTabs.ts";
import {NoTabScreen} from "./mainScreens/noTabScreen.tsx";
import {WelcomeScreen} from "./mainScreens/welcomeScreen.tsx";
import {listen} from "@tauri-apps/api/event";
import {makeCancelable} from "./lib";
import {MapDataSendCommand} from "./lib/map_data/send";
import MapEditor from "./mainScreens/mapEditor.tsx";

export const ThemeContext = createContext<{ theme: Theme, setTheme: (theme: Theme) => void }>({
    theme: Theme.Dark,
    setTheme: () => {
    }
});

export const TabContext = createContext<UseTabsReturn>(null)
export const EditorDataContext = createContext<EditorData>(null)

function App() {
    const [theme, setTheme] = useTheme();
    const [editorData, setEditorData] = useState<EditorData>()
    const [creatingMapName, setCreatingMapName] = useState<string>("")
    const tabs = useTabs()

    const [isSettingsWindowOpen, setIsSettingsWindowOpen] = useState<boolean>(false);
    const [isCreatingMapWindowOpen, setIsCreatingMapWindowOpen] = useState<boolean>(false);

    useEffect(() => {
        let unlistenDataChanged = makeCancelable(listen<EditorData>(
            EditorDataRecvEvent.EditorDataChanged,
            async (e) => {
                setEditorData(e.payload)

                if (!e.payload.config.cdda_path) {
                    tabs.addTab(
                        {
                            name: "Welcome to the CDDA Map Editor",
                            tab_type: TabType.Welcome,
                        }
                    )

                    tabs.setOpenedTab(0)
                }
            }))

        invoke("frontend_ready", {})

        return () => {
            unlistenDataChanged.cancel()
        }

        // Disable the warning since we do not want to re-run this
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    function getMainBasedOnTab(): React.JSX.Element {
        if (tabs.openedTab !== null) {
            if (tabs.tabs[tabs.openedTab].tab_type === TabType.MapEditor)
                return <MapEditor/>
            if (tabs.tabs[tabs.openedTab].tab_type === TabType.Welcome)
                return <WelcomeScreen/>
        }

        return <NoTabScreen/>
    }

    async function createMap() {
        await invoke(MapDataSendCommand.CreateMap, {data: {name: creatingMapName, size: "24,24"}})

        setIsCreatingMapWindowOpen(false)
        setCreatingMapName("")
    }

    return (
        <div className={`app ${theme}-theme`}>
            <EditorDataContext.Provider value={editorData}>
                <ThemeContext.Provider value={{theme, setTheme}}>
                    <TabContext.Provider value={tabs}>
                        <Header
                            isSettingsWindowOpen={isSettingsWindowOpen}
                            setIsSettingsWindowOpen={setIsSettingsWindowOpen}
                            isCreatingMapWindowOpen={isCreatingMapWindowOpen}
                            setIsCreatingMapWindowOpen={setIsCreatingMapWindowOpen}
                        />

                        <Window isOpen={isSettingsWindowOpen} title={"Settings"} setIsOpen={setIsSettingsWindowOpen}>
                            <button onClick={() => setTheme(theme === Theme.Dark ? Theme.Light : Theme.Dark)}>Switch
                                Theme
                            </button>
                        </Window>

                        <Window title={"Create a new Map"} isOpen={isCreatingMapWindowOpen}
                                setIsOpen={setIsCreatingMapWindowOpen}>
                            <label htmlFor={"map-name"}>Map Name</label>
                            <input name={"map-name"} value={creatingMapName}
                                   onChange={e => setCreatingMapName(e.target.value)}/>
                            <button onClick={createMap}>
                                Create
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
