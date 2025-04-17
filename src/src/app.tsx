import React, {createContext, useEffect, useRef, useState} from 'react';
import {Header} from "./components/header.tsx";
import {Theme, useTheme} from "./hooks/useTheme.ts";
import Window from "./components/window.tsx";
import {invoke} from "@tauri-apps/api/core";
import {TabTypeKind, useTabs, UseTabsReturn} from "./hooks/useTabs.ts";
import {NoTabScreen} from "./mainScreens/noTabScreen.tsx";
import {WelcomeScreen} from "./mainScreens/welcomeScreen.tsx";
import {listen} from "@tauri-apps/api/event";
import {makeCancelable} from "./lib/index.ts";
import {Scene} from "three";
import {useEditor} from "./hooks/useEditor.tsx";
import {useTileset} from "./hooks/useTileset.ts";
import {EditorData, EditorDataRecvEvent} from "./lib/editor_data.ts";
import {MapDataSendCommand} from "./lib/map_data.ts";
import {Panel, PanelGroup, PanelResizeHandle} from "react-resizable-panels";

import "./app.scss"

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

    const mapEditorCanvasContainerRef = useRef<HTMLDivElement>()
    const mapEditorCanvasRef = useRef<HTMLCanvasElement>();
    const mapEditorSceneRef = useRef<Scene>(new Scene())

    const [tilesheets, isTilesheetLoaded] = useTileset(editorData, mapEditorSceneRef)
    const isDisplayingMapEditor = tabs.tabs[tabs.openedTab]?.tab_type.type === TabTypeKind.MapEditor
    const mapEditorCanvasDisplay = isDisplayingMapEditor ? "flex" : "none"

    const {resize, displayInLeftPanel} = useEditor({
        canvasRef: mapEditorCanvasRef,
        sceneRef: mapEditorSceneRef,
        canvasContainerRef: mapEditorCanvasContainerRef,
        isDisplaying: isDisplayingMapEditor,
        tilesheetsRef: tilesheets,
        openedTab: tabs.openedTab,
        isTilesheetLoaded,
        theme
    })

    useEffect(() => {
        let unlistenDataChanged = makeCancelable(listen<EditorData>(
            EditorDataRecvEvent.EditorDataChanged,
            async (e) => {
                setEditorData(e.payload)

                const welcomeTab = e.payload.tabs.find(t => t.tab_type.type === TabTypeKind.Welcome)

                if (welcomeTab) tabs.setOpenedTab(e.payload.tabs.indexOf(welcomeTab))

                if (!e.payload.config.cdda_path && !welcomeTab) {
                    await tabs.addTab(
                        {
                            name: "Welcome to the CDDA Map Editor",
                            tab_type: {
                                type: TabTypeKind.Welcome,
                            }
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
            if (tabs.tabs[tabs.openedTab].tab_type.type === TabTypeKind.Welcome)
                return <WelcomeScreen/>

            if (tabs.tabs[tabs.openedTab].tab_type.type === TabTypeKind.MapEditor)
                return <></>
        }

        return <NoTabScreen setIsCreatingMapWindowOpen={setIsCreatingMapWindowOpen}/>
    }

    async function createMap() {
        await invoke(MapDataSendCommand.CreateProject, {data: {name: creatingMapName, size: "24,24"}})

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
                                   placeholder={"Map Name"}
                                   onChange={e => setCreatingMapName(e.target.value)}/>
                            <button onClick={createMap}>
                                Create
                            </button>
                        </Window>

                        <PanelGroup direction={'horizontal'}>
                            <Panel defaultSize={20} maxSize={20} onResize={resize}>
                                <div className={"side-panel"}>
                                    <div className={"side-panel-left"}>
                                        {
                                            isDisplayingMapEditor ?
                                                displayInLeftPanel :
                                                <div>
                                                    <h1>Hey there!</h1>
                                                    <p>This is where you can see the properties of any tiles you hover
                                                        over</p>
                                                    <p>If you wish to close this panel you can just drag the Line
                                                        between this panel and the main content to the left</p>
                                                </div>
                                        }
                                    </div>
                                    <div className={"side-panel-right"}/>
                                </div>
                            </Panel>
                            <PanelResizeHandle hitAreaMargins={{coarse: 30, fine: 10}}/>
                            <Panel onResize={resize}>
                                <div ref={mapEditorCanvasContainerRef}
                                     style={{width: "100%", height: "100%", display: mapEditorCanvasDisplay}}>
                                    {/* This should always be in the dom because then we only have to load the sprites once */}
                                    <canvas ref={mapEditorCanvasRef} tabIndex={0}/>
                                </div>
                                {getMainBasedOnTab()}
                            </Panel>
                        </PanelGroup>
                    </TabContext.Provider>
                </ThemeContext.Provider>
            </EditorDataContext.Provider>
        </div>
    );
}

export default App;
